use super::state::{MessageVerificationResult, TestState};
use base::constants::{ID, SessionID};
use client::oc_helper::user::TestUser;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Manages background streaming tasks for fetching messages
pub struct StreamingManager {
    /// Active streaming tasks per user
    tasks: dashmap::DashMap<ID, JoinHandle<()>>,
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

impl StreamingManager {
    pub fn new() -> Self {
        Self {
            tasks: dashmap::DashMap::new(),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start a background streaming task for a user
    pub fn start_streaming_for_user(
        &self,
        user_id: ID,
        user: Arc<Mutex<TestUser>>,
        state: Arc<TestState>,
    ) {
        let shutdown = Arc::clone(&self.shutdown);
        let handle = tokio::spawn(async move {
            streaming_task(user_id, user, state, shutdown).await;
        });

        self.tasks.insert(user_id, handle);
        info!("Started streaming task for user {}", user_id.0);
    }

    /// Stop all streaming tasks
    pub fn stop_all(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        for entry in self.tasks.iter() {
            entry.value().abort();
        }
        self.tasks.clear();
        info!("Stopped all streaming tasks");
    }
}

impl Drop for StreamingManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

/// Background task that continuously fetches messages for a user
async fn streaming_task(
    user_id: ID,
    user: Arc<Mutex<TestUser>>,
    state: Arc<TestState>,
    shutdown: Arc<AtomicBool>,
) {
    let mut consecutive_errors = 0;
    const MAX_CONSECUTIVE_ERRORS: usize = 5;

    while !shutdown.load(Ordering::SeqCst) {
        // Fetch messages with a short timeout
        let fetch_result = {
            let mut user_guard = user.lock().await;
            tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                user_guard.fetch_msgs().fetch(100),
            )
            .await
        };

        match fetch_result {
            Ok(Ok(messages)) => {
                consecutive_errors = 0;
                let msg_count = messages.len();
                for msg in &messages {
                    process_received_message(user_id, msg, &state).await;
                }
                debug!("User {} received {} messages", user_id.0, msg_count);
            }
            Ok(Err(e)) => {
                // Error from fetch_msgs - this could be a real error (e.g. rate limit)
                consecutive_errors += 1;
                warn!("Fetch error for user {}: {}", user_id.0, e);

                // Exponential backoff on errors to avoid hammering rate-limited server
                let backoff = tokio::time::Duration::from_millis(
                    500 * 2u64.pow(consecutive_errors.min(4) as u32),
                );
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = async {
                        while !shutdown.load(Ordering::SeqCst) {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    } => {}
                }
            }
            Err(_) => {
                // Timeout is expected when no messages - not an error
                consecutive_errors = 0;
            }
        }

        // Check for too many consecutive errors
        if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
            warn!(
                "Too many consecutive errors for user {}, stopping streaming",
                user_id.0
            );
            break;
        }

        // Small sleep to prevent tight loop
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
            _ = async {
                while !shutdown.load(Ordering::SeqCst) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            } => {}
        }
    }

    info!("Streaming task ended for user {}", user_id.0);
}

/// Process a received message and verify it
async fn process_received_message(
    receiver_id: ID,
    msg: &pb::service::ourchat::msg_delivery::v1::FetchMsgsResponse,
    state: &TestState,
) {
    let msg_id = msg.msg_id;

    // Extract session_id, sender_id, and content from respond_event_type
    let (session_id, sender_id, content) = match &msg.respond_event_type {
        Some(
            pb::service::ourchat::msg_delivery::v1::fetch_msgs_response::RespondEventType::Msg(
                msg_content,
            ),
        ) => {
            let session_id = SessionID(msg_content.session_id);
            let sender_id = ID(msg_content.sender_id);
            let content = msg_content.markdown_text.clone();
            (session_id, sender_id, content)
        }
        _ => {
            // Not a regular message - skip verification
            debug!("Received non-message event type for user {}", receiver_id.0);
            return;
        }
    };

    // Verify the message
    let result =
        state.record_received_message(receiver_id, msg_id, session_id, content.clone(), sender_id);

    match result {
        MessageVerificationResult::Valid => {
            debug!(
                "Valid message {} received by user {}",
                msg_id, receiver_id.0
            );
        }
        MessageVerificationResult::UnknownMessage(_id) => {
            // Message from before tracking started - ignore
            debug!("Unknown message {} for user {}", msg_id, receiver_id.0);
        }
        MessageVerificationResult::ContentMismatch { expected, actual } => {
            error!(
                "Content mismatch for message {}: expected '{}', got '{}'",
                msg_id, expected, actual
            );
        }
        MessageVerificationResult::WrongSession { expected, actual } => {
            error!(
                "Wrong session for message {}: expected {:?}, got {:?}",
                msg_id, expected, actual
            );
        }
        MessageVerificationResult::WrongSender { expected, actual } => {
            error!(
                "Wrong sender for message {}: expected {}, got {}",
                msg_id, expected.0, actual.0
            );
        }
        MessageVerificationResult::UnexpectedRecipient {
            msg_id,
            receiver_id,
            session_id,
        } => {
            warn!(
                "User {} unexpectedly received message {} in session {:?}",
                receiver_id.0, msg_id, session_id
            );
        }
    }
}

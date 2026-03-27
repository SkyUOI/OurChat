use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

pub type ShutdownFinishCallback = oneshot::Sender<()>;
pub type TaskId = u64;
pub type Tasks = Arc<DashMap<TaskId, Task>>;

/// Global task ID counter
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct Task {
    name: String,
    description: String,
    completion_receiver: Option<oneshot::Receiver<()>>,
}

#[derive(Debug, Clone)]
pub struct ShutdownSdr {
    callback: Arc<Mutex<Option<ShutdownFinishCallback>>>,
    tasks: Tasks,
    cancel: CancellationToken,
}

impl ShutdownSdr {
    pub fn new(callback: Option<ShutdownFinishCallback>) -> Self {
        Self {
            callback: Arc::new(Mutex::new(callback)),
            tasks: Arc::new(DashMap::new()),
            cancel: CancellationToken::new(),
        }
    }

    pub fn new_receiver(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> ShutdownRev {
        let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed);
        tracing::trace!("task {} created", task_id);

        let (completion_sdr, completion_rev) = oneshot::channel();
        let task = Task {
            name: name.into(),
            description: description.into(),
            completion_receiver: Some(completion_rev),
        };
        self.tasks.insert(task_id, task);

        ShutdownRev::new(
            self.cancel.child_token(),
            completion_sdr,
            self.tasks.clone(),
            task_id,
        )
    }

    pub async fn shutdown_all_tasks(&mut self) -> anyhow::Result<()> {
        // Signal all tasks simultaneously
        self.cancel.cancel();

        let mut waiting_finished = JoinSet::new();

        for mut entry in self.tasks.iter_mut() {
            let task_id = *entry.key();
            let name = entry.value().name.clone();
            let description = entry.value().description.clone();
            let tasks_ref = self.tasks.clone();

            // Take the completion receiver — if ShutdownRev was already dropped
            // without calling wait_shutting_down, this will be None.
            let completion_rev = {
                let task = entry.value_mut();
                task.completion_receiver.take()
            };

            waiting_finished.spawn(async move {
                if let Some(completion_rev) = completion_rev {
                    if completion_rev.await.is_err() {
                        // Sender dropped without sending (e.g. ShutdownRev
                        // dropped without calling wait_shutting_down)
                        tracing::error!(
                            "task {} completion channel closed unexpectedly; name: {}, description: {}",
                            task_id, name, description
                        );
                    }
                } else {
                    tracing::error!(
                        "task {} missing completion receiver (already taken?); name: {}, description: {}",
                        task_id, name, description
                    );
                }
                tasks_ref.remove(&task_id);
            });
        }

        tracing::trace!("total tasks: {}", self.tasks.len());
        waiting_finished.join_all().await;

        if let Some(callback) = self.callback.lock().take()
            && let Err(e) = callback.send(())
        {
            tracing::error!("failed to send shutdown completion callback: {:?}", e);
        }
        self.log_all_task();
        Ok(())
    }

    pub fn log_all_task(&self) {
        for task in self.tasks.iter() {
            tracing::info!(
                "task {} name: {}, description: {}",
                task.key(),
                task.value().name,
                task.value().description
            );
        }
    }

    pub fn gen_task_info(tasks: &Tasks, task_id: TaskId) -> anyhow::Result<String> {
        let task = tasks
            .get(&task_id)
            .ok_or_else(|| anyhow::anyhow!("task {} not found", task_id))?;
        Ok(format!(
            "task {} name: {}, description: {}",
            task_id, task.name, task.description
        ))
    }
}

pub struct ShutdownRev {
    cancel_token: CancellationToken,
    completion_sender: Option<oneshot::Sender<()>>,
    manager_handle: Arc<DashMap<TaskId, Task>>,
    task_id: TaskId,
}

impl ShutdownRev {
    pub async fn wait_shutting_down(&mut self) {
        let gen_err_info = || ShutdownSdr::gen_task_info(&self.manager_handle, self.task_id);

        self.cancel_token.cancelled().await;
        tracing::trace!("task {} received shutdown signal", self.task_id);

        // Take and drop the completion sender to signal that this task is done
        if let Some(sender) = self.completion_sender.take() {
            if let Err(e) = sender.send(()) {
                tracing::error!(
                    "task {} failed to send completion: {:?}; details: {:?}",
                    self.task_id,
                    e,
                    gen_err_info()
                );
            }
        } else {
            tracing::error!(
                "task {} completion sender already consumed; details: {:?}",
                self.task_id,
                gen_err_info()
            );
        }

        self.remove_self();
    }

    pub fn new(
        cancel_token: CancellationToken,
        completion_sender: oneshot::Sender<()>,
        manager_handle: Arc<DashMap<TaskId, Task>>,
        task_id: TaskId,
    ) -> Self {
        Self {
            cancel_token,
            completion_sender: Some(completion_sender),
            manager_handle,
            task_id,
        }
    }

    fn remove_self(&self) {
        tracing::trace!("task {} removed", self.task_id);
        self.manager_handle.remove(&self.task_id);
    }
}

impl Drop for ShutdownRev {
    fn drop(&mut self) {
        if self.completion_sender.is_some() {
            self.remove_self();
        }
    }
}

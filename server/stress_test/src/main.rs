mod framework;
mod tests;

use clap::Parser;
use framework::Report;
use tests::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Cleanup resources created during tests
/// This function attempts to clean up even if some operations fail
async fn cleanup_resources(users: &tests::UsersGroup, report: &mut Report) {
    // Attempt to delete sessions, friends, files, then unregister users
    // Note: This is a best-effort cleanup - some resources may already be deleted by tests
    let _ = test_delete_friend(users, report).await;
    let _ = test_unregister(users, report).await;
    for user in users.iter() {
        user.lock().await.async_drop().await;
    }
}

async fn test_endpoint(app: &mut client::ClientCore) -> anyhow::Result<()> {
    let mut report = Report::new();
    tracing::info!("");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("🚀 Starting OurChat Stress Test Suite");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("");

    // Basic service tests
    tracing::info!("📦 Phase 1: Basic Service Tests");
    test_basic_service(&mut report, app).await;
    test_preset_user_status(&mut report, app).await;

    // User registration
    tracing::info!("");
    tracing::info!("👤 Phase 2: User Registration");
    let users = test_register(&mut report, app).await;
    tracing::info!("✅ Registered {} users", users.len());

    // Use a scope to ensure cleanup runs even if tests fail
    let test_result = async {
        // Authentication tests
        tracing::info!("");
        tracing::info!("🔐 Phase 3: Authentication Tests");
        test_auth(&users, &mut report).await;
        test_get_info(&users, &mut report).await;
        test_get_id(&users, &mut report).await;
        test_set_account_info(&users, &mut report).await;

        // File operations
        tracing::info!("");
        tracing::info!("📁 Phase 4: File Operations");
        let keys = test_upload(&users, &mut report).await?;
        test_download(keys.clone(), &users, &mut report).await;

        // Session management tests
        tracing::info!("");
        tracing::info!("💬 Phase 5: Session Management");
        let sessions = test_new_session(&users, &mut report).await?;
        test_get_session_info(sessions.clone(), &users, &mut report).await;
        test_set_session_info(sessions.clone(), &users, &mut report).await;
        test_invite_user_to_session(sessions.clone(), &users, &mut report).await;
        test_leave_session(sessions.clone(), &users, &mut report).await;
        test_delete_session(sessions.clone(), &users, &mut report).await;

        // Session moderation tests
        tracing::info!("");
        tracing::info!("🔨 Phase 6: Session Moderation");
        let sessions = test_new_session(&users, &mut report).await?;
        test_ban(sessions.clone(), &users, &mut report).await;
        test_mute(sessions.clone(), &users, &mut report).await;
        test_kick(sessions.clone(), &users, &mut report).await;

        // Session role tests
        tracing::info!("");
        tracing::info!("👑 Phase 7: Session Roles");
        let role_ids = test_add_role(sessions.clone(), &users, &mut report).await?;
        test_set_role(sessions.clone(), &users, &mut report).await;
        test_get_role(role_ids, &users, &mut report).await;

        // Additional session tests
        tracing::info!("");
        tracing::info!("🔧 Phase 8: Advanced Session Operations");
        let sessions = test_new_session(&users, &mut report).await?;
        test_join_session(sessions.clone(), &users, &mut report).await;
        test_accept_join_session_invitation(sessions.clone(), &users, &mut report).await;
        test_allow_user_join_session(sessions.clone(), &users, &mut report).await;
        test_e2eeize_session(sessions.clone(), &users, &mut report).await;
        test_dee2eeize_session(sessions.clone(), &users, &mut report).await;
        test_send_room_key(sessions, &users, &mut report).await;

        // Friend management tests
        tracing::info!("");
        tracing::info!("🤝 Phase 9: Friend Management");
        test_add_friend(&users, &mut report).await;
        test_accept_friend_invitation(&users, &mut report).await;
        test_set_friend_info(&users, &mut report).await;
        test_delete_friend(&users, &mut report).await;

        // Message tests
        tracing::info!("");
        tracing::info!("✉️  Phase 10: Messaging");
        let sessions = test_new_session(&users, &mut report).await?;
        let msg_ids = test_send_msg(sessions.clone(), &users, &mut report).await?;
        test_fetch_msgs(&users, &mut report).await;
        test_recall(sessions, msg_ids, &users, &mut report).await;

        // WebRTC tests
        tracing::info!("");
        tracing::info!("🎥 Phase 11: WebRTC");
        test_create_room(&users, &mut report).await;

        // Negative tests - error scenarios
        tracing::info!("");
        tracing::info!("⚠️  Phase 12: Negative Tests (Error Scenarios)");
        test_join_nonexistent_session(&users, &mut report).await;
        test_send_msg_invalid_session(&users, &mut report).await;
        test_get_session_info_invalid(&users, &mut report).await;
        test_add_friend_invalid_user(&users, &mut report).await;
        test_send_empty_message(&users, &mut report).await;
        test_delete_session_unauthorized(&users, &mut report).await;

        anyhow::Ok(())
    }
    .await;

    // Always cleanup, even if tests failed
    tracing::info!("");
    tracing::info!("🧹 Cleaning up resources...");
    cleanup_resources(&users, &mut report).await;

    test_result?;

    tracing::info!("");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("✅ Stress Test Suite Completed");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("");

    println!("{report}");
    Ok(())
}

#[derive(Debug, Parser, Default)]
#[command(author = "SkyUOI", about = "The Stress Test of OurChat")]
pub struct ArgsParser {
    #[arg(short, long, help = "The path of server config")]
    pub config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = ArgsParser::parse();
    let mut app = {
        base::log::logger_init(true, None, std::io::stdout, "ourchat");
        let cfg = base::setting::read_config_and_deserialize(&args.config)?;
        client::ClientCore::new(cfg).await?
    };
    // test every endpoint's performance
    test_endpoint(&mut app).await?;
    Ok(())
}

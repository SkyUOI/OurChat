//! Test registry for stress test filtering
//!
//! Provides a centralized way to register and filter tests.

use crate::UsersGroup;
use crate::framework::Report;
use std::fmt;
use std::sync::OnceLock;

/// Type of test based on what dependencies it requires
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    /// Tests that only need the app (no users)
    AppOnly,
    /// Tests that need users registered
    WithUsers,
    /// Tests that need sessions created
    WithSessions,
}

/// Metadata about a registered test
pub struct TestInfo {
    /// Unique test identifier (e.g., "auth", "register", "send_msg")
    pub name: &'static str,
    /// Display name for the test
    pub display_name: &'static str,
    /// Module/category this test belongs to (e.g., "auth", "session", "message")
    pub module: &'static str,
    /// Test type for dependency tracking
    pub test_type: TestType,
}

impl TestInfo {
    /// Create new test info
    pub const fn new(
        name: &'static str,
        display_name: &'static str,
        module: &'static str,
        test_type: TestType,
    ) -> Self {
        Self {
            name,
            display_name,
            module,
            test_type,
        }
    }

    /// Check if this test matches a glob pattern
    ///
    /// Matching rules:
    /// - `"*"` matches everything
    /// - `"xxx*"` matches tests starting with "xxx" (prefix match)
    /// - `"*xxx"` matches tests ending with "xxx" (suffix match)
    /// - `"xxx*yyy"` matches tests starting with "xxx" AND ending with "yyy"
    /// - `"xxx"` (no wildcard) matches tests containing "xxx" (substring match)
    pub fn matches(&self, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        let name_lower = self.name.to_lowercase();
        let module_lower = self.module.to_lowercase();
        let full = format!("{}/{}", module_lower, name_lower);

        // Simple glob matching
        if pattern_lower == "*" {
            return true;
        }

        if pattern_lower.contains('*') {
            let parts: Vec<&str> = pattern_lower.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return name_lower.starts_with(prefix) && name_lower.ends_with(suffix)
                    || module_lower.starts_with(prefix) && module_lower.ends_with(suffix)
                    || full.starts_with(prefix) && full.ends_with(suffix);
            }
        }

        // Without wildcard, do substring matching
        name_lower.contains(&pattern_lower)
            || module_lower.contains(&pattern_lower)
            || full.contains(&pattern_lower)
    }
}

impl fmt::Display for TestInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} ({})", self.module, self.name, self.display_name)
    }
}

/// Global test registry
pub struct TestRegistry {
    tests: Vec<TestInfo>,
}

impl TestRegistry {
    /// Create a new registry
    pub const fn new() -> Self {
        Self { tests: Vec::new() }
    }

    /// Register a test
    pub fn register(&mut self, test: TestInfo) {
        self.tests.push(test);
    }

    /// Filter tests by include/exclude patterns
    pub fn filter(&self, include: &[String], exclude: &[String]) -> Vec<&TestInfo> {
        let mut result: Vec<&TestInfo> = self
            .tests
            .iter()
            .filter(|test| {
                // If no include patterns, include all
                let include_matches =
                    include.is_empty() || include.iter().any(|pattern| test.matches(pattern));

                // Exclude takes precedence
                let exclude_matches = exclude.iter().any(|pattern| test.matches(pattern));

                include_matches && !exclude_matches
            })
            .collect();

        // Sort by module and name for consistent ordering
        result.sort_by(|a, b| a.module.cmp(b.module).then_with(|| a.name.cmp(b.name)));

        result
    }

    /// List all tests as a formatted string
    pub fn list(&self) -> String {
        let mut output = String::from("Available tests:\n\n");

        let mut by_module: std::collections::HashMap<&str, Vec<&TestInfo>> =
            std::collections::HashMap::new();

        for test in &self.tests {
            by_module.entry(test.module).or_default().push(test);
        }

        let mut modules: Vec<&str> = by_module.keys().copied().collect();
        modules.sort();

        for module in modules {
            output.push_str(&format!("  {}:\n", module));
            for test in &by_module[module] {
                output.push_str(&format!("    {} - {}\n", test.name, test.display_name));
            }
        }

        output.push_str("\nFilter examples:\n");
        output.push_str("  --filter 'auth'        Run all auth tests\n");
        output.push_str("  --filter 'auth*'       Run tests starting with 'auth'\n");
        output.push_str("  --filter '*msg*'       Run tests containing 'msg'\n");
        output.push_str("  --exclude 'negative'   Exclude negative tests\n");

        output
    }
}

/// Global registry singleton using OnceLock for thread-safe lazy initialization
static REGISTRY: OnceLock<std::sync::Mutex<TestRegistry>> = OnceLock::new();

/// Get a mutable reference to the global registry
/// Registry is lazily initialized on first access
pub fn registry_mut() -> &'static std::sync::Mutex<TestRegistry> {
    REGISTRY.get_or_init(|| std::sync::Mutex::new(TestRegistry::new()))
}

/// List all available tests
pub fn list_all_tests() -> String {
    let registry = registry_mut();
    let registry = registry.lock().unwrap();
    registry.list()
}

/// Helper to run all filtered tests
pub async fn run_filtered_tests(
    include: &[String],
    exclude: &[String],
    app: &mut client::ClientCore,
) -> anyhow::Result<()> {
    // Get filtered test names and types (not references, to avoid lifetime issues)
    let test_specs = {
        let registry = registry_mut();
        let registry = registry.lock().unwrap();
        let filtered = registry.filter(include, exclude);
        filtered
            .iter()
            .map(|t| (t.name, t.module, t.test_type))
            .collect::<Vec<_>>()
    };

    if test_specs.is_empty() {
        tracing::warn!("No tests match the specified filters");
        return Ok(());
    }

    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("🚀 Running {} test(s)", test_specs.len());
    tracing::info!("═══════════════════════════════════════════════════════════════");

    for (name, module, _ty) in &test_specs {
        tracing::info!("  • {}/{}", module, name);
    }
    tracing::info!("");

    let mut report = Report::new();
    let mut users: UsersGroup = Vec::new();

    // Run tests based on their name - defer to actual test implementations
    // Import all test modules to access their functions
    use crate::tests::{
        auth::{test_auth, test_get_info, test_register, test_set_account_info, test_unregister},
        basic::{test_basic_service, test_get_id, test_preset_user_status},
        file::{test_download, test_upload},
        friend::{
            test_accept_friend_invitation, test_add_friend, test_delete_friend,
            test_set_friend_info,
        },
        message::{test_fetch_msgs, test_recall, test_send_msg},
        negative::{
            test_add_friend_invalid_user, test_delete_session_unauthorized,
            test_get_session_info_invalid, test_join_nonexistent_session, test_send_empty_message,
            test_send_msg_invalid_session,
        },
        session::{
            test_accept_join_session_invitation, test_add_role, test_allow_user_join_session,
            test_ban, test_dee2eeize_session, test_delete_session, test_e2eeize_session,
            test_get_role, test_get_session_info, test_invite_user_to_session, test_join_session,
            test_kick, test_leave_session, test_mute, test_new_session, test_send_room_key,
            test_set_role, test_set_session_info,
        },
        webrtc::test_create_room,
    };

    // App-only tests
    for (name, _module, test_type) in &test_specs {
        if *test_type != TestType::AppOnly {
            continue;
        }
        tracing::info!("▶️  Running: {}", name);
        match *name {
            "timestamp" | "get_server_info" => {
                test_basic_service(&mut report, app).await;
            }
            "preset_user_status" => {
                test_preset_user_status(&mut report, app).await;
            }
            _ => {}
        }
    }

    // Register users if needed
    let needs_users = test_specs
        .iter()
        .any(|(_, _, t)| *t == TestType::WithUsers || *t == TestType::WithSessions);
    if needs_users {
        tracing::info!("");
        tracing::info!("👤 Setting up test users...");
        users = test_register(&mut report, app).await;
        tracing::info!("✅ Registered {} users", users.len());
    }

    // User tests (after registration)
    for (name, _module, test_type) in &test_specs {
        if *test_type != TestType::WithUsers {
            continue;
        }
        if *name == "register" {
            continue; // Already ran
        }
        tracing::info!("▶️  Running: {}", name);
        match *name {
            "auth" => test_auth(&users, &mut report).await,
            "get_info" => test_get_info(&users, &mut report).await,
            "set_account_info" => test_set_account_info(&users, &mut report).await,
            "unregister" => test_unregister(&users, &mut report).await,
            "get_id" => test_get_id(&users, &mut report).await,
            "add_friend" => test_add_friend(&users, &mut report).await,
            "accept_friend_invitation" => test_accept_friend_invitation(&users, &mut report).await,
            "delete_friend" => test_delete_friend(&users, &mut report).await,
            "set_friend_info" => test_set_friend_info(&users, &mut report).await,
            "fetch_msgs" => test_fetch_msgs(&users, &mut report).await,
            "join_nonexistent_session" => test_join_nonexistent_session(&users, &mut report).await,
            "send_msg_invalid_session" => test_send_msg_invalid_session(&users, &mut report).await,
            "get_session_info_invalid" => test_get_session_info_invalid(&users, &mut report).await,
            "add_friend_invalid_user" => test_add_friend_invalid_user(&users, &mut report).await,
            "send_empty_message" => test_send_empty_message(&users, &mut report).await,
            "delete_session_unauthorized" => {
                test_delete_session_unauthorized(&users, &mut report).await
            }
            "create_room" => test_create_room(&users, &mut report).await,
            _ => {}
        }
    }

    // Session tests
    let needs_sessions = test_specs
        .iter()
        .any(|(_, _, t)| *t == TestType::WithSessions);
    let mut role_ids: Option<std::sync::Arc<dashmap::DashMap<base::consts::ID, u64>>> = None;
    let mut keys: Option<std::sync::Arc<dashmap::DashMap<base::consts::OCID, String>>> = None;
    let mut msg_ids: Option<std::sync::Arc<dashmap::DashMap<base::consts::ID, u64>>> = None;
    let sessions: Option<
        std::sync::Arc<dashmap::DashMap<base::consts::ID, base::consts::SessionID>>,
    > = if needs_sessions {
        tracing::info!("");
        tracing::info!("💬 Setting up test sessions...");
        match test_new_session(&users, &mut report).await {
            Ok(s) => {
                tracing::info!("✅ Created {} sessions", s.len());
                Some(s)
            }
            Err(_) => None,
        }
    } else {
        None
    };

    if let Some(sessions) = &sessions {
        for (name, _module, test_type) in &test_specs {
            if *test_type != TestType::WithSessions {
                continue;
            }
            if *name == "new_session" {
                continue; // Already ran for setup
            }
            tracing::info!("▶️  Running: {}", name);
            match *name {
                "get_session_info" => {
                    test_get_session_info(sessions.clone(), &users, &mut report).await
                }
                "set_session_info" => {
                    test_set_session_info(sessions.clone(), &users, &mut report).await
                }
                "invite_user_to_session" => {
                    test_invite_user_to_session(sessions.clone(), &users, &mut report).await
                }
                "leave_session" => test_leave_session(sessions.clone(), &users, &mut report).await,
                "delete_session" => {
                    test_delete_session(sessions.clone(), &users, &mut report).await
                }
                "ban" => test_ban(sessions.clone(), &users, &mut report).await,
                "mute" => test_mute(sessions.clone(), &users, &mut report).await,
                "kick" => test_kick(sessions.clone(), &users, &mut report).await,
                "add_role" => {
                    if let Ok(ids) = test_add_role(sessions.clone(), &users, &mut report).await {
                        role_ids = Some(ids);
                    }
                }
                "set_role" => test_set_role(sessions.clone(), &users, &mut report).await,
                "get_role" => {
                    let ids = role_ids
                        .clone()
                        .unwrap_or_else(|| std::sync::Arc::new(dashmap::DashMap::new()));
                    test_get_role(ids, &users, &mut report).await;
                }
                "join_session" => test_join_session(sessions.clone(), &users, &mut report).await,
                "accept_join_session_invitation" => {
                    test_accept_join_session_invitation(sessions.clone(), &users, &mut report).await
                }
                "allow_user_join_session" => {
                    test_allow_user_join_session(sessions.clone(), &users, &mut report).await
                }
                "e2eeize_session" => {
                    test_e2eeize_session(sessions.clone(), &users, &mut report).await
                }
                "dee2eeize_session" => {
                    test_dee2eeize_session(sessions.clone(), &users, &mut report).await
                }
                "send_room_key" => test_send_room_key(sessions.clone(), &users, &mut report).await,
                "upload" => {
                    if let Ok(k) = test_upload(&users, &mut report).await {
                        keys = Some(k);
                    }
                }
                "download" => {
                    if let Some(k) = &keys {
                        test_download(k.clone(), &users, &mut report).await;
                    }
                }
                "send_msg" => {
                    if let Ok(ids) = test_send_msg(sessions.clone(), &users, &mut report).await {
                        msg_ids = Some(ids);
                    }
                }
                "recall" => {
                    if let Some(ids) = &msg_ids {
                        test_recall(sessions.clone(), ids.clone(), &users, &mut report).await;
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup
    if !users.is_empty() {
        tracing::info!("");
        tracing::info!("🧹 Cleaning up resources...");
        let _ = test_unregister(&users, &mut report).await;
        for user in &users {
            user.lock().await.async_drop().await;
        }
    }

    tracing::info!("");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("✅ Stress Test Suite Completed");
    tracing::info!("═══════════════════════════════════════════════════════════════");
    tracing::info!("");

    println!("{report}");

    Ok(())
}

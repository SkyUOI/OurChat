pub use sea_orm_migration::prelude::*;

// Constants used by other parts of the codebase
pub mod constants;

// Common enum definitions
pub mod enums;

// Predefined roles and permissions
pub mod predefined;

pub mod m20251205_000001_core_tables;
pub mod m20251205_000002_files;
pub mod m20251205_000003_role_permission_system;
pub mod m20251205_000004_server_management;
pub mod m20251205_000005_user_status;
pub mod m20251205_000006_usercontact;
pub mod m20251205_000007_announcements;
pub mod m20251205_000008_webrtc;
pub mod m20251205_000009_oauthverification;
pub mod m20251205_000010_sessioninvitations;
pub mod m20251205_000011_server_management_permissions_extended;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251205_000001_core_tables::Migration),
            Box::new(m20251205_000002_files::Migration),
            Box::new(m20251205_000003_role_permission_system::Migration),
            Box::new(m20251205_000004_server_management::Migration),
            Box::new(m20251205_000005_user_status::Migration),
            Box::new(m20251205_000006_usercontact::Migration),
            Box::new(m20251205_000007_announcements::Migration),
            Box::new(m20251205_000008_webrtc::Migration),
            Box::new(m20251205_000009_oauthverification::Migration),
            Box::new(m20251205_000010_sessioninvitations::Migration),
            Box::new(m20251205_000011_server_management_permissions_extended::Migration),
        ]
    }
}

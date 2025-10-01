pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_table;
pub mod m20240812_083747_server_manager;
pub mod m20240829_010832_files;
mod m20241210_081859_add_cryption_for_msg;
mod m20241210_083126_create_index;
pub mod m20241229_022701_add_role_for_session;
mod m20241229_035143_add_data_for_session;
mod m20241230_092258_add_status_for_user;
mod m20250102_091815_add_attr_for_session;
mod m20250107_153037_record_who_created_role;
mod m20250206_160318_add_user_contact_info;
pub mod m20250218_093632_server_manage_permission;
pub mod m20250301_005919_add_soft_delete_columns;
mod m20250315_073350_delete_status_for_users;
mod m20250316_015417_add_preset_user_status;
mod m20250329_120341_add_announcement_table;
mod m20250329_153038_add_permission_link;
mod m20250403_113515_alter_announcement_id;
mod m20250404_114543_rename_user_chat_msg_to_message_records;
mod m20250404_135851_add_is_all_user_in_message_records;
mod m20250404_153258_add_announcement_message_table;
mod m20250405_081542_add_name_for_server_manager_role;
mod m20250524_082226_webrtc_room;
mod m20250531_080259_add_e2ee_on_in_session;
mod m20250607_110239_add_room_key_time_and_leaving_to_process_in_session;
mod m20250607_144007_add_public_key_in_user;
mod m20250609_074125_change_sender_id_to_be_optional_in_message_records;
mod m20250710_090847_add_session_id_to_friend_relation;
mod m20250712_025238_add_e2eeize_and_dee2eeize_session_permission;
mod m20250714_000446_session_invitation;
mod m20250924_053530_webrtc_room;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240812_083747_server_manager::Migration),
            Box::new(m20240829_010832_files::Migration),
            Box::new(m20241210_081859_add_cryption_for_msg::Migration),
            Box::new(m20241210_083126_create_index::Migration),
            Box::new(m20241229_022701_add_role_for_session::Migration),
            Box::new(m20241229_035143_add_data_for_session::Migration),
            Box::new(m20241230_092258_add_status_for_user::Migration),
            Box::new(m20250102_091815_add_attr_for_session::Migration),
            Box::new(m20250107_153037_record_who_created_role::Migration),
            Box::new(m20250206_160318_add_user_contact_info::Migration),
            Box::new(m20250218_093632_server_manage_permission::Migration),
            Box::new(m20250301_005919_add_soft_delete_columns::Migration),
            Box::new(m20250315_073350_delete_status_for_users::Migration),
            Box::new(m20250316_015417_add_preset_user_status::Migration),
            Box::new(m20250329_120341_add_announcement_table::Migration),
            Box::new(m20250329_153038_add_permission_link::Migration),
            Box::new(m20250403_113515_alter_announcement_id::Migration),
            Box::new(m20250404_114543_rename_user_chat_msg_to_message_records::Migration),
            Box::new(m20250404_135851_add_is_all_user_in_message_records::Migration),
            Box::new(m20250404_153258_add_announcement_message_table::Migration),
            Box::new(m20250405_081542_add_name_for_server_manager_role::Migration),
            Box::new(m20250524_082226_webrtc_room::Migration),
            Box::new(m20250531_080259_add_e2ee_on_in_session::Migration),
            Box::new(
                m20250607_110239_add_room_key_time_and_leaving_to_process_in_session::Migration,
            ),
            Box::new(m20250607_144007_add_public_key_in_user::Migration),
            Box::new(
                m20250609_074125_change_sender_id_to_be_optional_in_message_records::Migration,
            ),
            Box::new(m20250710_090847_add_session_id_to_friend_relation::Migration),
            Box::new(m20250712_025238_add_e2eeize_and_dee2eeize_session_permission::Migration),
            Box::new(m20250714_000446_session_invitation::Migration),
            Box::new(m20250924_053530_webrtc_room::Migration),
        ]
    }
}

pub mod delete_account {
    pub mod v1 {
        include!("../generated/service.server_manage.delete_account.v1.rs");
    }
}

pub mod set_server_status {
    pub mod v1 {
        include!("../generated/service.server_manage.set_server_status.v1.rs");
    }
}

pub mod publish_announcement {
    pub mod v1 {
        include!("../generated/service.server_manage.publish_announcement.v1.rs");
    }
}

pub mod v1 {
    include!("../generated/service.server_manage.v1.rs");
}

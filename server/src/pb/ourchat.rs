pub mod get_account_info;
pub mod msg_delivery;
pub mod upload;

pub mod v1 {
    tonic::include_proto!("service.ourchat.v1");
}

pub mod download {
    pub mod v1 {
        tonic::include_proto!("service.ourchat.download.v1");
    }
}

pub mod session {
    pub mod v1 {
        tonic::include_proto!("service.ourchat.session.v1");
    }
}

pub mod set_account_info {
    pub mod v1 {
        tonic::include_proto!("service.ourchat.set_account_info.v1");
    }
}

pub mod unregister {
    pub mod v1 {
        tonic::include_proto!("service.ourchat.unregister.v1");
    }
}

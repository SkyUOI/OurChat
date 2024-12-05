pub mod v1 {
    tonic::include_proto!("service.registry.v1");
}

pub mod transmit_msg {
    pub mod v1 {
        tonic::include_proto!("service.registry.transmit_msg.v1");
    }
}

pub mod chat_server {
    pub mod v1 {
        tonic::include_proto!("service.registry.chat_server.v1");
    }
}

pub mod server;

pub mod v1 {
    tonic::include_proto!("service.basic.v1");
}

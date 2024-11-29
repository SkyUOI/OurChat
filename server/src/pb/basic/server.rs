pub mod v1 {
    tonic::include_proto!("service.basic.server.v1");

    impl From<bool> for RunningStatus {
        fn from(value: bool) -> Self {
            if value {
                RunningStatus::Maintaining
            } else {
                RunningStatus::Normal
            }
        }
    }
}

pub mod v1 {
    include!("../generated/service.basic.server.v1.rs");

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

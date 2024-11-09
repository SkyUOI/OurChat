tonic::include_proto!("server");

impl From<bool> for RunningStatus {
    fn from(value: bool) -> Self {
        if value {
            RunningStatus::Maintaining
        } else {
            RunningStatus::Normal
        }
    }
}

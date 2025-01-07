pub mod v1 {
    use std::sync::LazyLock;

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

    pub static VERSION_SPLIT: LazyLock<ServerVersion> = LazyLock::new(|| {
        let ver = base::build::PKG_VERSION.split('.').collect::<Vec<_>>();
        ServerVersion {
            major: ver[0].parse().unwrap(),
            minor: ver[1].parse().unwrap(),
            patch: ver[2].parse().unwrap(),
        }
    });
}

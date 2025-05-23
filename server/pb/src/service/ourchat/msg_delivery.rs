pub mod v1 {
    include!("../../generated/service.ourchat.msg_delivery.v1.rs");

    pub type BundleMsgs = Vec<OneMsg>;
}

pub mod recall {
    pub mod v1 {
        include!("../../generated/service.ourchat.msg_delivery.recall.v1.rs");
    }
}

pub mod announcement {
    pub mod v1 {
        use entities::announcement;

        use crate::time::to_google_timestamp;

        include!("../../generated/service.ourchat.msg_delivery.announcement.v1.rs");

        impl From<announcement::Model> for AnnouncementResponse {
            fn from(value: announcement::Model) -> Self {
                Self {
                    announcement: Some(Announcement {
                        content: value.content,
                        title: value.title,
                        publisher_id: value.publisher_id as u64,
                    }),
                    created_at: Some(to_google_timestamp(value.created_at.to_utc())),
                    id: value.id as u64,
                }
            }
        }
    }
}

pub mod v1 {
    use base::time::to_google_timestamp;
    use entities::user_chat_msg;
    tonic::include_proto!("service.ourchat.msg_delivery.v1");

    pub type BundleMsgs = Vec<OneMsg>;

    impl TryFrom<user_chat_msg::Model> for Msg {
        type Error = anyhow::Error;

        fn try_from(msg: user_chat_msg::Model) -> Result<Self, anyhow::Error> {
            Ok(Self {
                msg_id: msg.chat_msg_id.try_into()?,
                bundle_msgs: serde_json::from_value(msg.msg_data)?,
                session_id: msg.session_id.try_into()?,
                time: Some(to_google_timestamp(msg.time.into())),
                sender_id: msg.sender_id.try_into()?,
                is_encrypted: msg.is_encrypted,
            })
        }
    }
}

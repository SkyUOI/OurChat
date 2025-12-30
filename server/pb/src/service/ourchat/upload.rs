pub mod v1 {
    use prost::bytes;
    use upload_request::Data;

    include!("../../generated/service.ourchat.upload.v1.rs");

    impl UploadRequest {
        pub fn header(self) -> Option<Header> {
            match self.data? {
                Data::Metadata(data) => Some(data),
                _ => None,
            }
        }

        pub fn new_header(
            size: usize,
            hash: bytes::Bytes,
            auto_clean: bool,
            session_id: Option<u64>,
        ) -> Self {
            Self {
                data: Some(Data::Metadata(Header {
                    hash,
                    size: size as u64,
                    auto_clean,
                    session_id,
                })),
            }
        }

        pub fn new_content(binary_data: bytes::Bytes) -> Self {
            Self {
                data: Some(Data::Content(binary_data)),
            }
        }
    }
}

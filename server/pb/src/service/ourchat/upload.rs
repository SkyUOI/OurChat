pub mod v1 {
    use std::fmt;

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

    impl fmt::Debug for UploadChunkRequest {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("UploadChunkRequest")
                .field("upload_id", &self.upload_id)
                .field("chunk_id", &self.chunk_id)
                .finish()
        }
    }
}

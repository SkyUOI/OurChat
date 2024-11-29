pub mod v1 {
    use upload_request::Data;

    tonic::include_proto!("service.ourchat.upload.v1");

    impl UploadRequest {
        pub fn header(self) -> Option<Header> {
            match self.data? {
                upload_request::Data::Metadata(data) => Some(data),
                _ => None,
            }
        }

        pub fn new_header(size: usize, hash: String, auto_clean: bool) -> Self {
            Self {
                data: Some(Data::Metadata(Header {
                    hash,
                    size: size as u64,
                    auto_clean,
                })),
            }
        }

        pub fn new_content(binary_data: Vec<u8>) -> Self {
            Self {
                data: Some(Data::Content(binary_data)),
            }
        }
    }
}

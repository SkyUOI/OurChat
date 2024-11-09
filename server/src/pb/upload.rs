tonic::include_proto!("upload");

impl UploadRequest {
    pub fn header(self) -> Option<Header> {
        match self.data? {
            upload_request::Data::Metadata(data) => Some(data),
            _ => None,
        }
    }
}

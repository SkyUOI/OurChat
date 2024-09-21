//! For Data-Driven Development

pub struct Email(String);

impl Email {
    pub fn new(email: String) -> anyhow::Result<Self> {
        Ok(Self(email))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

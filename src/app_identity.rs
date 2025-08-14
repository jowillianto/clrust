use crate::app_version::AppVersion;

#[derive(Clone, Debug, Default)]
pub struct AppIdentity {
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub version: AppVersion,
}

impl AppIdentity {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        version: AppVersion,
    ) -> Self {
        return Self {
            name: name.into(),
            description: description.into(),
            author: None,
            license: None,
            version: version,
        };
    }
    pub fn written_by(&mut self, author: impl Into<String>) -> &mut Self {
        self.author = Some(author.into());
        return self;
    }
    pub fn licensed_with(&mut self, license: impl Into<String>) -> &mut Self {
        self.license = Some(license.into());
        return self;
    }
    pub fn take(&mut self) -> Self {
        return std::mem::take(self);
    }
}

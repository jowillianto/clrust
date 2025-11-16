use std::fmt;

use crate::AppVersion;

#[derive(Debug, Clone)]
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
        Self {
            name: name.into(),
            description: description.into(),
            author: None,
            license: None,
            version,
        }
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }
}

impl fmt::Display for AppIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} v{}", self.name, self.version)?;
        if !self.description.is_empty() {
            writeln!(f, "{}", self.description)?;
        }
        if let Some(author) = &self.author {
            writeln!(f, "Written by : {}", author)?;
        }
        if let Some(license) = &self.license {
            writeln!(f, "{}", license)?;
        }
        Ok(())
    }
}

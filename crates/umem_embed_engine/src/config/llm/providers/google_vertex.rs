use crate::HashMap;
use anyhow::{Result, bail};

pub struct GoogleVertexAIConfig {
    pub project: String,
    pub location: String,
    pub headers: HashMap<String, String>,
    pub credentials: GoogleCredentials,
}

pub struct GoogleCredentials {
    client_email: String,
    private_key: String,
    private_key_id: Option<String>,
}

pub struct GoogleVertexAIConfigBuilder {
    pub project: Option<String>,
    pub location: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub credentials: Option<GoogleCredentials>,
}

impl GoogleVertexAIConfigBuilder {
    pub fn new() -> Self {
        GoogleVertexAIConfigBuilder {
            project: None,
            location: None,
            headers: None,
            credentials: None,
        }
    }
    pub fn project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }
    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }
    pub fn credentials(mut self, credentials: GoogleCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }
    pub fn build(self) -> Result<GoogleVertexAIConfig> {
        if self.project.is_none() || self.location.is_none() || self.credentials.is_none() {
            bail!("project, location, and credentials are required");
        }

        Ok(GoogleVertexAIConfig {
            project: self.project.unwrap(),
            location: self.location.unwrap(),
            headers: self.headers.unwrap_or_default(),
            credentials: self.credentials.unwrap(),
        })
    }
}

use anyhow::{Result, bail};

pub struct AmazonBedrockConfig {
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub session_token: Option<String>,
}

pub struct AmazonBedrockConfigBuilder {
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub session_token: Option<String>,
}
impl AmazonBedrockConfigBuilder {
    pub fn new() -> Self {
        AmazonBedrockConfigBuilder {
            region: None,
            access_key: None,
            secret_key: None,
            session_token: None,
        }
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn access_key(mut self, access_key: String) -> Self {
        self.access_key = Some(access_key);
        self
    }

    pub fn secret_key(mut self, secret_key: String) -> Self {
        self.secret_key = Some(secret_key);
        self
    }

    pub fn session_token(mut self, session_token: String) -> Self {
        self.session_token = Some(session_token);
        self
    }

    pub fn build(self) -> Result<AmazonBedrockConfig> {
        if self.region.is_none() || self.access_key.is_none() || self.secret_key.is_none() {
            bail!("region, access_key, and secret_key are required");
        }

        Ok(AmazonBedrockConfig {
            region: self.region.unwrap(),
            access_key: self.access_key.unwrap(),
            secret_key: self.secret_key.unwrap(),
            session_token: self.session_token,
        })
    }
}

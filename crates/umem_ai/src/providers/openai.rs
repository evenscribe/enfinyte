use anyhow::{bail, Result};
use async_trait::async_trait;
use base64::Engine;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use crate::{
    reqwest_client,
    response_generators::{
        messages::{FilePart, Message, UserMessagePart, UserModelMessage},
        GenerateTextRequest, GenerateTextResponse,
    },
    GeneratesText,
};

pub struct OpenAIProvider {
    pub api_key: String,
    pub base_url: String,
    pub default_headers: Vec<(String, String)>,
    pub organization: Option<String>,
    pub project: Option<String>,
}

#[async_trait]
impl GeneratesText for OpenAIProvider {
    async fn generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, reqwest::Error> {
        let system = request
            .messages
            .iter()
            .find_map(|msg| match msg {
                Message::System(v) => Some(v.as_str()),
                _ => None,
            })
            .unwrap();

        let user_messages: Vec<&UserModelMessage> = request
            .messages
            .iter()
            .filter_map(|msg| match msg {
                Message::User(v) => Some(v),
                _ => None,
            })
            .collect();

        let input: Vec<serde_json::Value> = user_messages
            .iter()
            .flat_map(|um| match um {
                UserModelMessage::Text(input_text) => {
                    vec![serde_json::json!({"type":"input_text","text":input_text})]
                }
                UserModelMessage::Parts(user_message_parts) => user_message_parts
                    .iter()
                    .map(|ump| match ump {
                        UserMessagePart::Text(input_text) => {
                            serde_json::json!({"type":"input_text","text":input_text})
                        }
                        UserMessagePart::Image(image_part) => match image_part {
                            FilePart::Url(ref image_url, _) => {
                                serde_json::json!({"type":"input_image","image_url":image_url})
                            }
                            FilePart::Base64(ref b64,ref media_type) => {
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_image","image_url":format!("data:{}/;base64,{}", media_type.to_string(), b64)})
                            }
                            FilePart::Buffer(buf,ref media_type) => {
                                let buf_as_b64 = base64::engine::general_purpose::STANDARD.encode(buf);
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_image","image_url":format!("data:{}/;base64,{}", media_type.to_string(), buf_as_b64)})
                            },
                        },
                        UserMessagePart::File(file_part) => match file_part{
                            FilePart::Url(ref image_url, _) => {
                                serde_json::json!({"type":"input_file","file_url":image_url})
                            }
                            FilePart::Base64(ref b64,ref media_type) => {
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_file","file_url":format!("data:{}/;base64,{}", media_type.to_string(), b64)})
                            }
                            FilePart::Buffer(buf,ref media_type) => {
                                let buf_as_b64 = base64::engine::general_purpose::STANDARD.encode(buf);
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_file","file_url":format!("data:{}/;base64,{}", media_type.to_string(), buf_as_b64)})
                            },
                        }
                    })
                    .collect(),
            })
            .collect();

        let request_body = serde_json::json!({
            "model": request.model,
            "instructions":system,
            "input": input,
            "max_output_tokens": request.max_output_tokens.unwrap_or(8192),
            "temperature": request.temperature.unwrap_or(1.0),
            "top_p": request.top_p.unwrap_or(1.0),
        })
        .to_string();

        let response = reqwest_client
            .post(format!("{}/responses", self.base_url))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
            .body(request_body)
            .send()
            .await?;

        unimplemented!()
    }
}

pub struct OpenAIProviderBuilder {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_headers: Option<Vec<(String, String)>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}

impl OpenAIProviderBuilder {
    pub fn new() -> Self {
        OpenAIProviderBuilder {
            api_key: None,
            base_url: None,
            default_headers: None,
            organization: None,
            project: None,
        }
    }

    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn default_headers(mut self, default_headers: Vec<(String, String)>) -> Self {
        self.default_headers = Some(default_headers);
        self
    }

    pub fn organization(mut self, organization: String) -> Self {
        self.organization = Some(organization);
        self
    }

    pub fn project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }

    pub fn build(self) -> Result<OpenAIProvider> {
        if self.api_key.is_none() {
            bail!("api_key is required");
        }

        Ok(OpenAIProvider {
            api_key: self.api_key.unwrap(),
            base_url: self
                .base_url
                .unwrap_or("https://api.openai.com/v1".to_string()),
            default_headers: self.default_headers.unwrap_or_default(),
            organization: self.organization,
            project: self.project,
        })
    }
}

use crate::{
    reqwest_client,
    response_generators::{
        messages::{FilePart, Message, UserMessagePart, UserModelMessage},
        GenerateTextError, GenerateTextRequest, GenerateTextResponse,
    },
    GeneratesText,
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    ) -> Result<GenerateTextResponse, GenerateTextError> {
        let system = request
            .messages
            .iter()
            .find_map(|msg| match msg {
                Message::System(v) => Some(v.as_str()),
                _ => None,
            })
            .unwrap_or("");

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
                                serde_json::json!({"type":"input_image","image_url":format!("data:{};base64,{}", media_type.to_string(), b64)})
                            }
                            FilePart::Buffer(buf,ref media_type) => {
                                let buf_as_b64 = base64::engine::general_purpose::STANDARD.encode(buf);
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_image","image_url":format!("data:{};base64,{}", media_type.to_string(), buf_as_b64)})
                            },
                        },
                        UserMessagePart::File(file_part) => match file_part{
                            FilePart::Url(ref image_url, _) => {
                                serde_json::json!({"type":"input_file","file_url":image_url})
                            }
                            FilePart::Base64(ref b64,ref media_type) => {
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_file","file_url":format!("data:{};base64,{}", media_type.to_string(), b64)})
                            }
                            FilePart::Buffer(buf,ref media_type) => {
                                let buf_as_b64 = base64::engine::general_purpose::STANDARD.encode(buf);
                                let media_type = media_type.clone().unwrap_or(mime::IMAGE_PNG);
                                serde_json::json!({"type":"input_file","file_url":format!("data:{};base64,{}", media_type.to_string(), buf_as_b64)})
                            },
                        }
                    })
                    .collect(),
            })
            .collect();

        let request_body = serde_json::json!({
            "model": request.model,
            "instructions":system,
            "input": [serde_json::json!({
                "type": "message",
                "role": "user",
                "content": input,
            })],
            "max_output_tokens": request.max_output_tokens.unwrap_or(8192),
            "temperature": request.temperature.unwrap_or(1.0),
            "top_p": request.top_p.unwrap_or(1.0),
            "reasoning" : serde_json::json!({
                "effort": "low"
            })
        })
        .to_string();

        let response = reqwest_client
            .post(format!("{}/responses", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .body(request_body)
            .send()
            .await?
            .error_for_status()?
            .json::<ApiResponse>()
            .await?;

        let output_text = response
            .output
            .iter()
            .find_map(|item| match item {
                OutputItem::Message { content, .. } => {
                    let texts: Vec<String> = content
                        .iter()
                        .filter_map(|mc| match mc {
                            MessageContent::OutputText { text, .. } => Some(text.clone()),
                            _ => None,
                        })
                        .collect();
                    Some(texts.join("\n"))
                }
                _ => None,
            })
            .unwrap_or_default();

        return Ok(GenerateTextResponse { text: output_text });
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    pub output: Vec<OutputItem>,
    #[serde(flatten)]
    pub response_metadata: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum OutputItem {
    Message {
        id: String,
        role: String,
        content: Vec<MessageContent>,
        status: Option<String>,
    },
    FunctionCall {
        id: String,
        call_id: String,
        name: String,
        arguments: String,
        status: Option<String>,
    },
    FileSearchCall {
        id: String,
        status: Option<String>,
        queries: Vec<String>,
        results: Option<Vec<Value>>,
    },
    WebSearchCall {
        id: String,
        status: Option<String>,
        action: Value,
    },
    ComputerCall {
        id: String,
        call_id: String,
        action: Value,
        status: Option<String>,
    },
    Reasoning {
        id: String,
        encrypted_content: Option<String>,
        summary: Option<Vec<Value>>,
        content: Option<Vec<Value>>,
        status: Option<String>,
    },
    CodeInterpreterCall {
        id: String,
        status: Option<String>,
        code: Option<String>,
        outputs: Option<Vec<Value>>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageContent {
    OutputText {
        text: String,
        annotations: Option<Vec<Value>>,
    },
    Refusal {
        refusal: String,
    },
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

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        response_generators::{generate_text, GenerateTextRequestBuilder},
        LLMProvider,
    };
    use std::sync::Arc;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_generate_text() -> () {
        let provider = Arc::new(LLMProvider::from(
            OpenAIProviderBuilder::new()
                .api_key(
                    "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",
                )
                .base_url("https://openrouter.ai/api/v1")
                .build()
                .unwrap(),
        ));

        let request = GenerateTextRequestBuilder::new()
            .model("arcee-ai/trinity-mini:free".to_string())
            .system("You are a helpful assistant.".to_string())
            .prompt("Invent a new holiday and describe its traditions.".to_string())
            .provider(Arc::clone(&provider))
            .max_output_tokens(10000)
            .temperature(0.7)
            .build()
            .unwrap();

        let generate_text_response = generate_text(request).await.unwrap();

        dbg!("generate_text_response: {:#?}", &generate_text_response);
    }
}

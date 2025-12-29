use crate::{
    messages::{FilePart, UserModelMessage},
    response_generators::{
        self, GenerateTextRequest, GenerateTextResponse, ResponseGeneratorError,
    },
    utils, GenerateObjectRequest, GenerateObjectResponse, GeneratesObject, GeneratesText,
    OpenAIProvider,
};
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_bedrockruntime::{
    operation::converse::builders::ConverseFluentBuilder,
    types::{ContentBlock, ImageBlock, Message},
};
use reqwest::header::HeaderMap;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use std::env;
use thiserror::Error;

pub struct AmazonBedrockProvider {
    client: aws_sdk_bedrockruntime::Client,
    default_headers: HeaderMap,
}

#[async_trait]
impl GeneratesText for AmazonBedrockProvider {
    async fn generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError> {
        let converse_request = self
            .normalize_generate_text_request(&request)
            .map_err(ResponseGeneratorError::Transient)?;

        let converse_response = converse_request
            .send()
            .await
            .map_err(|e| ResponseGeneratorError::BedrockConverseError(e.to_string()))?;

        let converse_output = match converse_response.output {
            Some(output) => output,
            None => {
                return Err(ResponseGeneratorError::EmptyProviderResponse);
            }
        };

        let output_message = converse_output.as_message().map_err(|_| {
            ResponseGeneratorError::InvalidProviderResponse(
                "was expecting the output to be a bedrock message".into(),
            )
        })?;

        let output_text = output_message
            .content
            .first()
            .ok_or(ResponseGeneratorError::EmptyProviderResponse)?
            .as_text()
            .map_err(|_| {
                ResponseGeneratorError::InvalidProviderResponse(
                    "was expecting the output message content to be text".into(),
                )
            })?;

        Ok(GenerateTextResponse {
            text: output_text.to_string(),
        })
    }
}

#[async_trait]
impl GeneratesObject for AmazonBedrockProvider {
    async fn generate_object<T: Clone + JsonSchema + Serialize + Send + Sync + DeserializeOwned>(
        &self,
        request: GenerateObjectRequest<T>,
    ) -> Result<GenerateObjectResponse<T>, ResponseGeneratorError> {
        let converse_request = self
            .normalizer_generate_object_request(&request)
            .map_err(ResponseGeneratorError::Transient)?;

        let converse_response = converse_request
            .send()
            .await
            .map_err(|e| ResponseGeneratorError::BedrockConverseError(e.to_string()))?;

        let converse_output = match converse_response.output {
            Some(output) => output,
            None => {
                return Err(ResponseGeneratorError::EmptyProviderResponse);
            }
        };

        let output_message = converse_output.as_message().map_err(|_| {
            ResponseGeneratorError::InvalidProviderResponse(
                "was expecting the output to be a bedrock message".into(),
            )
        })?;

        let output_text = output_message
            .content
            .first()
            .ok_or(ResponseGeneratorError::EmptyProviderResponse)?
            .as_text()
            .map_err(|_| {
                ResponseGeneratorError::InvalidProviderResponse(
                    "was expecting the output message content to be text".into(),
                )
            })?;

        serde_json::from_str::<T>(output_text)
            .map(|output| GenerateObjectResponse { output })
            .map_err(ResponseGeneratorError::Deserialization)
    }
}

impl AmazonBedrockProvider {
    fn normalize_generate_object_request<
        T: Clone + JsonSchema + Serialize + Send + Sync + DeserializeOwned,
    >(
        &self,
        request: &GenerateObjectRequest<T>,
    ) -> anyhow::Result<ConverseFluentBuilder> {
        let mut system = OpenAIProvider::normalize_system_message(&request.messages);
        system.push_str(
            r#"
            **Critical Instruction**: ALWAYS and ONLY respond with a JSON object that conforms EXACTLY to the provided JSON Schema (enclosed in `<OutputSchema>`).
        "#,
        );
        let mut user_messages = Self::normalize_user_messages(&request.messages).unwrap();
        user_messages.push(ContentBlock::Text(format!(
            "<OutputSchema>\n{}\n</OutputSchema>",
            serde_json::to_string_pretty(&request.output_schema,)?
        )));

        Ok(self
            .client
            .converse()
            .model_id(request.model.model_name.clone())
            .system(aws_sdk_bedrockruntime::types::SystemContentBlock::Text(
                system,
            ))
            .messages(
                Message::builder()
                    .role("user".into())
                    .set_content(Some(user_messages))
                    .build()
                    .unwrap(),
            ))
    }

    fn normalize_generate_text_request(
        &self,
        request: &GenerateTextRequest,
    ) -> anyhow::Result<ConverseFluentBuilder> {
        let system = OpenAIProvider::normalize_system_message(&request.messages);
        let user_messages = Self::normalize_user_messages(&request.messages)?;

        Ok(self
            .client
            .converse()
            .model_id(request.model.model_name.clone())
            .system(aws_sdk_bedrockruntime::types::SystemContentBlock::Text(
                system,
            ))
            .messages(
                Message::builder()
                    .role("user".into())
                    .set_content(Some(user_messages))
                    .build()
                    .unwrap(),
            ))
    }

    fn normalize_user_messages(
        messages: &[response_generators::messages::Message],
    ) -> anyhow::Result<Vec<ContentBlock>> {
        let user_messages: Vec<&response_generators::messages::UserModelMessage> = messages
            .iter()
            .filter_map(|msg| match msg {
                response_generators::messages::Message::User(v) => Some(v),
                _ => None,
            })
            .collect();

        let user_message_content_blocks: Vec<ContentBlock> = user_messages
            .iter()
            .flat_map(|um| match um {
                UserModelMessage::Text(text) => vec![ContentBlock::Text(text.into())],
                UserModelMessage::Parts(user_message_parts) => user_message_parts
                    .iter()
                    .map(|part| match part {
                        crate::messages::UserMessagePart::Text(text) => {
                            ContentBlock::Text(text.into())
                        }
                        crate::messages::UserMessagePart::Image(image_part) => {
                            let image_block = match image_part {
                                FilePart::Url(_, _) => {
                                    unimplemented!("AWS doesn't support URL images yet");
                                }
                                FilePart::Base64(b64_string, media_type) => ImageBlock::builder()
                                    .source(aws_sdk_bedrockruntime::types::ImageSource::Bytes(
                                        b64_string.as_bytes().into(),
                                    ))
                                    .format(
                                        media_type
                                            .clone()
                                            .unwrap_or(mime::IMAGE_PNG)
                                            .to_string()
                                            .as_str()
                                            .into(),
                                    )
                                    .build()
                                    .unwrap(),
                                FilePart::Buffer(items, media_type) => ImageBlock::builder()
                                    .source(aws_sdk_bedrockruntime::types::ImageSource::Bytes(
                                        items.as_slice().into(),
                                    ))
                                    .format(
                                        media_type
                                            .clone()
                                            .unwrap_or(mime::IMAGE_PNG)
                                            .to_string()
                                            .as_str()
                                            .into(),
                                    )
                                    .build()
                                    .unwrap(),
                            };
                            ContentBlock::Image(image_block)
                        }
                        crate::messages::UserMessagePart::File(file_part) => todo!(),
                    })
                    .collect(),
            })
            .collect();

        Ok(user_message_content_blocks)
    }
}

#[derive(Default)]
pub struct AmazonBedrockProviderBuilder {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    region: Option<Region>,
    default_headers: Vec<(String, String)>,
}

#[derive(Error, Debug)]
pub enum AmazonBedrockProviderBuilderError {
    #[error("Missing AWS Access Key ID")]
    MissingAccessKeyId,
    #[error("Missing AWS Secret Access Key")]
    MissingSecretAccessKey,
    #[error("Missing AWS Bedrock Region")]
    MissingRegion,
    #[error(transparent)]
    BadHeaders(#[from] utils::BuildHeaderMapError),
}

impl AmazonBedrockProviderBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn access_key_id(mut self, access_key_id: impl Into<String>) -> Self {
        self.access_key_id = Some(access_key_id.into());
        self
    }

    pub fn secret_access_key(mut self, secret_access_key: impl Into<String>) -> Self {
        self.secret_access_key = Some(secret_access_key.into());
        self
    }

    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(Region::new(region.into()));
        self
    }

    pub fn default_headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.default_headers = headers;
        self
    }

    pub async fn build(self) -> Result<AmazonBedrockProvider, AmazonBedrockProviderBuilderError> {
        if self.access_key_id.is_none() {
            return Err(AmazonBedrockProviderBuilderError::MissingAccessKeyId);
        }

        if self.secret_access_key.is_none() {
            return Err(AmazonBedrockProviderBuilderError::MissingSecretAccessKey);
        }

        if self.region.is_none() {
            return Err(AmazonBedrockProviderBuilderError::MissingRegion);
        }

        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", self.access_key_id.clone().unwrap());
            env::set_var(
                "AWS_SECRET_ACCESS_KEY",
                self.secret_access_key.clone().unwrap(),
            );
        }

        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(self.region)
            .load()
            .await;

        let default_headers = utils::build_header_map(self.default_headers.as_slice())?;

        Ok(AmazonBedrockProvider {
            client: aws_sdk_bedrockruntime::Client::new(&sdk_config),
            default_headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        generate_object, generate_text, AIProvider, GenerateObjectRequestBuilder,
        GenerateTextRequestBuilder, LanguageModel,
    };
    use serde::Deserialize;
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn test_bedrock_generate_object() {
        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("REGION")
                .access_key_id("ACESS_KEY_ID")
                .secret_access_key("SECRET_ACCESS_KEY")
                .build()
                .await
                .unwrap(),
        ));

        #[derive(Clone, JsonSchema, Serialize, Deserialize, Debug)]
        struct Holiday {
            name: String,
            traditions: String,
        }

        let model = Arc::new(LanguageModel {
            provider,
            model_name: "deepseek.v3-v1:0".to_string(),
        });

        let request = GenerateObjectRequestBuilder::<Holiday>::new()
            .model(model)
            .system("You are a helpful assistant.".to_string())
            .prompt("Invent a new holiday and describe its traditions.".to_string())
            .max_output_tokens(2000)
            .temperature(0.7)
            .build()
            .unwrap();

        let generate_object_response = generate_object(request).await.unwrap();

        dbg!(&generate_object_response);
    }

    #[tokio::test]
    async fn test_bedrock_generate_text() {
        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("REGION")
                .access_key_id("ACESS_KEY_ID")
                .secret_access_key("SECRET_ACCESS_KEY")
                .build()
                .await
                .unwrap(),
        ));

        let model = Arc::new(LanguageModel {
            provider,
            model_name: "deepseek.v3-v1:0".to_string(),
        });

        let request = GenerateTextRequestBuilder::new()
            .model(model)
            .system("You are a helpful assistant.")
            .prompt("Invent a new holiday and describe its traditions.")
            .max_output_tokens(2000)
            .temperature(0.7)
            .build()
            .unwrap();

        let generate_text_response = generate_text(request).await.unwrap();

        dbg!(&generate_text_response);
    }
}

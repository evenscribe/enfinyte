use crate::{
    Embeds, GenerateObjectRequest, GenerateObjectResponse, GeneratesObject, GeneratesText,
    OpenAIProvider, Ranking, RerankRequest, RerankResponse, Reranks, ReranksStructuredData,
    SerializationMode, StructuredRanking, StructuredRerankRequest, StructuredRerankResponse,
    embed::{EmbeddingRequest, EmbeddingResponse},
    messages::{FilePart, UserModelMessage},
    response_generators::{
        self, GenerateTextRequest, GenerateTextResponse, ResponseGeneratorError,
    },
    utils,
};
use anyhow::Result;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_bedrockagentruntime::types::{
    BedrockRerankingConfiguration, BedrockRerankingModelConfiguration, RerankDocument,
    RerankDocumentType, RerankQuery, RerankQueryContentType, RerankSource, RerankSourceType,
    RerankTextDocument, RerankingConfiguration, RerankingConfigurationType,
};
use aws_sdk_bedrockruntime::{
    error::{BuildError, ProvideErrorMetadata},
    operation::{converse::builders::ConverseFluentBuilder, invoke_model::InvokeModelOutput},
    types::{
        AnyToolChoice, ContentBlock, ConverseOutput, ImageBlock, InferenceConfiguration, Message,
        Tool, ToolChoice, ToolConfiguration, ToolInputSchema, ToolSpecification,
    },
};
use base64::Engine;
use futures::future::join_all;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Map;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Semaphore;

#[derive(Clone, Debug)]
pub struct AmazonBedrockProvider {
    region: Region,
    bedrockruntime_client: Arc<aws_sdk_bedrockruntime::Client>,
    bedrockagentruntime_client: Arc<aws_sdk_bedrockagentruntime::Client>,
}

impl AmazonBedrockProvider {
    async fn default() -> Self {
        Self::builder()
            .region(std::env::var("AWS_REGION").expect("AWS_REGION not set"))
            .access_key_id(std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID not set"))
            .secret_access_key(
                std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY not set"),
            )
            .build()
            .await
            .expect("Failed to build AmazonBedrockProvider based on default environment variables")
    }
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
            .set_inference_config(Some(
                InferenceConfiguration::builder()
                    .temperature(request.temperature.unwrap_or(0.0))
                    .top_p(request.top_p.unwrap_or(1.0))
                    .max_tokens(request.max_output_tokens.unwrap_or(0_usize) as i32)
                    .build(),
            ))
            .additional_model_request_fields(aws_smithy_types::Document::Object(
                request
                    .headers
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.as_str().to_string(),
                            aws_smithy_types::Document::String(value.to_str().unwrap().to_string()),
                        )
                    })
                    .collect(),
            ))
            .send()
            .await
            .map_err(|e| {
                tracing::error!("{}", e);
                ResponseGeneratorError::BedrockConverseError(format!("{:?}", e.meta()))
            })?;

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
            .normalize_generate_object_request(&request)
            .map_err(ResponseGeneratorError::Transient)?;

        let converse_response = converse_request
            .set_inference_config(Some(
                InferenceConfiguration::builder()
                    .temperature(request.temperature.unwrap_or(0.0))
                    .top_p(request.top_p.unwrap_or(1.0))
                    .max_tokens(request.max_output_tokens.unwrap_or(5140_usize) as i32)
                    .build(),
            ))
            .additional_model_request_fields(aws_smithy_types::Document::Object(
                request
                    .headers
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.as_str().to_string(),
                            aws_smithy_types::Document::String(value.to_str().unwrap().to_string()),
                        )
                    })
                    .collect(),
            ))
            .send()
            .await
            .map_err(|e| ResponseGeneratorError::BedrockConverseError(e.meta().to_string()))?;

        let converse_output = match converse_response.output {
            Some(output) => output,
            None => {
                return Err(ResponseGeneratorError::EmptyProviderResponse);
            }
        };

        let output_message = match converse_output {
            ConverseOutput::Message(msg) => msg,
            _ => {
                return Err(ResponseGeneratorError::InvalidProviderResponse(
                    "was expecting the output to be a bedrock message".into(),
                ));
            }
        };

        let json_tool = output_message
            .content
            .into_iter()
            .rfind(|content_item| content_item.is_tool_use())
            .ok_or(ResponseGeneratorError::EmptyProviderResponse)?;

        let json_tool_input = json_tool
            .as_tool_use()
            .map_err(|_| {
                ResponseGeneratorError::InvalidProviderResponse(
                    "was expecting the model to call the tool use".into(),
                )
            })?
            .input();

        serde_json::from_value::<T>(utils::aws_smithy_document_to_json(json_tool_input))
            .map(|output| GenerateObjectResponse { output })
            .map_err(|e| {
                ResponseGeneratorError::Deserialization(e, format!("{:?}", json_tool_input))
            })
    }
}

#[async_trait]
impl Reranks for AmazonBedrockProvider {
    async fn rerank(
        &self,
        request: RerankRequest,
    ) -> Result<RerankResponse, ResponseGeneratorError> {
        let inline_sources: Vec<RerankSource> = request
            .documents
            .iter()
            .map(|document| {
                RerankSource::builder()
                    .inline_document_source(
                        RerankDocument::builder()
                            .r#type(RerankDocumentType::Text)
                            .text_document(RerankTextDocument::builder().text(document).build())
                            .build()?,
                    )
                    .r#type(RerankSourceType::Inline)
                    .build()
            })
            .collect::<Result<_, BuildError>>()
            .map_err(|e| {
                ResponseGeneratorError::InvalidArgumentsProvided(format!(
                    "Failed to build Bedrock-compat Rerank Sources from provided documents, Error: {}", e
                ))
            })?;

        let response = self
            .bedrockagentruntime_client
            .rerank()
            .queries(
                RerankQuery::builder()
                    .r#type(RerankQueryContentType::Text)
                    .text_query(RerankTextDocument::builder().text(&request.query).build())
                    .build()
                    .map_err(|e| ResponseGeneratorError::InvalidArgumentsProvided(
                        format!("Failed to build RerankQuery, Details: {}", e)
                    ))?
            )
            .set_sources(
                Some(inline_sources)
            )
            .reranking_configuration(
                RerankingConfiguration::builder()
                    .r#type(RerankingConfigurationType::BedrockRerankingModel)
                    .bedrock_reranking_configuration(
                        BedrockRerankingConfiguration::builder()
                            .model_configuration(
                                BedrockRerankingModelConfiguration::builder()
                                    .model_arn(format!("arn:aws:bedrock:{}::foundation-model/{}",&self.region, &request.model.model_name))
                                    .build()
                                    .map_err(|e| {
                                        ResponseGeneratorError::InvalidArgumentsProvided(
                                            format!("Failed to build BedrockRerankingModelConfiguration, Details: {}", e)
                                        )
                                    })?,
                            )
                            .number_of_results(request.top_k as i32)
                            .build(),
                    )
                    .build()
                    .map_err(|e| {
                        ResponseGeneratorError::InvalidArgumentsProvided(
                            format!("Failed to build RerankingConfiguration, Details: {}", e)
                        )
                    })?,
            )
            .send()
            .await
            .map_err(|e| ResponseGeneratorError::BedrockAgentRerankCommandSendError(e.meta().to_string()))?;

        let results = response.results();

        let (rankings, ranked_documents) = results.iter().try_fold(
            (
                Vec::with_capacity(results.len()),
                Vec::with_capacity(results.len()),
            ),
            |(mut rankings, mut docs), result| {
                let document = request.documents.get(result.index as usize).ok_or(
                    ResponseGeneratorError::InvalidProviderResponse(
                        "Bedrock Rerank API returned an invalid index".to_string(),
                    ),
                )?;
                docs.push(document.clone());
                rankings.push(Ranking {
                    original_index: result.index as usize,
                    score: result.relevance_score,
                    document: document.clone(),
                });
                Ok::<_, ResponseGeneratorError>((rankings, docs))
            },
        )?;

        Ok(RerankResponse {
            rankings,
            ranked_documents,
            raw_fields: Map::with_capacity(0_usize),
        })
    }
}

#[async_trait]
impl ReranksStructuredData for AmazonBedrockProvider {
    async fn rerank_structured<T>(
        &self,
        request: StructuredRerankRequest<T>,
    ) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
    where
        T: Serialize + Clone + Send + Sync,
    {
        let inline_sources: Vec<RerankSource> = match request.serialization_mode {
            SerializationMode::Json => {
                let json_documents: Vec<aws_smithy_types::Document> = request
                    .documents
                    .iter()
                    .map(|doc| {
                        serde_json::to_value(doc)
                            .map(utils::json_to_aws_smithy_document)
                            .map_err(|e| {
                                ResponseGeneratorError::StructuredRerankDocumentsSerializationError(
                                    e.to_string(),
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                json_documents
                    .into_iter()
                    .map(|document| {
                        RerankSource::builder()
                            .inline_document_source(
                                RerankDocument::builder()
                                    .r#type(RerankDocumentType::Json)
                                    .json_document(document)
                                    .build()?,
                            )
                            .r#type(RerankSourceType::Inline)
                            .build()
                    })
                    .collect::<Result<_, BuildError>>()
            }
            SerializationMode::Yaml => {
                let yaml_documents: Vec<String> = request
                    .documents
                    .iter()
                    .map(|doc| {
                        serde_saphyr::to_string(doc).map_err(|e| {
                            ResponseGeneratorError::StructuredRerankDocumentsSerializationError(
                                e.to_string(),
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                yaml_documents
                    .iter()
                    .map(|document| {
                        RerankSource::builder()
                            .inline_document_source(
                                RerankDocument::builder()
                                    .r#type(RerankDocumentType::Text)
                                    .text_document(
                                        RerankTextDocument::builder().text(document).build(),
                                    )
                                    .build()?,
                            )
                            .r#type(RerankSourceType::Inline)
                            .build()
                    })
                    .collect::<Result<_, BuildError>>()
            }
        }
        .map_err(|e| {
            ResponseGeneratorError::InvalidArgumentsProvided(format!(
                "Failed to build Bedrock-compat Rerank Sources from provided documents, Error: {}",
                e
            ))
        })?;

        let response = self
            .bedrockagentruntime_client
            .rerank()
            .queries(
                RerankQuery::builder()
                    .r#type(RerankQueryContentType::Text)
                    .text_query(RerankTextDocument::builder().text(&request.query).build())
                    .build()
                    .map_err(|e| {
                        ResponseGeneratorError::InvalidArgumentsProvided(format!(
                            "Failed to build RerankQuery, Details: {}",
                            e
                        ))
                    })?,
            )
            .set_sources(Some(inline_sources))
            .reranking_configuration(
                RerankingConfiguration::builder()
                    .r#type(RerankingConfigurationType::BedrockRerankingModel)
                    .bedrock_reranking_configuration(
                        BedrockRerankingConfiguration::builder()
                            .model_configuration(
                                BedrockRerankingModelConfiguration::builder()
                                    .model_arn(format!(
                                        "arn:aws:bedrock:{}::foundation-model/{}",
                                        &self.region, &request.model.model_name
                                    ))
                                    .build()
                                    .map_err(|e| {
                                        ResponseGeneratorError::InvalidArgumentsProvided(format!(
                                            "Failed to build BedrockRerankingModelConfiguration, Details: {}",
                                            e
                                        ))
                                    })?,
                            )
                            .number_of_results(request.top_n as i32)
                            .build(),
                    )
                    .build()
                    .map_err(|e| {
                        ResponseGeneratorError::InvalidArgumentsProvided(format!(
                            "Failed to build RerankingConfiguration, Details: {}",
                            e
                        ))
                    })?,
            )
            .send()
            .await
            .map_err(|e| ResponseGeneratorError::BedrockAgentRerankCommandSendError(e.meta().to_string()))?;

        let results = response.results();

        let (rankings, ranked_documents) = results.iter().try_fold(
            (
                Vec::with_capacity(results.len()),
                Vec::with_capacity(results.len()),
            ),
            |(mut rankings, mut docs), result| {
                let document = request.documents.get(result.index as usize).ok_or(
                    ResponseGeneratorError::InvalidProviderResponse(
                        "Bedrock Rerank API returned an invalid index".to_string(),
                    ),
                )?;
                docs.push(document.clone());
                rankings.push(StructuredRanking {
                    original_index: result.index as usize,
                    score: result.relevance_score,
                    document: document.clone(),
                });
                Ok::<_, ResponseGeneratorError>((rankings, docs))
            },
        )?;

        Ok(StructuredRerankResponse {
            rankings,
            ranked_documents,
            raw_fields: Map::with_capacity(0_usize),
        })
    }
}

#[async_trait]
impl Embeds for AmazonBedrockProvider {
    async fn embed(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, ResponseGeneratorError> {
        if request.input.is_empty() {
            return Err(ResponseGeneratorError::InvalidArgumentsProvided(
                "Embedding input cannot be empty".to_string(),
            ));
        }

        let semaphore = Arc::new(Semaphore::new(request.max_parallels));
        let mut embedding_invoke_handles = Vec::with_capacity(request.input.len());

        for data in request.input.into_iter() {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                ResponseGeneratorError::InternalServerError(format!(
                    "Failed to acquire thread lock while making multiple requests. Details: {e}"
                ))
            })?;

            let bedrockruntime_client = Arc::clone(&self.bedrockruntime_client);
            let model_name = request.model.model_name.clone();

            let handle = tokio::spawn(async move {
                let invoke_res = bedrockruntime_client
                    .invoke_model()
                    .model_id(&model_name)
                    .body(aws_smithy_types::Blob::new(
                        serde_json::json!({"inputText":data, "dimensions": request.dimensions, "normalize": request.normalize}).to_string(),
                    ))
                    .send()
                    .await
                    .map_err(|e| {
                        ResponseGeneratorError::BedrockInvokeError(format!(
                            "Failed to invoke Bedrock embedding model: {}",
                            e.meta()
                        ))
                    });
                drop(permit);
                invoke_res
            });

            embedding_invoke_handles.push(handle)
        }

        let results: Result<Vec<InvokeModelOutput>, ResponseGeneratorError> =
            join_all(embedding_invoke_handles)
                .await
                .into_iter()
                .map(|r| {
                    r.map_err(|e| {
                        ResponseGeneratorError::InternalServerError(format!(
                            "Failed to acquire result from the relevant thread. Details: {e}"
                        ))
                    })
                })
                .map(|r| r.and_then(|inner| inner))
                .collect();

        let embeddings = results?
            .into_iter()
            .map(|r| {
                serde_json::from_slice::<AmazonBedrockEmbeddingInvokeModelResponse>(
                    &r.body.into_inner(),
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                ResponseGeneratorError::Deserialization(
                    e,
                    "Failed to deserialize Bedrock embedding response".to_string(),
                )
            })?;

        Ok(EmbeddingResponse {
            embeddings: embeddings.into_iter().map(|e| e.embedding).collect(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AmazonBedrockEmbeddingInvokeModelResponse {
    embedding: Vec<f32>,
    input_text_token_count: usize,
}

impl AmazonBedrockProvider {
    fn normalize_generate_object_request<
        T: Clone + JsonSchema + Serialize + Send + Sync + DeserializeOwned,
    >(
        &self,
        request: &GenerateObjectRequest<T>,
    ) -> anyhow::Result<ConverseFluentBuilder> {
        let system = OpenAIProvider::normalize_system_message(&request.messages);
        let user_messages = Self::normalize_user_messages(&request.messages).unwrap();
        let output_schema_value = serde_json::to_value(&request.output_schema)?;

        Ok(self
            .bedrockruntime_client
            .converse()
            .model_id(request.model.model_name.clone())
            .system(aws_sdk_bedrockruntime::types::SystemContentBlock::Text(
                system,
            ))
            .tool_config(
                ToolConfiguration::builder()
                    .tools(Tool::ToolSpec(
                        ToolSpecification::builder()
                            .name("json_output")
                            .description("Return output as JSON.")
                            .input_schema(ToolInputSchema::Json(
                                utils::json_to_aws_smithy_document(output_schema_value),
                            ))
                            .build()?,
                    ))
                    .tool_choice(ToolChoice::Any(AnyToolChoice::builder().build()))
                    .build()?,
            )
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
            .bedrockruntime_client
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
                                FilePart::Base64(b64_string, media_type) => {
                                    let decoded = base64::engine::general_purpose::STANDARD
                                        .decode(b64_string)
                                        .expect("not a valid base64 string");

                                    ImageBlock::builder()
                                        .source(aws_sdk_bedrockruntime::types::ImageSource::Bytes(
                                            decoded.as_slice().into(),
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
                                        .expect("failed to build image block")
                                }
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
                                    .expect("failed to build image block"),
                            };
                            ContentBlock::Image(image_block)
                        }
                        crate::messages::UserMessagePart::File(_) => {
                            unimplemented!("file handling not yet supported for Bedrock")
                        }
                    })
                    .collect(),
            })
            .collect();

        Ok(user_message_content_blocks)
    }

    fn builder() -> AmazonBedrockProviderBuilder {
        AmazonBedrockProviderBuilder::new()
    }
}

#[derive(Default)]
pub struct AmazonBedrockProviderBuilder {
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    region: Option<Region>,
    provider_name: Option<String>,
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

    pub fn provider_name(mut self, provider_name: impl Into<String>) -> Self {
        self.provider_name = Some(provider_name.into());
        self
    }

    pub async fn build(self) -> Result<AmazonBedrockProvider, AmazonBedrockProviderBuilderError> {
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(self.region.clone())
            .credentials_provider(
                aws_sdk_bedrockruntime::config::Credentials::builder()
                    .access_key_id(
                        self.access_key_id
                            .clone()
                            .ok_or(AmazonBedrockProviderBuilderError::MissingAccessKeyId)?,
                    )
                    .secret_access_key(
                        self.secret_access_key
                            .ok_or(AmazonBedrockProviderBuilderError::MissingSecretAccessKey)?,
                    )
                    .provider_name("umem-ai-bedrock-provider")
                    .build(),
            )
            .load()
            .await;

        Ok(AmazonBedrockProvider {
            region: self
                .region
                .ok_or(AmazonBedrockProviderBuilderError::MissingRegion)?,
            bedrockruntime_client: Arc::new(aws_sdk_bedrockruntime::Client::new(&sdk_config)),
            bedrockagentruntime_client: Arc::new(aws_sdk_bedrockagentruntime::Client::new(
                &sdk_config,
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AIProvider, GenerateObjectRequestBuilder, GenerateTextRequestBuilder, SerializationFormat,
        generate_object, generate_text,
        models::{EmbeddingModel, LanguageModel, RerankingModel},
        rerank, structured_rerank,
    };
    use serde::Deserialize;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_bedrock_generate_object() {
        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("us-west-2")
                .access_key_id("test_access_key_id")
                .secret_access_key("test_secret_access_key")
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

    #[tokio::test]
    async fn test_rerank() {
        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("REGION")
                .access_key_id("ACESS_KEY_ID")
                .secret_access_key("SECRET_ACCESS_KEY")
                .build()
                .await
                .unwrap(),
        ));

        let model = Arc::new(RerankingModel {
            provider,
            model_name: "cohere.rerank-v3-5:0".to_string(),
        });

        let request = RerankRequest::builder()
            .model(model)
            .document("Stock markets reached record highs today as investors reacted positively to economic data.")
            .document("The local sports team won their championship game in a thrilling overtime victory.")
            .document("A new cafe opened downtown, offering a variety of artisanal coffees and pastries.")
            .document("Scientists have discovered a new species of bird in the remote rainforests of the Amazon.")
            .document("The city council has approved a new plan to improve public transportation and reduce traffic congestion.")
            .document("Researchers develop more efficient solar panel technology.")
            .query("environmental sustainability initiatives")
            .top_k(6)
            .build()
            .unwrap();

        let rerank_response = rerank(request).await.unwrap();
        dbg!(&rerank_response);
    }

    #[tokio::test]
    async fn test_structured_rerank() {
        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("REGION")
                .access_key_id("ACESS_KEY_ID")
                .secret_access_key("SECRET_ACCESS_KEY")
                .build()
                .await
                .unwrap(),
        ));

        let model = Arc::new(RerankingModel {
            provider,
            model_name: "cohere.rerank-v3-5:0".to_string(),
        });

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Document {
            id: String,
            content: String,
            metadata: DocumentMetadata,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct DocumentMetadata {
            category: String,
            author: Option<String>,
            timestamp: Option<String>,
        }

        let request = StructuredRerankRequest::builder()
            .model(model)
            .serialization_format(SerializationFormat::Compact)
            .serialization_mode(SerializationMode::Json)
            .documents(vec![
            Document {
                id: "doc_1".to_string(),
                content: "Python is a high-level programming language known for its simplicity and readability.".to_string(),
                metadata: DocumentMetadata {
                    category: "Programming Languages".to_string(),
                    author: Some("Tech Writer".to_string()),
                    timestamp: Some("2024-01-15".to_string()),
                },
            },
            Document {
                id: "doc_2".to_string(),
                content: "JavaScript is primarily used for web development and runs in browsers.".to_string(),
                metadata: DocumentMetadata {
                    category: "Programming Languages".to_string(),
                    author: Some("Tech Writer".to_string()),
                    timestamp: Some("2024-01-16".to_string()),
                },
            },
            Document {
                id: "doc_3".to_string(),
                content: "Machine learning models require large datasets for training.".to_string(),
                metadata: DocumentMetadata {
                    category: "Machine Learning".to_string(),
                    author: Some("Data Scientist".to_string()),
                    timestamp: Some("2024-01-17".to_string()),
                },
            },
            Document {
                id: "doc_4".to_string(),
                content: "Python's pandas library is excellent for data manipulation and analysis.".to_string(),
                metadata: DocumentMetadata {
                    category: "Data Analysis".to_string(),
                    author: Some("Data Analyst".to_string()),
                    timestamp: Some("2024-01-18".to_string()),
                },
            },
            Document {
                id: "doc_5".to_string(),
                content: "The React framework is built on top of JavaScript for building user interfaces.".to_string(),
                metadata: DocumentMetadata {
                    category: "Web Development".to_string(),
                    author: Some("Frontend Dev".to_string()),
                    timestamp: Some("2024-01-19".to_string()),
                },
            },
            Document {
                id: "doc_6".to_string(),
                content: "Deep learning is a subset of machine learning that uses neural networks.".to_string(),
                metadata: DocumentMetadata {
                    category: "Machine Learning".to_string(),
                    author: Some("ML Engineer".to_string()),
                    timestamp: Some("2024-01-20".to_string()),
                },
            },
            Document {
                id: "doc_7".to_string(),
                content: "Python supports multiple programming paradigms including object-oriented and functional programming.".to_string(),
                metadata: DocumentMetadata {
                    category: "Programming Languages".to_string(),
                    author: Some("Tech Writer".to_string()),
                    timestamp: Some("2024-01-21".to_string()),
                },
            },
            Document {
                id: "doc_8".to_string(),
                content: "Node.js allows JavaScript to run on the server side.".to_string(),
                metadata: DocumentMetadata {
                    category: "Backend Development".to_string(),
                    author: Some("Backend Dev".to_string()),
                    timestamp: Some("2024-01-22".to_string()),
                },
            },
        ])
            .query("How to use Python for data analysis?")
            .top_k(6)
            .build()
            .unwrap();

        let rerank_response = structured_rerank(request).await.unwrap();
        dbg!(&rerank_response);
    }

    #[tokio::test]
    async fn embedding_test() {
        use crate::embed::embed;

        let provider = Arc::new(AIProvider::from(
            AmazonBedrockProviderBuilder::default()
                .region("REGION")
                .access_key_id("ACESS_KEY_ID")
                .secret_access_key("SECRET_ACCESS_KEY")
                .build()
                .await
                .unwrap(),
        ));

        let model = Arc::new(EmbeddingModel {
            provider,
            model_name: "amazon.titan-embed-text-v2:0".to_string(),
        });

        let request = EmbeddingRequest::builder()
            .model(model)
            .input(vec![
                "The quick brown fox jumps over the lazy dog.".to_string(),
                "To be or not to be, that is the question.".to_string(),
                "All that glitters is not gold.".to_string(),
            ])
            .build();

        let embedding_response = embed(request).await.unwrap();
        dbg!(&embedding_response);
    }
}

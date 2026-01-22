use crate::{
    AIProvider, AIProviderError, AmazonBedrockProviderBuilder, LanguageModel, LanguageModelError,
    OpenAIProvider,
};
use std::sync::Arc;
use tokio::sync::OnceCell;
use umem_config::CONFIG;

pub static LANGUAGE_MODEL: OnceCell<Arc<LanguageModel>> = OnceCell::const_new();

impl LanguageModel {
    pub async fn get_model() -> Result<Arc<LanguageModel>, LanguageModelError> {
        LANGUAGE_MODEL
            .get_or_try_init(|| async {
                match CONFIG.language_model.provider.clone() {
                    umem_config::Provider::OpenAI(open_ai) => {
                        let openai_provider = OpenAIProvider::builder()
                            .api_key(open_ai.api_key)
                            .base_url(open_ai.base_url)
                            .default_headers(open_ai.default_headers.unwrap_or_default())
                            .project(open_ai.project)
                            .organization(open_ai.organization)
                            .build();

                        let provider = Arc::new(AIProvider::from(openai_provider));

                        Ok(Arc::new(LanguageModel {
                            provider,
                            model_name: CONFIG.language_model.model.clone(),
                        }))
                    }
                    umem_config::Provider::AmazonBedrock(config) => {
                        let provider = AmazonBedrockProviderBuilder::default()
                            .region(config.region)
                            .access_key_id(config.key_id)
                            .secret_access_key(config.access_key)
                            .build()
                            .await
                            .map_err(|e| AIProviderError::ProviderBuilderError(e.into()))?;

                        let provider = Arc::new(AIProvider::from(provider));

                        Ok(Arc::new(LanguageModel {
                            provider,
                            model_name: CONFIG.language_model.model.clone(),
                        }))
                    }
                }
            })
            .await
            .cloned()
    }
}

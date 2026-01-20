use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use umem_core::{
    MemoryContent,
    MemoryKind,
    // MemorySignals, Provenance
};

use umem_ai::{
    GenerateObjectRequestBuilder, GenerateObjectRequestBuilderError,
    GenerateTextRequestBuilderError, LanguageModel, ResponseGeneratorError,
};

pub struct Annotation;

#[derive(Debug, Error)]
pub enum AnnotationError {
    #[error("llm response generate_text failed: {0}")]
    ResponseGeneratorError(#[from] ResponseGeneratorError),

    #[error("llm response generate_text failed: {0}")]
    GenerateTextRequestBuilderError(#[from] GenerateTextRequestBuilderError),

    #[error("llm response generate_object failed: {0}")]
    GenerateObjectRequestBuilderError(#[from] GenerateObjectRequestBuilderError),
}

const ANNOTATION_PROMPT: &str = r#"
You are a memory annotation system. Your task is to analyze a chat session between a user and an AI agent, then extract structured memory metadata that can be stored and retrieved efficiently.

## Input
A conversation transcript containing user messages and agent responses. Focus on extracting what the user learned, decided, asked about, or expressed preferences for—not the back-and-forth dialogue itself.

## Output

### content.summary
Extract the key points from the conversation as a concise, information-dense summary. Requirements:
- Preserve specific details: names, dates, numbers, URLs, technical terms, and concrete values
- Focus on actionable information, facts, preferences, and decisions made by the user
- Capture the resolution or answer, not the process of arriving at it
- Omit filler words, pleasantries, and redundant back-and-forth
- Use clear, direct language

### content.tags
Extract 3-7 lowercase keywords that categorize and index this memory:
- Use singular forms (e.g., "project" not "projects")
- Include domain-specific terms, proper nouns (lowercased), and action verbs where relevant
- Prioritize terms useful for future retrieval

### kind
Classify the memory into exactly one type:
- **Semantic**: General knowledge, facts, concepts, definitions, explanations
- **Episodic**: Specific events, experiences, occurrences with temporal or spatial context
- **Procedural**: How-to knowledge, workflows, step-by-step processes, techniques, habits
- **Instruction**: Explicit directives, user preferences, rules, constraints, configurations
- **Relational**: Information about people, organizations, entities, and their relationships
- **Working**: Temporary context relevant only to an ongoing task or session
- **Prospective**: Future intentions, goals, plans, reminders, scheduled commitments
"#;

#[derive(Clone, schemars::JsonSchema, Serialize, Deserialize)]
pub struct LLMAnnotated {
    pub content: MemoryContent,
    pub kind: MemoryKind,
    // pub signals: MemorySignals,
    // pub provenance: Provenance,
}

impl Annotation {
    pub async fn generate(
        raw_content: impl Into<String>,
        model: Arc<LanguageModel>,
    ) -> Result<LLMAnnotated, AnnotationError> {
        let request = GenerateObjectRequestBuilder::<LLMAnnotated>::new()
            .model(model)
            .system(ANNOTATION_PROMPT)
            .prompt(raw_content)
            .max_output_tokens(10000)
            .temperature(0.7)
            .build()?;

        let annotations = umem_ai::generate_object(request).await?;
        Ok(annotations.output)
    }
}

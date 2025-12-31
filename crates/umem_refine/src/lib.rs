use rust_stemmers::{Algorithm, Stemmer};
use rustc_hash::FxHashSet;
use stopwords::Language;
use stopwords::{Stopwords, NLTK};
use thiserror::Error;

mod tantivy_tokenizer;

use tantivy_tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer, Token};

pub struct Segmenter;

#[derive(Error, Debug)]
pub enum RefineError {
    #[error("refine stopwords failed")]
    StopWordsError,
}

impl Segmenter {
    pub fn process(query: impl Into<String>) -> Result<Vec<String>, RefineError> {
        let stemmer = Stemmer::create(Algorithm::English);
        let stopwords = NLTK::stopwords(Language::English).ok_or(RefineError::StopWordsError)?;
        let stopwords: FxHashSet<&str> = stopwords.iter().copied().collect();
        let mut text_analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .build();

        let query = query.into();
        let mut token_stream = text_analyzer.token_stream(&query);
        let mut tokens: Vec<String> = vec![];
        let mut add_token = |token: &Token| {
            if !stopwords.contains(token.text.as_str()) {
                tokens.push(stemmer.stem(&token.text).to_string());
            }
        };

        token_stream.process(&mut add_token);

        Ok(tokens)
    }
}

use rust_stemmers::{Algorithm, Stemmer};
use rustc_hash::FxHashSet;
use stopwords::Language;
use stopwords::{Stopwords, NLTK};
use thiserror::Error;

mod tantivy_tokenizer;

use tantivy_tokenizer::{SimpleTokenizer, TextAnalyzer, Token};

use crate::tantivy_tokenizer::LowerCaser;

pub struct Refine;

#[derive(Error, Debug)]
pub enum RefineError {
    #[error("refine stopwords failed")]
    StopWordsError,
}

impl Refine {
    pub fn process(query: impl Into<String>) -> Result<Vec<String>, RefineError> {
        let stemmer = Stemmer::create(Algorithm::English);
        let stopwords = NLTK::stopwords(Language::English)
            .unwrap_or_else(|| panic!("{}", RefineError::StopWordsError.to_string()));
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

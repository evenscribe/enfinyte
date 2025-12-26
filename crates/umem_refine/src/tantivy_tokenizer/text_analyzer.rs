/// The tokenizer module contains all of the tools used to process
/// text in `tantivy`.
use super::token::{BoxTokenStream, TokenFilter, Tokenizer};

/// `TextAnalyzer` tokenizes an input text into tokens and modifies the resulting `TokenStream`.
#[derive(Clone)]
pub struct TextAnalyzer {
    tokenizer: Box<dyn BoxableTokenizer>,
}

impl Tokenizer for Box<dyn BoxableTokenizer> {
    type TokenStream<'a> = BoxTokenStream<'a>;

    // Note: we want to call `box_token_stream` on the concrete `Tokenizer`
    // implementation, not the `BoxableTokenizer` one as it will cause
    // a recursive call (and a stack overflow).
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        (**self).box_token_stream(text)
    }
}

impl Clone for Box<dyn BoxableTokenizer> {
    // Note: we want to call `box_clone` on the concrete `Tokenizer`
    // implementation in order to clone the concrete `Tokenizer`.
    fn clone(&self) -> Self {
        (**self).box_clone()
    }
}

/// A boxable `Tokenizer`, with its `TokenStream` type erased.
pub trait BoxableTokenizer: 'static + Send + Sync {
    /// Creates a boxed token stream for a given `str`.
    fn box_token_stream<'a>(&'a mut self, text: &'a str) -> BoxTokenStream<'a>;
    /// Clone this tokenizer.
    fn box_clone(&self) -> Box<dyn BoxableTokenizer>;
}

impl<T: Tokenizer> BoxableTokenizer for T {
    fn box_token_stream<'a>(&'a mut self, text: &'a str) -> BoxTokenStream<'a> {
        BoxTokenStream::new(self.token_stream(text))
    }
    fn box_clone(&self) -> Box<dyn BoxableTokenizer> {
        Box::new(self.clone())
    }
}

impl<T: Tokenizer + Clone> From<T> for TextAnalyzer {
    fn from(tokenizer: T) -> Self {
        TextAnalyzer::builder(tokenizer).build()
    }
}

impl TextAnalyzer {
    /// Create a new TextAnalyzerBuilder
    pub fn builder<T: Tokenizer>(tokenizer: T) -> TextAnalyzerBuilder<T> {
        TextAnalyzerBuilder { tokenizer }
    }

    /// Creates a token stream for a given `str`.
    pub fn token_stream<'a>(&'a mut self, text: &'a str) -> BoxTokenStream<'a> {
        self.tokenizer.token_stream(text)
    }
}

/// Builder helper for [`TextAnalyzer`]
pub struct TextAnalyzerBuilder<T = Box<dyn BoxableTokenizer>> {
    tokenizer: T,
}

#[allow(dead_code)]
impl<T: Tokenizer> TextAnalyzerBuilder<T> {
    /// Appends a token filter to the current builder.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tantivy::tokenizer::*;
    ///
    /// let en_stem = TextAnalyzer::builder(SimpleTokenizer::default())
    ///     .filter(RemoveLongFilter::limit(40))
    ///     .filter(LowerCaser)
    ///     .filter(Stemmer::default())
    ///     .build();
    /// ```
    pub fn filter<F: TokenFilter>(self, token_filter: F) -> TextAnalyzerBuilder<F::Tokenizer<T>> {
        TextAnalyzerBuilder {
            tokenizer: token_filter.transform(self.tokenizer),
        }
    }

    /// Boxes the internal tokenizer. This is useful for adding dynamic filters.
    /// Note: this will be less performant than the non boxed version.
    pub fn dynamic(self) -> TextAnalyzerBuilder {
        let boxed_tokenizer = Box::new(self.tokenizer);
        TextAnalyzerBuilder {
            tokenizer: boxed_tokenizer,
        }
    }

    /// Appends a token filter to the current builder and returns a boxed version of the
    /// tokenizer. This is useful when you want to build a `TextAnalyzer` dynamically.
    /// Prefer using `TextAnalyzer::builder(tokenizer).filter(token_filter).build()` if
    /// possible as it will be more performant and create less boxes.
    pub fn filter_dynamic<F: TokenFilter>(self, token_filter: F) -> TextAnalyzerBuilder {
        self.filter(token_filter).dynamic()
    }

    /// Finalize building the TextAnalyzer
    pub fn build(self) -> TextAnalyzer {
        TextAnalyzer {
            tokenizer: Box::new(self.tokenizer),
        }
    }
}

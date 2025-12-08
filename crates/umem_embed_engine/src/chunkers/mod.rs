use crate::data_type::DataType;
pub mod pdf_chunker;

pub trait Chunker {
    fn get_data_type(&self) -> DataType;
    fn get_chunks(&self, content: String) -> Vec<String>;
}

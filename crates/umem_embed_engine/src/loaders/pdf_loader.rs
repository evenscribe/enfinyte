use super::{LoadDataResult, Loader};
use crate::{HashMap, engine::AddSource, reqwest_client, utils};
use anyhow::Result;
use reqwest::header::USER_AGENT;

pub struct PdfLoader;

#[async_trait::async_trait]
impl Loader for PdfLoader {
    async fn load_data(&self, source: &AddSource) -> Result<LoadDataResult> {
        let doc = match source {
            AddSource::LocalFile(local_url) => lopdf::Document::load(local_url)?,
            AddSource::Url(remote_url) => {
                let resp_bytes = reqwest_client
                    .get(remote_url)
                    .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36" )
                    .send()
                    .await?
                    .bytes()
                    .await?;
                lopdf::Document::load_mem(&resp_bytes)?
            }
        };

        let mut data = Vec::new();
        let mut all_text_contents = Vec::new();

        let metadata = doc
            .trailer
            .get(b"Info")
            .ok()
            .and_then(|info| match info {
                lopdf::Object::Dictionary(dict) => Some(dict),
                lopdf::Object::Reference(id) => doc.get_dictionary(*id).ok(),
                _ => None,
            })
            .map(convert_lopdf_dict_to_metadata_hashmap)
            .unwrap_or_default();

        for (object_number, _) in doc.page_iter() {
            let text = utils::clean_string(doc.extract_text(&[object_number])?);
            all_text_contents.push(text.clone());
            data.push((text, metadata.clone()));
        }

        let mut hash_content = all_text_contents.join(" ");
        hash_content.push_str(&source.to_string());

        let doc_id = blake3::hash(hash_content.as_bytes()).to_hex().to_string();

        Ok(LoadDataResult { doc_id, data })
    }
}

const METADATA_WE_CARE_FOR: [&str; 8] = [
    "Title",
    "Author",
    "Subject",
    "Keywords",
    "Creator",
    "Producer",
    "CreationDate",
    "ModDate",
];

fn convert_lopdf_dict_to_metadata_hashmap(dict: &lopdf::Dictionary) -> HashMap<String, String> {
    let mut metadata: HashMap<String, String> = HashMap::default();

    for &key in METADATA_WE_CARE_FOR.iter() {
        if let Ok(value) = dict.get(key.as_bytes()) {
            if let Ok(value_str) = value.as_str() {
                metadata.insert(
                    key.to_string(),
                    String::from_utf8_lossy(value_str).into_owned(),
                );
            }
        }
    }

    metadata
}

impl PdfLoader {
    pub fn new() -> Self {
        PdfLoader
    }
}

use crate::{engine::AddSource, reqwest_client};
use ada_url::SchemeType;
use anyhow::{Result, anyhow};
use core::fmt;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use yaml_rust2::YamlLoader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Text,
    YoutubeVideo,
    PdfFile,
    WebPage,
    Sitemap,
    Xml,
    Docx,
    DocsSite,
    Notion,
    Csv,
    Mdx,
    QnaPair,
    Image,
    Unstructured,
    Json,
    OpenApi,
    Gmail,
    Substack,
    YoutubeChannel,
    Discord,
    Custom,
    RssFeed,
    Beehiiv,
    GoogleDrive,
    Directory,
    Slack,
    Dropbox,
    TextFile,
    ExcelFile,
    Audio,
    Github,
}

pub const DIRECT_DATA_TYPES: [DataType; 1] = [DataType::Text];

// TODO: add all indirect data types
pub const INDIRECT_DATA_TYPES: [DataType; 2] = [DataType::PdfFile, DataType::Json];

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::YoutubeChannel => write!(f, "youtube_channel"),
            DataType::GoogleDrive => write!(f, "google_drive"),
            DataType::DocsSite => write!(f, "docs_site"),
            DataType::PdfFile => write!(f, "pdf_file"),
            DataType::TextFile => write!(f, "text_file"),
            DataType::WebPage => write!(f, "web_page"),
            DataType::Sitemap => write!(f, "sitemap"),
            DataType::Csv => write!(f, "csv"),
            DataType::Mdx => write!(f, "mdx"),
            DataType::Docx => write!(f, "docx"),
            DataType::Json => write!(f, "json"),
            DataType::OpenApi => write!(f, "openapi"),
            DataType::Audio => write!(f, "audio"),
            DataType::Text => write!(f, "text"),
            DataType::YoutubeVideo => write!(f, "youtube_video"),
            DataType::Xml => write!(f, "xml"),
            DataType::Notion => write!(f, "notion"),
            DataType::QnaPair => write!(f, "qna_pair"),
            DataType::Image => write!(f, "image"),
            DataType::Unstructured => write!(f, "unstructured"),
            DataType::Gmail => write!(f, "gmail"),
            DataType::Substack => write!(f, "substack"),
            DataType::Discord => write!(f, "discord"),
            DataType::Custom => write!(f, "custom"),
            DataType::RssFeed => write!(f, "rss_feed"),
            DataType::Beehiiv => write!(f, "beehiiv"),
            DataType::Directory => write!(f, "directory"),
            DataType::Slack => write!(f, "slack"),
            DataType::Dropbox => write!(f, "dropbox"),
            DataType::ExcelFile => write!(f, "excel_file"),
            DataType::Github => write!(f, "github"),
        }
    }
}

impl DataType {
    pub async fn try_from_source(source: &AddSource) -> Result<Self> {
        match source {
            AddSource::Url(url) => detect_data_type_from_url(&url).await,
            AddSource::LocalFile(local_path) => detect_data_type_from_file(&local_path),
        }
    }
}

fn detect_data_type_from_file(local_path: &str) -> Result<DataType> {
    match local_path.rsplit('.').next() {
        Some("docx") => Ok(DataType::Docx),
        Some("csv") => Ok(DataType::Csv),
        Some("xml") => Ok(DataType::Xml),
        Some("md") | Some("mdx") => Ok(DataType::Mdx),
        Some("txt") => Ok(DataType::TextFile),
        Some("pdf") => Ok(DataType::PdfFile),
        Some("json") => Ok(DataType::Json),
        Some("yaml") | Some("yml") => {
            let file_content = fs::read_to_string(local_path).map_err(|_| {
                anyhow!(
                    "failed to read the content from the file path {}",
                    local_path
                )
            })?;

            if let Err(e) = is_valid_openapi_yaml(&file_content) {
                return Err(e);
            }

            Ok(DataType::OpenApi)
        }
        _ => match fs::read_to_string(local_path) {
            Ok(_) => Ok(DataType::TextFile),
            Err(_) => Err(anyhow!(
                "Unsupported file extension for local path: {}",
                local_path
            )),
        },
    }
}

const YOUTUBE_ALLOWED_HOSTNAMES: [&str; 6] = [
    "www.youtube.com",
    "m.youtube.com",
    "youtu.be",
    "youtube.com",
    "vid.plus",
    "www.youtube-nocookie.com",
];

const NOTION_ALLOWED_HOSTNAMES: [&str; 2] = ["www.notion.so", "notion.so"];

const SUPPORTED_AUDIO_FILE_EXTENSIONS: [&str; 11] = [
    ".mp3", ".mp4", ".mp2", ".aac", ".wav", ".flac", ".pcm", ".m4a", ".ogg", ".opus", ".webm",
];

lazy_static! {
    static ref google_drive_regex: Regex =
        Regex::new(r"^drive\.google\.com\/drive\/(?:u\/\d+\/)folders\/([a-zA-Z0-9_-]+)$").unwrap();
}

fn is_google_drive_folder(to_string: String) -> bool {
    google_drive_regex.is_match(to_string.as_str())
}

fn is_valid_openapi_yaml(yaml_content: &str) -> Result<()> {
    let yaml_content = YamlLoader::load_from_str(yaml_content)
        .map_err(|_| anyhow!("Invalid yaml content supplied: {}", yaml_content))?;

    let doc = &yaml_content[0];

    if doc["openapi"].is_null() && doc["info"].is_null() {
        return Err(anyhow!(
            "the yaml content is not a valid OpenAPI specification: {:#?}",
            yaml_content
        ));
    }

    Ok(())
}

async fn detect_data_type_from_url(url: &str) -> Result<DataType> {
    let url = match ada_url::Url::try_from(url) {
        Ok(url) => url,
        Err(_) => {
            return Err(anyhow!(
                "Invalid URL provided, AddSource::Url must be a valid URL"
            ));
        }
    };

    let host_name = url.hostname();
    let path = url.pathname();

    if YOUTUBE_ALLOWED_HOSTNAMES
        .iter()
        .any(|&hn| host_name.contains(hn))
    {
        Ok(DataType::YoutubeVideo)
    } else if NOTION_ALLOWED_HOSTNAMES
        .iter()
        .any(|&hn| host_name.contains(hn))
    {
        Ok(DataType::Notion)
    } else if path.ends_with(".pdf") {
        Ok(DataType::PdfFile)
    } else if path.ends_with(".xml") {
        Ok(DataType::Sitemap)
    } else if path.ends_with(".csv") {
        Ok(DataType::Csv)
    } else if path.ends_with(".mdx") || path.ends_with(".md") {
        Ok(DataType::Mdx)
    } else if path.ends_with(".docx") {
        Ok(DataType::Json)
    } else if SUPPORTED_AUDIO_FILE_EXTENSIONS
        .iter()
        .any(|&ext| path.ends_with(ext))
    {
        Ok(DataType::Audio)
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
        let url_string = url.to_string();
        let yaml_content_response = reqwest_client
            .get(&url_string)
            .send()
            .await
            .map_err(|_| anyhow!("failed to get content from the url {}", url_string))?;

        let yaml_content_body = yaml_content_response.text().await.map_err(|_| {
            anyhow!(
                "failed to read the content body from the url {}",
                url_string
            )
        })?;

        if let Err(e) = is_valid_openapi_yaml(&yaml_content_body) {
            return Err(e);
        }

        Ok(DataType::OpenApi)
    } else if url.pathname().ends_with(".json") {
        Ok(DataType::Json)
    } else if url.host().contains("github.com") {
        Ok(DataType::Github)
    } else if url.host().contains("docs")
        || (url.pathname().contains("docs") && url.scheme_type() != SchemeType::File)
    {
        Ok(DataType::DocsSite)
    } else if is_google_drive_folder(url.to_string()) {
        Ok(DataType::GoogleDrive)
    } else {
        Ok(DataType::WebPage)
    }
}

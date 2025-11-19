use anyhow::{Result, anyhow};

use crate::engine::AddSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectDataType {
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndirectDataType {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialDataType {
    QnaPair,
}

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
}

impl TryFrom<AddSource> for DataType {
    fn try_from(source: AddSource) -> Result<Self, Self::Error> {
        match source {
            AddSource::Url(url) => detect_data_type_from_url(&url),
            AddSource::LocalFile(local_path) => detect_data_type_from_file(&local_path),
        }
    }

    type Error = anyhow::Error;
}

fn detect_data_type_from_file(local_path: &str) -> Result<DataType> {
    todo!()
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

const SUPPORTED_AUDIO_FILE_EXTENSIONS:[&str;11] = [".mp3", ".mp4", ".mp2", ".aac", ".wav", ".flac", ".pcm", ".m4a", ".ogg", ".opus", ".webm"]

fn detect_data_type_from_url(url: &str) -> Result<DataType> {
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
    } else if path.ends_with(".yaml") || path.ends_with(".yml"){
        const yamlContent = req

    }
    else {
        Ok(DataType::Unstructured)
    }
}

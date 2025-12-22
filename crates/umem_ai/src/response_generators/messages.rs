#[derive(Clone)]
pub enum Message {
    System(String),
    User(UserModelMessage),
}

#[derive(Clone)]
pub enum UserModelMessage {
    Text(String),
    Parts(Vec<UserMessagePart>),
}

#[derive(Clone)]
pub enum UserMessagePart {
    Text(String),
    Image(FilePart),
    File(FilePart),
}

#[derive(Clone)]
pub enum FilePart {
    Url(String, Option<mime::Mime>),
    Base64(String, Option<mime::Mime>),
    Buffer(Vec<u8>, Option<mime::Mime>),
}

impl From<String> for UserModelMessage {
    fn from(value: String) -> Self {
        UserModelMessage::Text(value)
    }
}

impl From<Vec<UserMessagePart>> for UserModelMessage {
    fn from(value: Vec<UserMessagePart>) -> Self {
        UserModelMessage::Parts(value)
    }
}

impl From<String> for UserMessagePart {
    fn from(value: String) -> Self {
        UserMessagePart::Text(value)
    }
}
impl From<FilePart> for UserMessagePart {
    fn from(value: FilePart) -> Self {
        let is_image = match &value {
            FilePart::Url(_, Some(mime))
            | FilePart::Base64(_, Some(mime))
            | FilePart::Buffer(_, Some(mime)) => mime.type_() == mime::IMAGE,
            _ => false,
        };

        if is_image {
            UserMessagePart::Image(value)
        } else {
            UserMessagePart::File(value)
        }
    }
}

impl FilePart {
    pub fn from_url<T: Into<String>>(url: T, media_type: Option<mime::Mime>) -> Self {
        FilePart::Url(url.into(), media_type)
    }

    pub fn from_base64<T: Into<String>>(base64: T, media_type: Option<mime::Mime>) -> Self {
        FilePart::Base64(base64.into(), media_type)
    }

    pub fn from_buffer<T: Into<Vec<u8>>>(buffer: T, media_type: Option<mime::Mime>) -> Self {
        FilePart::Buffer(buffer.into(), media_type)
    }
}

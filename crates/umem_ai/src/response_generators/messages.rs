pub enum Message {
    System(String),
    User(UserModelMessage),
}

pub enum UserModelMessage {
    Text(String),
    Parts(Vec<UserMessagePart>),
}

pub enum UserMessagePart {
    Text(String),
    Image(FilePart),
    File(FilePart),
}

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
    pub fn from_url(url: Into<String>, media_type: Option<Into<mime::Mime>>) -> Self {
        FilePart::Url(url, media_type)
    }

    pub fn from_base64(base64: Into<String>, media_type: Option<Into<mime::Mime>>) -> Self {
        FilePart::Base64(base64, media_type)
    }

    pub fn from_buffer(buffer: Into<Vec<u8>>, media_type: Option<Into<mime::Mime>>) -> Self {
        FilePart::Buffer(buffer, media_type)
    }
}

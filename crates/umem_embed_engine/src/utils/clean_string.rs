use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    static ref CONSECUTIVE_RE: Regex = Regex::new(r"([^\w\s])\1*").unwrap();
}

pub fn clean_string(text: String) -> String {
    let cleaned = WHITESPACE_RE.replace_all(text.trim(), " ");

    let cleaned: String = cleaned
        .chars()
        .map(|c| match c {
            '\\' => None,
            '#' => Some(' '),
            _ => Some(c),
        })
        .flatten()
        .collect();

    CONSECUTIVE_RE.replace_all(&cleaned, "$1").into_owned()
}

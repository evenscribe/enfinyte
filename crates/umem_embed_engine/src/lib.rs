mod client;
pub mod config;
pub mod data_type;
pub mod engine;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

pub use client::Client;
use lazy_static::lazy_static;

lazy_static! {
    static ref reqwest_client: reqwest::Client = reqwest::Client::new();
}

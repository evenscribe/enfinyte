pub mod chunkers;
pub mod client;
pub mod config;
pub mod data_formatter;
pub mod data_type;
pub mod engine;
pub mod loaders;
pub mod utils;
pub mod vectordb;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type HashSet<T> = rustc_hash::FxHashSet<T>;

pub use client::Client;
use lazy_static::lazy_static;

lazy_static! {
    static ref reqwest_client: reqwest::Client = reqwest::Client::new();
}

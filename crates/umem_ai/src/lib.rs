// TODO: remove this allow once the module is fully implemented
#![allow(dead_code)]
mod model_impl;
pub mod models;
mod providers;
mod response_generators;
mod utils;
use lazy_static::lazy_static;

pub use model_impl::*;
pub use models::*;
pub use providers::*;
pub use response_generators::*;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

lazy_static! {
    static ref reqwest_client: reqwest::Client = reqwest::Client::new();
}

use lazy_static::lazy_static;
use reqwest::Client;
mod cf_baai_bge_m3;
pub use cf_baai_bge_m3::CfBaaiBgeM3Embeder;

lazy_static! {
    static ref client: Client = Client::new();
}

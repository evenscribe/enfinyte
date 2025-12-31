use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::Result;
use dotenv::dotenv;
use serde::Deserialize;
use tracing::info;
use umem::tracing_conf;
use umem_controller::{CreateMemoryRequest, MemoryController};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing_conf::init_tracing()?;

    let file = File::open("data_chatgpt/conversations_general.jsonl")?;
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();

    #[derive(Debug, Deserialize)]
    struct Chat {
        title: String,
    }

    let mut count = 1;

    loop {
        buf.clear();
        let bytes = reader.read_until(b'\n', &mut buf)?;
        if bytes == 0 {
            break;
        }

        let chat: Chat = match serde_json::from_slice(&buf) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Bad JSON: {}", e);
                continue;
            }
        };

        info!("working on chat {count} : {} ", chat.title);

        let raw_chats = std::str::from_utf8(&buf).unwrap();

        let _ = MemoryController::create(
            CreateMemoryRequest::builder()
                .user_id(Some("harry".to_string()))
                .raw_content(raw_chats.to_owned())
                .build(),
            None,
        )
        .await;

        count += 1;
    }

    Ok(())
}

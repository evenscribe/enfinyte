fn main() {}
// use std::{
//     fs::File,
//     io::{BufRead, BufReader},
//     sync::{
//         atomic::{AtomicUsize, Ordering},
//         Arc,
//     },
// };

// use anyhow::Result;
// use dotenv::dotenv;
// use tracing::info;
// use umem::tracing_conf;
// use umem_controller::{CreateMemoryRequest, MemoryController};

// const CONCURRENCY_LIMIT: usize = 10;

// #[tokio::main]
// async fn main() -> Result<()> {
//     // dotenv().ok();
//     // let _guard = tracing_conf::init_tracing()?;

//     // let file = File::open("data_chatgpt/conversations_general.jsonl")?;
//     // let reader = BufReader::new(file);
//     // let lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

//     // let total = lines.len();
//     // info!("loaded {total} chat records, processing with {CONCURRENCY_LIMIT} concurrent tasks");

//     // let semaphore = Arc::new(tokio::sync::Semaphore::new(CONCURRENCY_LIMIT));
//     // let completed = Arc::new(AtomicUsize::new(0));

//     // let mut tasks = Vec::new();

//     // for raw_chats in lines {
//     //     let sem = semaphore.clone();
//     //     let completed = completed.clone();

//     //     let task = tokio::spawn(async move {
//     //         let _permit = sem.acquire().await?;

//     //         let _ = MemoryController::create(
//     //             CreateMemoryRequest::builder()
//     //                 .user_id(Some("harry".to_string()))
//     //                 .raw_content(raw_chats)
//     //                 .build(),
//     //             None,
//     //         )
//     //         .await;

//     //         let count = completed.fetch_add(1, Ordering::SeqCst) + 1;
//     //         info!("Progress: {count}/{total} completed");

//     //         Ok::<(), anyhow::Error>(())
//     //     });

//     //     tasks.push(task);
//     // }

//     // for task in tasks {
//     //     task.await??;
//     // }

//     // info!("all {total} chat records processed successfully");
//     Ok(())
// }

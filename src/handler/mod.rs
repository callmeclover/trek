pub mod debian;

use std::sync::{Arc, Mutex};

use anyhow::{Ok as FinishOk, Result};
use bytes::{BufMut, Bytes};
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::{Client, Response};

pub trait RepositoryHandler {
    type Package;
    async fn fetch_package_data(&self) -> Result<Vec<String>>;
    fn parse_packages(&self, index_data: String) -> Vec<Self::Package>;
    fn create_database(&mut self) -> Result<()>;
    fn store_packages(&self, packages: Vec<Self::Package>) -> Result<()>;
    async fn sync_repository(&mut self) -> Result<()>;
}

pub async fn download_with_progress(tasks: Vec<String>) -> Result<Vec<Vec<u8>>> {
    let progress: MultiProgress = MultiProgress::new();
    let client: Client = Client::new();

    let bodies = stream::iter(tasks)
        .map(|url: String| {
            let client: Client = client.clone();
            tokio::spawn({
                let progress_binding: MultiProgress = progress.clone();
                async move {
                    let response: Response = client.get(url.clone()).send().await?;
                    let total_size: u64 = response.content_length().unwrap_or(0);

                    let pb: ProgressBar = ProgressBar::new(total_size)
                        .with_style(
                            ProgressStyle::default_bar()
                                .template("{msg} {bar:60} {percent}% {bytes}/{total_bytes}")?
                                .progress_chars("█▓▒░"),
                        )
                        .with_finish(ProgressFinish::WithMessage("Downloaded!".into()))
                        .with_message(url.to_string());
                    progress_binding.add(pb.clone());

                    let mut stream = response.bytes_stream();

                    let mut bytes: Vec<u8> = vec![];
                    while let Some(chunk) = stream.next().await {
                        let chunk: Bytes = chunk?;
                        bytes.put(chunk);
                        pb.set_position(bytes.len() as u64);
                    }
                    FinishOk::<Vec<u8>>(bytes)
                }
            })
        })
        .buffer_unordered(5);

    let out: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));
    bodies
        .for_each(
            |b: std::result::Result<
                std::result::Result<Vec<u8>, anyhow::Error>,
                tokio::task::JoinError,
            >| {
                let binding: Arc<Mutex<Vec<Vec<u8>>>> = out.clone();
                async move {
                    match b {
                        Ok(Ok(b)) => binding.lock().unwrap().push(b),
                        Ok(Err(e)) => eprintln!("Got a reqwest::Error: {e}"),
                        Err(e) => eprintln!("Got a tokio::JoinError: {e}"),
                    }
                }
            },
        )
        .await;

    let out: Vec<Vec<u8>> = (*out.lock().unwrap().clone()).to_vec();
    Ok(out)
}

pub mod debian;

use std::sync::{Arc, Mutex};

use anyhow::{Ok as FinishOk, Result};
use bytes::{BufMut, Bytes};
use debian::DebianPackage;
use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};
use reqwest::{Client, Response};

pub trait RepositoryHandler {
    async fn fetch_package_data(&self) -> Result<Vec<String>>;
    fn parse_packages(&self, index_data: String) -> Vec<DebianPackage>;
    fn create_database(&self);
    fn store_packages(&self);
    fn sync_repository(&self);
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
                        .with_finish(ProgressFinish::WithMessage("Downloaded!".into())).with_message(url.to_string());
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

/*
def parse_packages(packages_data):
    """Parse the Packages.gz file data and extract package info."""
    packages = []
    current_package = {}

    for line in packages_data.splitlines():
        if line.startswith("Package:"):
            if current_package:
                packages.append(current_package)
            current_package = {"Package": line.split(":", 1)[1].strip()}
        elif line.startswith("Version:"):
            current_package["Version"] = line.split(":", 1)[1].strip()
        elif line.startswith("Architecture:"):
            current_package["Architecture"] = line.split(":", 1)[1].strip()
        elif line.startswith("Description:"):
            current_package["Description"] = line.split(":", 1)[1].strip()

    if current_package:
        packages.append(current_package)  # Append last package

    return packages

def create_database():
    """Create the SQLite database and table."""
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()

    cursor.execute("""
    CREATE TABLE IF NOT EXISTS packages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        version TEXT NOT NULL,
        architecture TEXT NOT NULL,
        description TEXT
    );
    """)

    conn.commit()
    conn.close()

def store_packages(packages):
    """Store the parsed package data into the SQLite database."""
    conn = sqlite3.connect(DB_FILE)
    cursor = conn.cursor()

    for package in packages:
        cursor.execute("""
        INSERT INTO packages (name, version, architecture, description)
        VALUES (?, ?, ?, ?)
        """, (package['Package'], package['Version'], package['Architecture'], package.get('Description', 'N/A')))

    conn.commit()
    conn.close()

def sync_repository():
    """Sync the Debian repository and save package info to the database."""
    create_database()  # Ensure the database and table exist

    # Step 1: Fetch and decompress the data
    packages_data = fetch_package_data()
    if not packages_data:
        print("Failed to fetch data.")
        return

    # Step 2: Parse the data
    packages = parse_packages(packages_data)
    print(f"Parsed {len(packages)} packages.")

    # Step 3: Store the data in the database
    store_packages(packages)
    print(f"Stored {len(packages)} packages in the database.")

if __name__ == "__main__":
    sync_repository()
*/

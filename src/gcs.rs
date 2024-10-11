use anyhow::Result;
use google_cloud_storage::{
    client::Client,
    http::{
        objects::upload::{Media, UploadObjectRequest, UploadType},
        resumable_upload_client::ChunkSize,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufReader, Read, Seek, Write},
};

const CHUNK_SIZE: usize = 256 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct UploadState {
    url: String,
    last_chunk: u32,
}

pub async fn upload_to_gcs(client: &Client, bucket: &str, file_name: &str) -> Result<()> {
    let object_name = format!("berachain/snapshots/{}", file_name);
    let state_file = format!("{}.upload_state", file_name);

    let (uploader, mut last_chunk) = if let Ok(state) = read_state(&state_file) {
        println!("Resuming previous upload");
        (client.get_resumable_upload(state.url), state.last_chunk)
    } else {
        println!("Starting new upload");
        let upload_type = UploadType::Simple(Media::new(object_name));
        let upload_request = UploadObjectRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };
        let uploader = client
            .prepare_resumable_upload(&upload_request, &upload_type)
            .await?;
        (uploader, 0)
    };

    let file = File::open(file_name)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(CHUNK_SIZE * 2, file);

    loop {
        let start = u64::from(last_chunk) * CHUNK_SIZE as u64;

        if start >= file_size {
            break;
        }

        reader.seek(std::io::SeekFrom::Start(start))?;
        let mut buffer = vec![0; CHUNK_SIZE];
        let n = reader.read(&mut buffer)?;
        buffer.truncate(n);

        let end = start + n as u64 - 1;
        let chunk = ChunkSize::new(start, end, Some(file_size));
        uploader.upload_multiple_chunk(buffer, &chunk).await?;

        last_chunk += 1;

        save_state(
            &state_file,
            &UploadState {
                url: uploader.url().to_string(),
                last_chunk,
            },
        )?;

        print_progress(start + n as u64, file_size, file_name);
    }

    println!("\nUpload completed successfully for {}", file_name);
    std::fs::remove_file(state_file)?;

    Ok(())
}

fn print_progress(uploaded: u64, total: u64, file_name: &str) {
    let percentage = (uploaded.min(total) as f64 / total as f64) * 100.0;
    let uploaded_gb = uploaded as f64 / 1_073_741_824.0;
    let total_gb = total as f64 / 1_073_741_824.0;
    print!(
        "\rUploaded {:.2} GB of {:.2} GB ({:.2}%) for {}",
        uploaded_gb, total_gb, percentage, file_name
    );
    std::io::stdout().flush().unwrap();
}

fn save_state(file_name: &str, state: &UploadState) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_name)?;
    let data = serde_json::to_string(state)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn read_state(file_name: &str) -> Result<UploadState> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let state: UploadState = serde_json::from_str(&contents)?;
    Ok(state)
}

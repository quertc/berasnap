use anyhow::Result;
use google_cloud_storage::{
    client::Client,
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
    http::resumable_upload_client::{ChunkSize, UploadStatus},
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek};
use serde::{Serialize, Deserialize};

const CHUNK_SIZE: usize = 8 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct UploadState {
    url: String,
    uploaded: u64,
}

pub async fn upload_to_gcs(client: &Client, bucket: &str, file_name: &str) -> Result<()> {
    let object_name = format!("berachain/snapshots/{}", file_name);
    let state_file = format!("{}.upload_state", file_name);

    let (uploader, mut uploaded) = if let Ok(state) = read_state(&state_file) {
        println!("Resuming previous upload");
        (client.get_resumable_upload(state.url), state.uploaded)
    } else {
        println!("Starting new upload");
        let media = Media::new(object_name);
        let upload_type = UploadType::Simple(media);
        let upload_request = UploadObjectRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };
        let uploader = client.prepare_resumable_upload(&upload_request, &upload_type).await?;
        (uploader, 0)
    };

    let mut file = File::open(file_name)?;
    let file_size = file.metadata()?.len();

    file.seek(std::io::SeekFrom::Start(uploaded))?;

    loop {
        let mut buffer = vec![0; CHUNK_SIZE];
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        buffer.truncate(n);

        let chunk = ChunkSize::new(uploaded, uploaded + n as u64 - 1, Some(file_size));
        let status = uploader.upload_multiple_chunk(buffer, &chunk).await?;

        uploaded += n as u64;

        println!("Uploaded {} of {} bytes for {}", uploaded, file_size, file_name);

        save_state(&state_file, &UploadState { url: uploader.url().to_string(), uploaded })?;

        if let UploadStatus::Ok(_) = status {
            break;
        }
    }

    println!("Upload completed successfully for {}", file_name);
    std::fs::remove_file(state_file)?;

    Ok(())
}

fn save_state(file_name: &str, state: &UploadState) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_name)?;
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

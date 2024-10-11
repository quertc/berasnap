use anyhow::Result;
use bytes::Bytes;
use log::info;
use object_store::{gcp::GoogleCloudStorage, path::Path, ObjectStore, WriteMultipart};
use std::{
    fs::File,
    io::{BufReader, Read},
    time::Instant,
};

const CHUNK_SIZE: usize = 128 * 1024 * 1024;
const MAX_CONCURRENT_UPLOADS: usize = 12;

pub async fn upload_to_gcs(gcs: &GoogleCloudStorage, folder: &str, file_name: &str) -> Result<()> {
    let start_time = Instant::now();
    info!("Starting upload for file: {}", file_name);

    let object_name = format!("{}/{}", folder, file_name);
    let path = Path::from(object_name);

    let file = File::open(file_name)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);

    let multipart = gcs.put_multipart(&path).await?;
    let mut write = WriteMultipart::new_with_chunk_size(multipart, CHUNK_SIZE);

    let mut uploaded = 0;
    let mut last_log_time = Instant::now();

    loop {
        write.wait_for_capacity(MAX_CONCURRENT_UPLOADS).await?;

        let mut buffer = vec![0; CHUNK_SIZE];
        let n = reader.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        buffer.truncate(n);
        write.put(Bytes::from(buffer));
        uploaded += n as u64;

        if last_log_time.elapsed().as_secs() >= 300 {
            info!(
                "Upload progress: {:.2}% for {}",
                (uploaded as f64 / file_size as f64) * 100.0,
                file_name
            );
            last_log_time = Instant::now();
        }
    }

    write.finish().await?;

    let duration = start_time.elapsed();
    info!(
        "Upload completed successfully for {} in {:?}",
        file_name, duration
    );

    Ok(())
}

use anyhow::{anyhow, Result};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use log::{info, warn};
use object_store::{
    gcp::GoogleCloudStorage, path::Path, Attribute, Attributes, ObjectStore, PutOptions,
    WriteMultipart,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{BufReader, Read},
    time::Instant,
};

const CHUNK_SIZE: usize = 128 * 1024 * 1024;
const MAX_CONCURRENT_UPLOADS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Beacond,
    Reth,
}

impl NodeType {
    fn as_str(&self) -> &'static str {
        match self {
            NodeType::Beacond => "beacond",
            NodeType::Reth => "reth",
        }
    }
}

pub async fn upload_to_gcs(
    gcs: &GoogleCloudStorage,
    bucket_name: &str,
    folder: &str,
    file_name: &str,
    node_type: NodeType,
    keep: usize,
) -> Result<()> {
    let start_time = Instant::now();
    info!("Starting upload for file: {}", file_name);

    let object_name = format!("{}/{}", folder, file_name);
    let object_uri = format!("{}/{}", bucket_name, object_name);
    let path = Path::from(object_name);

    if gcs.head(&path).await.is_ok() {
        warn!("File {} already exists in GCS. Skipping upload.", file_name);
        return Ok(());
    }

    let file = File::open(file_name)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);

    let multipart = gcs.put_multipart(&path).await?;
    let mut write = WriteMultipart::new_with_chunk_size(multipart, CHUNK_SIZE);

    let mut uploaded = 0;
    let mut last_log_time = Instant::now();
    let mut hasher = Sha256::new();

    loop {
        write.wait_for_capacity(MAX_CONCURRENT_UPLOADS).await?;

        let mut buffer = vec![0; CHUNK_SIZE];
        let n = reader.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        buffer.truncate(n);
        hasher.update(&buffer);
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

    let hash = format!("{:x}", hasher.finalize());
    update_json_metadata(
        gcs,
        bucket_name,
        folder,
        &object_uri,
        &hash,
        node_type,
        keep,
    )
    .await?;

    Ok(())
}

async fn update_json_metadata(
    gcs: &GoogleCloudStorage,
    bucket_name: &str,
    folder: &str,
    object_uri: &str,
    hash: &str,
    node_type: NodeType,
    keep: usize,
) -> Result<()> {
    let json_path = Path::from(format!("{}/metadata.json", folder));

    let mut metadata: Value = match gcs.get(&json_path).await {
        Ok(data) => {
            let bytes = data.bytes().await?;
            serde_json::from_slice(&bytes)?
        }
        Err(_) => json!({
            "snapshots": []
        }),
    };

    let new_entry = json!({
        "fileName": object_uri,
        "sha256": hash,
        "type": node_type.as_str(),
        "uploadTime": chrono::Utc::now().to_rfc3339()
    });

    let snapshots = metadata["snapshots"]
        .as_array_mut()
        .ok_or_else(|| anyhow!("Invalid metadata structure"))?;
    snapshots.push(new_entry);

    snapshots.sort_by(|a, b| {
        let a_time: DateTime<Utc> = a["uploadTime"].as_str().unwrap().parse().unwrap();
        let b_time: DateTime<Utc> = b["uploadTime"].as_str().unwrap().parse().unwrap();
        b_time.cmp(&a_time)
    });

    let mut beacond_count = 0;
    let mut reth_count = 0;
    let mut to_delete = Vec::new();

    snapshots.retain(|snapshot| {
        let snapshot_type = snapshot["type"].as_str().unwrap();
        let keep_snapshot = match snapshot_type {
            "beacond" => {
                beacond_count += 1;
                beacond_count <= keep
            }
            "reth" => {
                reth_count += 1;
                reth_count <= keep
            }
            _ => false,
        };

        if !keep_snapshot {
            to_delete.push(snapshot["fileName"].as_str().unwrap().to_string());
        }

        keep_snapshot
    });

    let json_content = serde_json::to_string_pretty(&metadata)?;

    let mut attributes = Attributes::new();
    attributes.insert(Attribute::ContentType, "application/json".into());
    let put_options = PutOptions::from(attributes);

    gcs.put_opts(&json_path, json_content.into(), put_options)
        .await?;

    info!(
        "Updated metadata.json with information about {}",
        object_uri
    );

    for file_to_delete in to_delete {
        let delete_path =
            Path::from(file_to_delete.trim_start_matches(&format!("{}/", bucket_name)));
        if let Err(e) = gcs.delete(&delete_path).await {
            warn!(
                "Failed to delete file from GCS: {}. Error: {}",
                file_to_delete, e
            );
        } else {
            info!("Deleted excess snapshot from GCS: {}", file_to_delete);
        }
    }

    Ok(())
}

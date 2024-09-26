use anyhow::Result;
use chrono::Local;
use google_cloud_storage::{
    client::Client,
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
};

pub async fn upload_to_gcs(client: &Client, bucket: &str, name: &str) -> Result<()> {
    let date = Local::now().format("%d-%m-%y").to_string();
    let file_name = format!("{}_{}.tar.lz4", name, date);

    let upload_type = UploadType::Simple(Media::new(file_name));
    let upload_request = UploadObjectRequest {
        bucket: bucket.to_string(),
        ..Default::default()
    };

    client
        .upload_object(&upload_request, "snapshot".as_bytes(), &upload_type)
        .await?;

    Ok(())
}

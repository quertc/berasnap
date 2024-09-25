use anyhow::Result;
use chrono::Local;
use compose_rs::{Compose, ComposeCommand};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use lz4::EncoderBuilder;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;
use tar::Builder;
use tokio::fs::File as TokioFile;
use walkdir::WalkDir;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    Start(StartOpt),
}

#[derive(StructOpt)]
pub struct StartOpt {
    #[structopt(long, env = "DOCKER_COMPOSE_FILE")]
    pub path: String,

    #[structopt(long, env = "CRON_JOB_TIME")]
    pub job_time: String,

    #[structopt(long, env = "GCS_BUCKET")]
    pub gcs_bucket: String,
}

async fn create_snapshot(compose_path: &str, gcs_bucket: &str) -> Result<()> {
    // let compose = Compose::builder().path(compose_path).build()?;
    // compose.down().exec()?;

    create_tar_lz4(
        compose_path,
        "pruned_snapshot",
        &["./data/beacond/data"],
        &["priv_validator_state.json"],
    )?;
    create_tar_lz4(
        compose_path,
        "reth_snapshot",
        &["./data/reth/static_files", "./data/reth/db"],
        &[],
    )?;

    // compose.up().exec()?;

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);
    // upload_to_gcs(&client, gcs_bucket, "pruned_snapshot").await?;
    // upload_to_gcs(&client, gcs_bucket, "reth_snapshot").await?;

    Ok(())
}


fn create_tar_lz4(
    base_path: &str,
    name: &str,
    include_paths: &[&str],
    exclude_files: &[&str],
) -> Result<()> {
    let date = Local::now().format("%d-%m-%y").to_string();
    let file_name = format!("{}_{}.tar.lz4", name, date);
    let output_file = File::create(&file_name)?;
    let mut encoder = EncoderBuilder::new().build(output_file)?;
    let mut tar = Builder::new(Vec::new());

    for include_path in include_paths {
        let full_path = Path::new(base_path).join(include_path);
        add_to_tar(&mut tar, &full_path, include_path, exclude_files)?;
    }

    let tar_data = tar.into_inner()?;
    encoder.write_all(&tar_data)?;
    encoder.finish();
    Ok(())
}

fn add_to_tar<W: Write>(
    tar: &mut Builder<W>,
    full_path: &Path,
    base_path: &str,
    exclude_files: &[&str],
) -> Result<()> {
    let walker = WalkDir::new(full_path).into_iter();

    for entry in walker.filter_entry(|e| 
        !exclude_files.iter().any(|ex| e.path().to_str().unwrap().ends_with(ex))
    ) {
        let entry = entry?;
        let path = entry.path();
        if path == full_path {
            continue;
        }

        let relative = path.strip_prefix(full_path)?;
        let tar_path = Path::new(base_path).join(relative);

        if path.is_file() {
            let mut file = File::open(path)?;
            tar.append_file(tar_path, &mut file)?;
        } else if path.is_dir() {
            tar.append_dir(tar_path, path)?;
        }
    }
    Ok(())
}

async fn upload_to_gcs(client: &Client, bucket: &str, name: &str) -> Result<()> {
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

pub async fn start_scheduler(opt: StartOpt) -> Result<()> {
    let mut sched = JobScheduler::new().await?;

    let job = Job::new_async(opt.job_time.as_str(), move |_uuid, _l| {
        let path = opt.path.clone();
        let bucket = opt.gcs_bucket.clone();
        Box::pin(async move {
            if let Err(e) = create_snapshot(&path, &bucket).await {
                eprintln!("Error during snapshot creation and upload: {}", e);
            }
        })
    })?;

    sched.add(job).await?;
    sched.start().await?;

    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Start(start_opt) => {
            start_scheduler(start_opt).await?;
        }
    }

    Ok(())
}

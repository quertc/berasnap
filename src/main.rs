use anyhow::Result;
use cli::{Command, Opt, StartOpt};
use compose_rs::{Compose, ComposeCommand};
use gcs::upload_to_gcs;
use google_cloud_storage::client::{Client, ClientConfig};
use structopt::StructOpt;
use tar::create_tar_lz4;
use tokio_cron_scheduler::{Job, JobScheduler};

mod cli;
mod gcs;
mod tar;

async fn create_snapshot(node_path: &str, gcs_bucket: &str) -> Result<()> {
    let compose_path = format!("{}/docker-compose.yml", node_path);
    let compose = Compose::builder().path(compose_path).build()?;
    compose.down().exec()?;

    let beacond_file_name = create_tar_lz4(
        node_path,
        "pruned_snapshot",
        &["./data/beacond/data"],
        &["priv_validator_state.json"],
    )?;
    let reth_file_name = create_tar_lz4(
        node_path,
        "reth_snapshot",
        &["./data/reth/static_files", "./data/reth/db"],
        &[],
    )?;

    compose.up().exec()?;

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);
    upload_to_gcs(&client, gcs_bucket, &beacond_file_name).await?;
    upload_to_gcs(&client, gcs_bucket, &reth_file_name).await?;

    Ok(())
}

pub async fn start_scheduler(opt: StartOpt) -> Result<()> {
    let sched = JobScheduler::new().await?;

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

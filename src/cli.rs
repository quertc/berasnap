use structopt::StructOpt;

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
    /// Path to the Berachain Docker node
    #[structopt(long, env = "NODE_PATH")]
    pub path: String,

    /// Cron job schedule for taking snapshots
    #[structopt(long, env = "CRON_JOB_TIME")]
    pub job_time: String,

    /// Enable Google Cloud Storage upload
    #[structopt(short, long)]
    pub gcs: bool,

    /// GCS bucket name (required if `--gcs` is set)
    #[structopt(long, env = "GCS_BUCKET", required_if("gcs", "true"))]
    pub gcs_bucket: Option<String>,

    /// GCS folder path (required if `--gcs` is set)
    #[structopt(long, env = "GCS_FOLDER", required_if("gcs", "true"))]
    pub gcs_folder: Option<String>,
}

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

    /// Enable API for snapshot distribution
    #[structopt(long, short)]
    pub api: bool,

    /// API server port (required if `--api` is set)
    #[structopt(long, env = "API_PORT", default_value = "3050", required_if("api", "true"))]
    pub api_port: u16,

    /// Path to store local snapshots
    #[structopt(long, env = "STORAGE_PATH", default_value = "storage")]
    pub storage_path: String,

    /// Enable Google Cloud Storage upload
    #[structopt(short, long)]
    pub gcs: bool,

    /// GCS bucket name (required if `--gcs` is set)
    #[structopt(long, env = "GCS_BUCKET", required_if("gcs", "true"))]
    pub gcs_bucket: Option<String>,

    /// GCS folder path (required if `--gcs` is set)
    #[structopt(long, env = "GCS_FOLDER", required_if("gcs", "true"))]
    pub gcs_folder: Option<String>,

    /// Number of snapshots to keep
    #[structopt(long, env = "SNAPS_KEEP", default_value = "1")]
    pub keep: usize,
}

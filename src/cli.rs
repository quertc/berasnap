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
    #[structopt(long, env = "NODE_PATH")]
    pub path: String,

    #[structopt(long, env = "CRON_JOB_TIME")]
    pub job_time: String,

    #[structopt(long, env = "GCS_BUCKET")]
    pub gcs_bucket: String,
}

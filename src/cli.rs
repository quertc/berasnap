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
    /// Path to the docker-compose file
    #[structopt(long, env = "DOCKER_COMPOSE_FILE")]
    pub path: String,

    /// The time to run the cron job
    #[structopt(long, env = "CRON_JOB_TIME")]
    pub job_time: String,
}

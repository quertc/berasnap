use std::time::Duration;

use crate::cli::StartOpt;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};


pub async fn start_scheduler(opt: StartOpt) -> Result<(), JobSchedulerError> {
    let mut sched = JobScheduler::new().await?;

    let job = Job::new(opt.job_time.as_str(), |_uuid, _l| {
        println!("I run every 10 seconds");
    })?;

    let jj = Job::new_repeated(Duration::from_secs(8), |_uuid, _l| {
        println!("I run repeatedly every 8 seconds");
    })?;


    sched.add(job).await?;
    sched.add(jj).await?;

    sched.start().await?;

    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}

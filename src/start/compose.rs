use anyhow::Result;
use compose_rs::{Compose, ComposeCommand};

async fn _x(path: String) -> Result<()> {
    let compose = Compose::builder().path(path).build()?;
    compose.down().exec()?;

    Ok(())
}

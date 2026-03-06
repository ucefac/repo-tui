//! repotui - A TUI tool for browsing and managing GitHub repositories

use anyhow::Result;
use repotui::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

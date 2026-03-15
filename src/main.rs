//! repo-tui - A TUI tool for browsing and managing GitHub repositories

use anyhow::Result;
use repo_tui::run;

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}

mod window_manager;
mod event_loop;
mod utils;

use anyhow::Result;
use tracing::{info, error};

fn main() -> Result<()> {
    // Initialize logging
    initialize_logging();
    
    info!("Starting Tiling Window Manager v0.1.0");
    
    Ok(())
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_core=debug,info")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();
}

// These modules are not yet used in the binary, but are part of the library API
#[allow(dead_code, unused_imports)]
mod window_manager;
#[allow(dead_code, unused_imports)]
mod event_loop;
#[allow(dead_code, unused_imports)]
mod utils;

use anyhow::Result;
use tracing::info;

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

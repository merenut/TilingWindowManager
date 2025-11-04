//! Event Loop Demo
//!
//! This example demonstrates the Windows Event Loop by monitoring and displaying
//! window events in real-time. It requires Windows to run.
//!
//! # Usage
//!
//! ```bash
//! cargo run -p tiling-wm-core --example event_loop_demo
//! ```
//!
//! Then open, close, minimize, or focus windows to see events being detected.
//! Press Ctrl+C to stop the demo.

fn main() -> anyhow::Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║     Windows Event Loop Demo            ║");
    println!("╚════════════════════════════════════════╝");
    println!();
    
    #[cfg(not(target_os = "windows"))]
    {
        println!("⚠️  This demo requires Windows to run.");
        println!("   The event loop is only functional on Windows platforms.");
        return Ok(());
    }
    
    #[cfg(target_os = "windows")]
    {
        use tiling_wm_core::event_loop::EventLoop;
        use std::time::Duration;
        println!("📋 Starting event loop...");
        
        let mut event_loop = EventLoop::new();
        
        match event_loop.start() {
            Ok(_) => {
                println!("✅ Event loop started successfully!");
                println!();
                println!("📝 Instructions:");
                println!("   - Open new applications to see WindowCreated events");
                println!("   - Close applications to see WindowDestroyed events");
                println!("   - Switch focus between windows to see WindowFocused events");
                println!("   - Move or resize windows to see WindowMoved events");
                println!("   - Minimize/restore windows to see WindowHidden/WindowRestored events");
                println!();
                println!("⏸️  Press Ctrl+C to stop the demo");
                println!();
                println!("═══════════════════════════════════════════════════════════");
                println!();
                
                let mut event_count = 0;
                let start_time = std::time::Instant::now();
                
                loop {
                    // Process Windows messages
                    event_loop.process_messages()?;
                    
                    // Poll for events
                    for event in event_loop.poll_events() {
                        event_count += 1;
                        let elapsed = start_time.elapsed().as_secs();
                        println!("[{:>4}s] [{:>5}] {:?}", elapsed, event_count, event);
                    }
                    
                    // Small sleep to prevent high CPU usage
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            Err(e) => {
                println!("❌ Failed to start event loop: {}", e);
                println!();
                println!("💡 Troubleshooting:");
                println!("   - Make sure you're running on Windows");
                println!("   - Try running as Administrator");
                println!("   - Check Windows Event Viewer for errors");
                return Err(e);
            }
        }
    }
}

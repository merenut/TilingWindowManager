use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;

#[cfg(windows)]
use anyhow::Context;
#[cfg(windows)]
use comfy_table::{presets::UTF8_FULL, Table};
#[cfg(windows)]
use serde_json::Value;
#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[derive(Parser)]
#[command(name = "twm")]
#[command(about = "Tiling Window Manager CLI", long_about = None)]
#[command(version)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// Named pipe path
    #[arg(long, default_value = r"\\.\pipe\tenraku")]
    pipe: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Compact,
}

#[derive(Subcommand)]
enum Commands {
    /// Get list of windows
    Windows {
        /// Filter by workspace
        #[arg(short, long)]
        workspace: Option<usize>,
    },

    /// Get active window information
    ActiveWindow,

    /// Get list of workspaces
    Workspaces,

    /// Get list of monitors
    Monitors,

    /// Get configuration info
    Config,

    /// Get version information
    Version,

    /// Execute a command
    #[command(subcommand)]
    Exec(ExecCommands),

    /// Switch to workspace
    Workspace {
        /// Workspace ID
        id: usize,
    },

    /// Close active or specified window
    Close {
        /// Window HWND (hex or decimal)
        #[arg(short, long)]
        window: Option<String>,
    },

    /// Focus a window
    Focus {
        /// Window HWND (hex or decimal)
        window: String,
    },

    /// Move window to workspace
    Move {
        /// Window HWND
        window: String,
        /// Target workspace ID
        workspace: usize,
    },

    /// Toggle floating for active or specified window
    ToggleFloat {
        /// Window HWND
        #[arg(short, long)]
        window: Option<String>,
    },

    /// Toggle fullscreen for active or specified window
    ToggleFullscreen {
        /// Window HWND
        #[arg(short, long)]
        window: Option<String>,
    },

    /// Create a new workspace
    CreateWorkspace {
        /// Workspace name
        name: String,
        /// Monitor ID
        #[arg(short, long, default_value = "0")]
        monitor: usize,
    },

    /// Delete a workspace
    DeleteWorkspace {
        /// Workspace ID
        id: usize,
    },

    /// Rename a workspace
    RenameWorkspace {
        /// Workspace ID
        id: usize,
        /// New name
        name: String,
    },

    /// Set layout
    Layout {
        /// Layout name (dwindle, master)
        name: String,
    },

    /// Reload configuration
    Reload,

    /// Subscribe to events (listen mode)
    Listen {
        /// Events to subscribe to
        #[arg(short, long, value_delimiter = ',')]
        events: Vec<String>,
    },

    /// Ping the server
    Ping,
}

#[derive(Subcommand)]
enum ExecCommands {
    /// Adjust master factor
    MasterFactor {
        /// Delta value
        delta: f32,
    },

    /// Increase master count
    IncreaseMaster,

    /// Decrease master count
    DecreaseMaster,
}

fn main() -> Result<()> {
    // Platform check
    #[cfg(not(windows))]
    {
        eprintln!("{}", "Error: This CLI tool only works on Windows.".red());
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let cli = Cli::parse();

        // Connect to named pipe
        let mut client = connect_to_pipe(&cli.pipe)
            .context("Failed to connect to window manager. Is it running?")?;

        // Build request
        let request = build_request(&cli.command)?;

        // Send request
        send_request(&mut client, &request)?;

        // Handle response based on command type
        match cli.command {
            Commands::Listen { .. } => {
                // Listen mode: keep receiving events
                loop {
                    let response = receive_response(&mut client)?;
                    print_response(&response, cli.format);
                }
            }
            _ => {
                // Single request: receive one response
                let response = receive_response(&mut client)?;
                print_response(&response, cli.format);
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
fn connect_to_pipe(pipe_path: &str) -> Result<std::fs::File> {
    // Windows-specific constant for FILE_FLAG_OVERLAPPED
    const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_OVERLAPPED)
        .open(pipe_path)
        .context("Failed to open named pipe")
}

#[cfg(windows)]
fn build_request(command: &Commands) -> Result<Value> {
    let request = match command {
        Commands::Windows { workspace } => {
            serde_json::json!({
                "type": "get_windows",
                "workspace": workspace,
            })
        }
        Commands::ActiveWindow => {
            serde_json::json!({
                "type": "get_active_window"
            })
        }
        Commands::Workspaces => {
            serde_json::json!({
                "type": "get_workspaces"
            })
        }
        Commands::Monitors => {
            serde_json::json!({
                "type": "get_monitors"
            })
        }
        Commands::Config => {
            serde_json::json!({
                "type": "get_config"
            })
        }
        Commands::Version => {
            serde_json::json!({
                "type": "get_version"
            })
        }
        Commands::Workspace { id } => {
            serde_json::json!({
                "type": "switch_workspace",
                "id": id,
            })
        }
        Commands::Close { window } => {
            serde_json::json!({
                "type": "close_window",
                "hwnd": window,
            })
        }
        Commands::Focus { window } => {
            serde_json::json!({
                "type": "focus_window",
                "hwnd": window,
            })
        }
        Commands::Move { window, workspace } => {
            serde_json::json!({
                "type": "move_window",
                "hwnd": window,
                "workspace": workspace,
            })
        }
        Commands::ToggleFloat { window } => {
            serde_json::json!({
                "type": "toggle_floating",
                "hwnd": window,
            })
        }
        Commands::ToggleFullscreen { window } => {
            serde_json::json!({
                "type": "toggle_fullscreen",
                "hwnd": window,
            })
        }
        Commands::CreateWorkspace { name, monitor } => {
            serde_json::json!({
                "type": "create_workspace",
                "name": name,
                "monitor": monitor,
            })
        }
        Commands::DeleteWorkspace { id } => {
            serde_json::json!({
                "type": "delete_workspace",
                "id": id,
            })
        }
        Commands::RenameWorkspace { id, name } => {
            serde_json::json!({
                "type": "rename_workspace",
                "id": id,
                "name": name,
            })
        }
        Commands::Layout { name } => {
            serde_json::json!({
                "type": "set_layout",
                "layout": name,
            })
        }
        Commands::Reload => {
            serde_json::json!({
                "type": "reload_config"
            })
        }
        Commands::Listen { events } => {
            serde_json::json!({
                "type": "subscribe",
                "events": events,
            })
        }
        Commands::Ping => {
            serde_json::json!({
                "type": "ping"
            })
        }
        Commands::Exec(exec_cmd) => match exec_cmd {
            ExecCommands::MasterFactor { delta } => {
                serde_json::json!({
                    "type": "adjust_master_factor",
                    "delta": delta,
                })
            }
            ExecCommands::IncreaseMaster => {
                serde_json::json!({
                    "type": "increase_master_count"
                })
            }
            ExecCommands::DecreaseMaster => {
                serde_json::json!({
                    "type": "decrease_master_count"
                })
            }
        },
    };

    Ok(request)
}

#[cfg(windows)]
fn send_request<W>(writer: &mut W, request: &Value) -> Result<()>
where
    W: Write,
{
    let data = serde_json::to_vec(request)?;
    let len = data.len() as u32;

    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&data)?;
    writer.flush()?;

    Ok(())
}

#[cfg(windows)]
fn receive_response<R>(reader: &mut R) -> Result<Value>
where
    R: Read,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;

    let mut data = vec![0u8; len];
    reader.read_exact(&mut data)?;

    let response: Value = serde_json::from_slice(&data)?;
    Ok(response)
}

#[cfg(windows)]
fn print_response(response: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(response).unwrap());
        }
        OutputFormat::Table => {
            print_table(response);
        }
        OutputFormat::Compact => {
            print_compact(response);
        }
    }
}

#[cfg(windows)]
fn print_table(response: &Value) {
    // Check response type
    if let Some(response_type) = response.get("type").and_then(|t| t.as_str()) {
        match response_type {
            "success" => {
                if let Some(data) = response.get("data") {
                    // Try to format as table if it's an array
                    if let Some(arr) = data.as_array() {
                        if !arr.is_empty() {
                            // Check what type of data this is
                            if let Some(first) = arr.first() {
                                if first.get("id").is_some() && first.get("name").is_some() {
                                    // Workspace data
                                    print_workspace_table(arr);
                                } else if first.get("hwnd").is_some() {
                                    // Window data
                                    print_window_table(arr);
                                } else if first.get("width").is_some() {
                                    // Monitor data
                                    print_monitor_table(arr);
                                } else {
                                    // Generic array
                                    println!(
                                        "{}: {}",
                                        "Success".green(),
                                        serde_json::to_string_pretty(data).unwrap()
                                    );
                                }
                            }
                        } else {
                            println!("{}: {}", "Success".green(), "No data".yellow());
                        }
                    } else if data.is_object() {
                        // Single object - format nicely
                        println!("{}", "Success".green());
                        if let Some(obj) = data.as_object() {
                            for (key, value) in obj {
                                println!("  {}: {}", key.cyan(), format_value(value));
                            }
                        }
                    } else {
                        println!("{}: {}", "Success".green(), data);
                    }
                } else {
                    println!("{}", "Success".green());
                }
            }
            "error" => {
                let message = response
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("{}: {}", "Error".red(), message);
            }
            "event" => {
                let name = response
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let data = response.get("data").unwrap_or(&Value::Null);
                println!(
                    "{} {}: {}",
                    "Event".cyan(),
                    name.bright_cyan(),
                    format_value(data)
                );
            }
            "pong" => {
                println!("{}", "Pong".green());
            }
            _ => {
                println!("{}", serde_json::to_string_pretty(response).unwrap());
            }
        }
    }
}

#[cfg(windows)]
fn print_workspace_table(workspaces: &[Value]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["ID", "Name", "Monitor", "Windows", "Active"]);

    for ws in workspaces {
        if let (Some(id), Some(name), Some(monitor), Some(window_count)) = (
            ws.get("id").and_then(|v| v.as_u64()),
            ws.get("name").and_then(|v| v.as_str()),
            ws.get("monitor").and_then(|v| v.as_u64()),
            ws.get("window_count").and_then(|v| v.as_u64()),
        ) {
            let active = ws.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
            table.add_row(vec![
                id.to_string(),
                name.to_string(),
                monitor.to_string(),
                window_count.to_string(),
                if active {
                    "✓".green().to_string()
                } else {
                    " ".to_string()
                },
            ]);
        }
    }

    println!("{}", table);
}

#[cfg(windows)]
fn print_window_table(windows: &[Value]) {
    const MAX_TITLE_LENGTH: usize = 40;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["HWND", "Title", "Workspace", "State", "Focused"]);

    for win in windows {
        if let (Some(hwnd), Some(title), Some(workspace), Some(state)) = (
            win.get("hwnd").and_then(|v| v.as_str()),
            win.get("title").and_then(|v| v.as_str()),
            win.get("workspace").and_then(|v| v.as_u64()),
            win.get("state").and_then(|v| v.as_str()),
        ) {
            let focused = win
                .get("focused")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let title_truncated: String = title.chars().take(MAX_TITLE_LENGTH).collect();

            table.add_row(vec![
                hwnd.to_string(),
                title_truncated,
                workspace.to_string(),
                state.to_string(),
                if focused {
                    "✓".green().to_string()
                } else {
                    " ".to_string()
                },
            ]);
        }
    }

    println!("{}", table);
}

#[cfg(windows)]
fn print_monitor_table(monitors: &[Value]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        "ID",
        "Name",
        "Resolution",
        "Position",
        "Scale",
        "Primary",
    ]);

    for mon in monitors {
        if let (Some(id), Some(name), Some(width), Some(height), Some(x), Some(y), Some(scale)) = (
            mon.get("id").and_then(|v| v.as_u64()),
            mon.get("name").and_then(|v| v.as_str()),
            mon.get("width").and_then(|v| v.as_i64()),
            mon.get("height").and_then(|v| v.as_i64()),
            mon.get("x").and_then(|v| v.as_i64()),
            mon.get("y").and_then(|v| v.as_i64()),
            mon.get("scale").and_then(|v| v.as_f64()),
        ) {
            let primary = mon
                .get("primary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            table.add_row(vec![
                id.to_string(),
                name.to_string(),
                format!("{}x{}", width, height),
                format!("{},{}", x, y),
                format!("{:.2}", scale),
                if primary {
                    "✓".green().to_string()
                } else {
                    " ".to_string()
                },
            ]);
        }
    }

    println!("{}", table);
}

#[cfg(windows)]
fn print_compact(response: &Value) {
    if let Some(response_type) = response.get("type").and_then(|t| t.as_str()) {
        match response_type {
            "success" => {
                if let Some(data) = response.get("data") {
                    println!("{}", serde_json::to_string(data).unwrap());
                } else {
                    println!("ok");
                }
            }
            "error" => {
                let message = response
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("error");
                eprintln!("{}", message);
            }
            "event" => {
                println!("{}", serde_json::to_string(response).unwrap());
            }
            "pong" => {
                println!("pong");
            }
            _ => {
                println!("{}", serde_json::to_string(response).unwrap());
            }
        }
    }
}

#[cfg(windows)]
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "?".to_string())
        }
    }
}

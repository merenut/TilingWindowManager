use dioxus::prelude::*;
use std::process;
use tracing::{error, info};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

mod commands;
mod ipc_client;
mod recent;
mod scanner;

use commands::{CommandCatalog, CommandEntry};
use ipc_client::IpcClient;
use recent::{ItemType, RecentItems};
use scanner::{ExecutableEntry, Scanner};

const WINDOW_WIDTH: i32 = 800;
// Exact height: search (94) + header (29) + 1 result (60) + footer (41) + borders (4) = 228px
const WINDOW_HEIGHT: i32 = 230;
const MAX_RESULTS: usize = 10;

#[derive(Clone, Debug, PartialEq)]
enum ResultItem {
    Command(CommandEntry),
    Executable(ExecutableEntry),
}

impl ResultItem {
    fn display_name(&self) -> String {
        match self {
            ResultItem::Command(cmd) => cmd.display_name.clone(),
            ResultItem::Executable(exe) => {
                if exe.show_path {
                    format!("{} - {}", exe.name, exe.full_path.display())
                } else {
                    exe.name.clone()
                }
            }
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ResultItem::Command(_) => "⚡",
            ResultItem::Executable(_) => "🖥️",
        }
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Tenraku Command Palette");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let parent_hwnd = if args.len() > 2 && args[1] == "--parent-hwnd" {
        args[2].parse::<isize>().ok().map(|h| HWND(h as _))
    } else {
        None
    };

    // Get cursor position
    let (x, y) = get_cursor_position();
    let (window_x, window_y) = calculate_window_position(x, y);

    info!(
        "Positioning window at ({}, {}) near cursor ({}, {})",
        window_x, window_y, x, y
    );

    // Configure window
    let window_builder = dioxus::desktop::WindowBuilder::new()
        .with_title("Tenraku Command Palette")
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top(true)
        .with_resizable(false)
        .with_inner_size(dioxus::desktop::LogicalSize::new(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        ))
        .with_position(dioxus::desktop::LogicalPosition::new(window_x, window_y));

    // Note: Parent window support may be added in future Dioxus versions
    // For now, the palette works as a standalone always-on-top window
    let _ = parent_hwnd; // Suppress unused warning

    let config = dioxus::desktop::Config::new().with_window(window_builder);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

fn get_cursor_position() -> (i32, i32) {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);
        (point.x, point.y)
    }
}

fn calculate_window_position(cursor_x: i32, cursor_y: i32) -> (i32, i32) {
    unsafe {
        use windows::Win32::Graphics::Gdi::GetMonitorInfoW;

        // Get monitor info for the cursor position
        let point = POINT {
            x: cursor_x,
            y: cursor_y,
        };
        let monitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);

        let mut monitor_info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
            let work_area = monitor_info.rcWork;

            // Center window on cursor
            let mut x = cursor_x - WINDOW_WIDTH / 2;
            let mut y = cursor_y - WINDOW_HEIGHT / 2;

            // Clamp to screen bounds
            x = x.max(work_area.left);
            x = x.min(work_area.right - WINDOW_WIDTH);
            y = y.max(work_area.top);
            y = y.min(work_area.bottom - WINDOW_HEIGHT);

            (x, y)
        } else {
            // Fallback: center on cursor without bounds checking
            (cursor_x - WINDOW_WIDTH / 2, cursor_y - WINDOW_HEIGHT / 2)
        }
    }
}

#[component]
fn App() -> Element {
    // Initialize scanner and start scan
    let scanner = use_signal(|| {
        let s = Scanner::new();
        s.start_scan();
        s
    });

    // Initialize command catalog
    let command_catalog = use_signal(|| CommandCatalog::new());

    // Initialize recent items
    let recent_items = use_signal(|| {
        RecentItems::new().unwrap_or_else(|e| {
            error!("Failed to load recent items: {}", e);
            RecentItems::new().unwrap()
        })
    });

    // Search query
    let mut query = use_signal(|| String::new());
    let mut selected_index = use_signal(|| 0usize);

    // Get window handle for resizing
    let window = dioxus::window();

    // Calculate results
    let results = use_memo(move || {
        let q = query.read().clone();

        if q.is_empty() {
            // Show recent items
            recent_items
                .read()
                .get_recent(MAX_RESULTS)
                .iter()
                .filter_map(|entry| match entry.item_type {
                    ItemType::Command => {
                        // Try to find command by display name
                        command_catalog
                            .read()
                            .all_commands()
                            .iter()
                            .find(|cmd| cmd.display_name == entry.name)
                            .map(|cmd| ResultItem::Command(cmd.clone()))
                    }
                    ItemType::Executable => Some(ResultItem::Executable(ExecutableEntry {
                        name: entry.name.clone(),
                        full_path: std::path::PathBuf::from(&entry.name),
                        show_path: false,
                    })),
                })
                .collect::<Vec<_>>()
        } else {
            // Search both commands and executables
            let mut all_results: Vec<(ResultItem, i64)> = Vec::new();

            // Search commands
            for (cmd, score) in command_catalog.read().search(&q) {
                all_results.push((ResultItem::Command(cmd), score));
            }

            // Search executables
            for (exe, score) in scanner.read().search(&q) {
                all_results.push((ResultItem::Executable(exe), score));
            }

            // Sort by score descending and take top results
            all_results.sort_by(|a, b| b.1.cmp(&a.1));
            all_results
                .into_iter()
                .take(MAX_RESULTS)
                .map(|(item, _)| item)
                .collect()
        }
    });

    // Keyboard event handler
    let on_key_down = move |evt: Event<KeyboardData>| {
        let key = evt.data.key();
        let results_list = results.read();
        let result_count = results_list.len();

        match key {
            Key::Escape => {
                process::exit(0);
            }
            Key::ArrowUp => {
                if result_count > 0 {
                    let current = *selected_index.read();
                    *selected_index.write() = if current == 0 {
                        result_count - 1
                    } else {
                        current - 1
                    };
                }
            }
            Key::ArrowDown => {
                if result_count > 0 {
                    let current = *selected_index.read();
                    *selected_index.write() = (current + 1) % result_count;
                }
            }
            Key::Enter => {
                if result_count > 0 {
                    let index = *selected_index.read();
                    if let Some(item) = results_list.get(index) {
                        execute_item(item.clone(), recent_items);
                    }
                }
            }
            _ => {}
        }
    };

    let is_scanning = scanner.read().is_scanning();

    // Dynamically resize window based on content
    let result_count = results.read().len();
    use_effect(move || {
        // Base: search section (94) + padding (20)
        let base_height = 114;
        // Each result: 60px, section header: 29px, footer: 41px
        let header_height = if query.read().is_empty() && result_count > 0 {
            29
        } else {
            0
        };
        let results_height = result_count.min(MAX_RESULTS) * 60;
        let footer_height = if is_scanning { 41 } else { 0 };
        let total_height = base_height + header_height + results_height + footer_height;

        let _ = window.set_inner_size(dioxus::desktop::LogicalSize::new(
            WINDOW_WIDTH as u32,
            total_height as u32,
        ));
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/command-palette.css") }
        document::Style { {include_str!("../assets/command-palette.css")} }
        div {
            class: "palette-container",
            onkeydown: on_key_down,
            div {
                class: "search-section",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Type to search...",
                    value: "{query}",
                    autofocus: true,
                    oninput: move |evt| {
                        query.set(evt.value().clone());
                        selected_index.set(0);
                    }
                }
            }
            div {
                class: "results-section",
                if query.read().is_empty() && !results.read().is_empty() {
                    div { class: "section-header", "RECENT" }
                }
                {results().iter().enumerate().map(|(i, item)| {
                    let item = item.clone();
                    let is_selected = i == *selected_index.read();
                    rsx!(
                        div {
                            key: "{i}",
                            class: if is_selected { "result-item selected" } else { "result-item" },
                            onclick: move |_| {
                                execute_item(item.clone(), recent_items);
                            },
                            span { class: "result-icon", "{item.icon()}" }
                            span { class: "result-name", "{item.display_name()}" }
                        }
                    )
                })}
                if results.read().is_empty() && !query.read().is_empty() {
                    div { class: "no-results", "No results found" }
                }
            }
            if is_scanning {
                div {
                    class: "scanning-footer",
                    span { class: "spinner", "🔄" }
                    " Scanning..."
                }
            }
        }
    }
}

fn execute_item(item: ResultItem, mut recent_items: Signal<RecentItems>) {
    match item {
        ResultItem::Command(cmd) => {
            info!("Executing command: {}", cmd.display_name);

            // Add to recent
            recent_items
                .write()
                .add(cmd.display_name.clone(), ItemType::Command);
            let _ = recent_items.read().save_to_json();

            // Execute via IPC
            let ipc_client = IpcClient::new();
            if let Err(e) = ipc_client.execute_command(&cmd.command, cmd.args) {
                error!("Failed to execute command: {}", e);
            }
        }
        ResultItem::Executable(exe) => {
            info!("Launching executable: {}", exe.full_path.display());

            // Add to recent
            recent_items
                .write()
                .add(exe.name.clone(), ItemType::Executable);
            let _ = recent_items.read().save_to_json();

            // Launch executable
            if let Err(e) = process::Command::new(&exe.full_path).spawn() {
                error!("Failed to launch executable: {}", e);
            }
        }
    }

    // Close palette
    process::exit(0);
}

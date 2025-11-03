# Phase 1: Project Foundation - Detailed Task List

**Timeline:** Weeks 1-3 (3 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 1 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 1 Goals:**
- Set up complete project structure with Rust workspace
- Implement Windows API wrapper utilities
- Create binary tree data structures for window layout
- Build basic event loop with Windows hooks
- Establish window enumeration and tracking
- Set up testing, building, and linting infrastructure

---

## Prerequisites

**Required Tools:**
- Rust 1.75+ with cargo
- Windows 10/11 SDK
- Git
- Visual Studio Build Tools (for Windows API development)

**Knowledge Areas:**
- Rust programming language
- Windows API (Win32)
- Binary tree data structures
- Event-driven programming

---

## Task Breakdown

### Week 1: Project Structure & Development Environment

#### Task 1.1: Initialize Rust Workspace

**Objective:** Create a multi-crate Rust workspace with proper structure for the window manager, CLI, and status bar components.

**Steps:**

1. **Create workspace root directory structure:**
   ```bash
   mkdir -p crates/core/src
   mkdir -p crates/cli/src
   mkdir -p crates/status-bar/src
   mkdir -p config
   mkdir -p docs
   mkdir -p tests
   ```

2. **Create root Cargo.toml with workspace configuration:**
   ```toml
   [workspace]
   members = [
       "crates/core",
       "crates/cli",
       "crates/status-bar",
   ]
   resolver = "2"

   [workspace.dependencies]
   windows = { version = "0.52", features = [
       "Win32_Foundation",
       "Win32_UI_WindowsAndMessaging",
       "Win32_Graphics_Dwm",
       "Win32_Graphics_Gdi",
       "Win32_System_Threading",
       "Win32_UI_Input_KeyboardAndMouse",
       "Win32_System_Com",
       "Win32_UI_Shell",
   ] }
   tokio = { version = "1.35", features = ["full"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   toml = "0.8"
   anyhow = "1.0"
   thiserror = "1.0"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   ```

3. **Create crates/core/Cargo.toml:**
   ```toml
   [package]
   name = "tiling-wm-core"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   windows = { workspace = true }
   tokio = { workspace = true }
   serde = { workspace = true }
   serde_json = { workspace = true }
   toml = { workspace = true }
   anyhow = { workspace = true }
   thiserror = { workspace = true }
   tracing = { workspace = true }
   tracing-subscriber = { workspace = true }
   notify = "6.1"
   chrono = "0.4"
   ```

4. **Create basic module structure in crates/core/src/:**
   - `main.rs` - Application entry point
   - `lib.rs` - Library exports
   - `utils/mod.rs` - Utility module root
   - `utils/win32.rs` - Windows API wrappers
   - `window_manager/mod.rs` - Window manager root
   - `window_manager/tree.rs` - Binary tree structures
   - `event_loop.rs` - Event loop implementation

5. **Initialize Git repository:**
   ```bash
   git init
   echo "target/" > .gitignore
   echo "Cargo.lock" >> .gitignore
   git add .
   git commit -m "Initialize Rust workspace for tiling window manager"
   ```

**Acceptance Criteria:**
- [ ] Workspace compiles without errors: `cargo build --workspace`
- [ ] Directory structure matches specification
- [ ] All Cargo.toml files are valid and properly configured
- [ ] Git repository initialized with appropriate .gitignore
- [ ] Can run `cargo check --workspace` successfully

**Validation Commands:**
```bash
cargo build --workspace
cargo check --workspace
cargo tree --workspace
```

**Expected Output:**
- Clean workspace compilation
- No dependency resolution errors
- Proper crate dependency graph

---

#### Task 1.2: Implement Windows API Wrapper Utilities

**Objective:** Create safe Rust wrappers around Windows API functions for window management, providing a clean interface for the rest of the application.

**File:** `crates/core/src/utils/win32.rs`

**Required Implementations:**

1. **WindowHandle struct with core methods:**

   ```rust
   use windows::{
       core::*,
       Win32::Foundation::*,
       Win32::UI::WindowsAndMessaging::*,
       Win32::System::Threading::*,
   };

   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct WindowHandle(pub HWND);

   impl WindowHandle {
       pub fn from_hwnd(hwnd: HWND) -> Self;
       pub fn get_title(&self) -> Result<String>;
       pub fn get_class_name(&self) -> Result<String>;
       pub fn get_process_name(&self) -> Result<String>;
       pub fn is_visible(&self) -> bool;
       pub fn get_rect(&self) -> Result<RECT>;
       pub fn set_pos(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()>;
       pub fn show(&self) -> Result<()>;
       pub fn hide(&self) -> Result<()>;
       pub fn close(&self) -> Result<()>;
       pub fn is_maximized(&self) -> bool;
       pub fn is_minimized(&self) -> bool;
   }
   ```

2. **Window enumeration functions:**
   ```rust
   pub fn enumerate_windows() -> Result<Vec<WindowHandle>>;
   pub fn enumerate_visible_windows() -> Result<Vec<WindowHandle>>;
   pub fn get_foreground_window() -> Option<WindowHandle>;
   ```

3. **Window filtering utilities:**
   ```rust
   pub fn is_manageable_window(hwnd: HWND) -> bool;
   pub fn should_tile_window(hwnd: HWND) -> bool;
   ```

**Implementation Requirements:**

- Use `unsafe` blocks only where necessary for Win32 API calls
- Implement proper error handling with `anyhow::Result`
- Add comprehensive documentation comments for all public functions
- Handle edge cases (null handles, invalid windows, etc.)
- Implement `get_process_name()` using:
  - `GetWindowThreadProcessId()` to get PID
  - `OpenProcess()` to get process handle
  - `GetModuleFileNameExW()` or `QueryFullProcessImageNameW()` to get executable path

**Acceptance Criteria:**
- [ ] All WindowHandle methods compile and work correctly
- [ ] Window enumeration successfully lists all visible windows
- [ ] Process name retrieval works for standard applications
- [ ] Window title and class name retrieval handle Unicode correctly
- [ ] `set_pos()` successfully moves and resizes windows
- [ ] Error handling is comprehensive with meaningful error messages
- [ ] Code includes inline documentation for all public APIs
- [ ] No memory leaks in Win32 API calls (verified by testing)

**Testing Requirements:**

Create `crates/core/src/utils/win32_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enumerate_windows() {
        let windows = enumerate_windows().unwrap();
        assert!(!windows.is_empty(), "Should find at least one window");
    }

    #[test]
    fn test_window_handle_methods() {
        let windows = enumerate_visible_windows().unwrap();
        if let Some(window) = windows.first() {
            let title = window.get_title();
            assert!(title.is_ok());
            
            let class = window.get_class_name();
            assert!(class.is_ok());
            
            let rect = window.get_rect();
            assert!(rect.is_ok());
        }
    }

    #[test]
    fn test_foreground_window() {
        let fg = get_foreground_window();
        assert!(fg.is_some(), "Should have a foreground window");
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core win32
cargo clippy -p tiling-wm-core -- -D warnings
```

**Expected Output:**
- All tests pass
- No clippy warnings
- Manual verification: Can enumerate windows from Task Manager, Explorer, etc.

---

### Week 2: Binary Tree Data Structures & Layout System

#### Task 2.1: Implement Binary Tree Data Structure

**Objective:** Create a binary tree structure to represent window layouts with support for horizontal and vertical splits.

**File:** `crates/core/src/window_manager/tree.rs`

**Required Implementations:**

1. **Rect structure with operations:**

   ```rust
   use serde::{Serialize, Deserialize};

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub struct Rect {
       pub x: i32,
       pub y: i32,
       pub width: i32,
       pub height: i32,
   }

   impl Rect {
       pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self;
       pub fn area(&self) -> i32;
       pub fn contains_point(&self, x: i32, y: i32) -> bool;
       pub fn intersects(&self, other: &Rect) -> bool;
       pub fn split_horizontal(&self, ratio: f32) -> (Rect, Rect);
       pub fn split_vertical(&self, ratio: f32) -> (Rect, Rect);
       pub fn apply_gaps(&self, gaps_in: i32, gaps_out: i32) -> Rect;
       pub fn shrink(&self, amount: i32) -> Rect;
       pub fn expand(&self, amount: i32) -> Rect;
   }
   ```

2. **Split enum:**

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub enum Split {
       Horizontal,
       Vertical,
   }

   impl Split {
       pub fn opposite(&self) -> Split;
   }
   ```

3. **TreeNode structure:**

   ```rust
   use crate::utils::win32::WindowHandle;

   pub struct TreeNode {
       pub window: Option<WindowHandle>,
       pub split: Split,
       pub rect: Rect,
       pub left: Option<Box<TreeNode>>,
       pub right: Option<Box<TreeNode>>,
   }

   impl TreeNode {
       pub fn new_leaf(window: WindowHandle, rect: Rect) -> Self;
       pub fn new_container(split: Split, rect: Rect) -> Self;
       pub fn is_leaf(&self) -> bool;
       pub fn is_container(&self) -> bool;
       pub fn insert_window(&mut self, window: WindowHandle, split: Split);
       pub fn remove_window(&mut self, window: &WindowHandle) -> Option<WindowHandle>;
       pub fn find_window(&self, window: &WindowHandle) -> Option<&TreeNode>;
       pub fn find_window_mut(&mut self, window: &WindowHandle) -> Option<&mut TreeNode>;
       pub fn apply_geometry(&self) -> anyhow::Result<()>;
       pub fn collect_windows(&self) -> Vec<WindowHandle>;
       pub fn count_windows(&self) -> usize;
       pub fn rebalance(&mut self);
   }
   ```

**Implementation Requirements:**

- Implement proper memory management (no leaks with Box pointers)
- `insert_window()` should convert leaves to containers
- `remove_window()` should collapse containers when a window is removed
- `apply_geometry()` should recursively set window positions
- Handle edge cases: empty trees, single window, etc.
- Add comprehensive documentation

**Acceptance Criteria:**
- [ ] Rect split operations produce correct dimensions and positions
- [ ] Gaps application works correctly (shrinks rect from all sides)
- [ ] TreeNode can insert windows and build proper tree structure
- [ ] Window removal correctly collapses tree nodes
- [ ] `apply_geometry()` successfully sets window positions via Win32 API
- [ ] `collect_windows()` returns all windows in the tree
- [ ] No memory leaks (verified with cargo test with leak detection)
- [ ] All tree operations maintain tree invariants

**Testing Requirements:**

Create `crates/core/src/window_manager/tree_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_split_horizontal() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let (left, right) = rect.split_horizontal(0.5);
        
        assert_eq!(left.x, 0);
        assert_eq!(left.y, 0);
        assert_eq!(left.width, 960);
        assert_eq!(left.height, 1080);
        
        assert_eq!(right.x, 960);
        assert_eq!(right.y, 0);
        assert_eq!(right.width, 960);
        assert_eq!(right.height, 1080);
    }

    #[test]
    fn test_rect_split_vertical() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let (top, bottom) = rect.split_vertical(0.5);
        
        assert_eq!(top.height, 540);
        assert_eq!(bottom.height, 540);
        assert_eq!(bottom.y, 540);
    }

    #[test]
    fn test_rect_gaps() {
        let rect = Rect::new(0, 0, 100, 100);
        let gapped = rect.apply_gaps(5, 10);
        
        assert_eq!(gapped.x, 10);
        assert_eq!(gapped.y, 10);
        assert_eq!(gapped.width, 100 - 20 - 5);
        assert_eq!(gapped.height, 100 - 20 - 5);
    }

    #[test]
    fn test_tree_node_creation() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let node = TreeNode::new_container(Split::Vertical, rect);
        
        assert!(node.is_container());
        assert!(!node.is_leaf());
        assert_eq!(node.count_windows(), 0);
    }

    #[test]
    fn test_tree_insert_and_count() {
        // This test will need to be adjusted based on actual window handles
        // For now, test the structure
        let rect = Rect::new(0, 0, 1920, 1080);
        let mut root = TreeNode::new_container(Split::Vertical, rect);
        
        // Would insert actual windows here
        assert_eq!(root.count_windows(), 0);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core tree
cargo clippy -p tiling-wm-core -- -D warnings
```

**Expected Output:**
- All tests pass
- Rect operations produce mathematically correct results
- Tree maintains proper structure after operations

---

#### Task 2.2: Implement Window Manager Core

**Objective:** Create the central WindowManager struct that manages window trees, workspaces, and monitors.

**File:** `crates/core/src/window_manager/mod.rs`

**Required Implementations:**

1. **Module structure:**
   ```rust
   pub mod tree;
   pub mod layout;
   pub mod window;
   pub mod monitor;
   pub mod focus;

   pub use tree::{TreeNode, Rect, Split};
   pub use window::ManagedWindow;
   ```

2. **WindowManager struct:**

   ```rust
   use std::collections::HashMap;
   use crate::utils::win32::WindowHandle;

   pub struct WindowManager {
       trees: HashMap<usize, TreeNode>,
       active_workspace: usize,
       monitors: Vec<MonitorInfo>,
       managed_windows: HashMap<isize, ManagedWindow>,
   }

   impl WindowManager {
       pub fn new() -> Self;
       pub fn initialize(&mut self) -> anyhow::Result<()>;
       pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()>;
       pub fn unmanage_window(&mut self, window: &WindowHandle) -> anyhow::Result<()>;
       pub fn tile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()>;
       pub fn switch_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()>;
       pub fn refresh_monitors(&mut self) -> anyhow::Result<()>;
       pub fn should_manage_window(&self, window: &WindowHandle) -> anyhow::Result<bool>;
       pub fn get_active_workspace(&self) -> usize;
       pub fn get_workspace_tree(&self, workspace_id: usize) -> Option<&TreeNode>;
       pub fn get_workspace_tree_mut(&mut self, workspace_id: usize) -> Option<&mut TreeNode>;
   }
   ```

3. **MonitorInfo struct:**

   ```rust
   #[derive(Debug, Clone)]
   pub struct MonitorInfo {
       pub id: usize,
       pub name: String,
       pub work_area: Rect,
       pub dpi_scale: f32,
   }
   ```

**Implementation Requirements:**

- `initialize()` should:
  - Enumerate all monitors
  - Create initial workspace trees
  - Set up default configuration
- `manage_window()` should:
  - Check if window should be managed (not popup, tooltip, etc.)
  - Add window to current workspace tree
  - Apply geometry to tile windows
- `should_manage_window()` should filter:
  - Invisible windows
  - Popup windows
  - Tool windows
  - Windows without title bars
  - System windows
- `refresh_monitors()` should use `EnumDisplayMonitors` Win32 API
- Handle DPI scaling correctly

**Acceptance Criteria:**
- [ ] WindowManager initializes successfully
- [ ] Can manage multiple windows in a tree
- [ ] Window filtering correctly excludes non-manageable windows
- [ ] Monitor enumeration works for 1-3+ monitors
- [ ] DPI scaling is detected and stored
- [ ] Workspace switching works correctly
- [ ] Memory is properly managed (no leaks)

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new();
        assert_eq!(wm.get_active_workspace(), 1);
    }

    #[test]
    fn test_window_manager_initialization() {
        let mut wm = WindowManager::new();
        let result = wm.initialize();
        assert!(result.is_ok());
    }

    // Integration test - requires actual windows
    #[test]
    #[ignore]
    fn test_manage_real_windows() {
        let mut wm = WindowManager::new();
        wm.initialize().unwrap();
        
        let windows = crate::utils::win32::enumerate_visible_windows().unwrap();
        for window in windows.iter().take(3) {
            if wm.should_manage_window(window).unwrap() {
                let result = wm.manage_window(*window);
                assert!(result.is_ok());
            }
        }
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core window_manager
cargo test -p tiling-wm-core --test integration -- --ignored
```

---

### Week 3: Event Loop & Main Application

#### Task 3.1: Implement Windows Event Hook System

**Objective:** Create an event loop that monitors Windows events and dispatches them to the window manager.

**File:** `crates/core/src/event_loop.rs`

**Required Implementations:**

1. **WindowEvent enum:**

   ```rust
   use windows::Win32::Foundation::HWND;

   #[derive(Debug, Clone)]
   pub enum WindowEvent {
       WindowCreated(HWND),
       WindowDestroyed(HWND),
       WindowShown(HWND),
       WindowHidden(HWND),
       WindowMoved(HWND),
       WindowMinimized(HWND),
       WindowRestored(HWND),
       WindowFocused(HWND),
       MonitorChanged,
   }
   ```

2. **EventLoop struct:**

   ```rust
   use std::sync::mpsc::{channel, Sender, Receiver};
   use windows::Win32::UI::WindowsAndMessaging::*;

   pub struct EventLoop {
       event_tx: Sender<WindowEvent>,
       event_rx: Receiver<WindowEvent>,
       hook: Option<HWINEVENTHOOK>,
   }

   impl EventLoop {
       pub fn new() -> Self;
       pub fn start(&mut self) -> anyhow::Result<()>;
       pub fn stop(&mut self) -> anyhow::Result<()>;
       pub fn poll_events(&self) -> impl Iterator<Item = WindowEvent> + '_;
       pub fn process_messages(&self) -> anyhow::Result<()>;
   }
   ```

3. **Event hook callback:**
   ```rust
   unsafe extern "system" fn win_event_proc(
       hook: HWINEVENTHOOK,
       event: u32,
       hwnd: HWND,
       id_object: i32,
       id_child: i32,
       id_event_thread: u32,
       dwms_event_time: u32,
   );
   ```

**Implementation Requirements:**

- Use `SetWinEventHook()` to register for Windows events:
  - `EVENT_OBJECT_CREATE` - Window created
  - `EVENT_OBJECT_DESTROY` - Window destroyed
  - `EVENT_OBJECT_SHOW` - Window shown
  - `EVENT_OBJECT_HIDE` - Window hidden
  - `EVENT_SYSTEM_MOVESIZEEND` - Window moved/resized
  - `EVENT_SYSTEM_FOREGROUND` - Window focused
- Use `WINEVENT_OUTOFCONTEXT` flag for out-of-context hook
- Implement thread-safe event queue using `std::sync::mpsc`
- Handle Windows message pump with `GetMessage` and `DispatchMessage`
- Properly cleanup hook with `UnhookWinEvent` in Drop trait

**Acceptance Criteria:**
- [ ] Event hook successfully registers with Windows
- [ ] Window creation events are detected
- [ ] Window destruction events are detected
- [ ] Window focus changes are detected
- [ ] Events are properly queued and can be polled
- [ ] No memory leaks or handle leaks
- [ ] Hook is properly unregistered on drop
- [ ] Thread-safe event communication works correctly

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_creation() {
        let event_loop = EventLoop::new();
        // Basic structure test
    }

    #[test]
    #[ignore]
    fn test_event_loop_start() {
        let mut event_loop = EventLoop::new();
        let result = event_loop.start();
        assert!(result.is_ok());
        event_loop.stop().unwrap();
    }

    // Manual test: Open/close windows and verify events
    #[test]
    #[ignore]
    fn test_window_events_detection() {
        let mut event_loop = EventLoop::new();
        event_loop.start().unwrap();
        
        // Wait for events
        std::thread::sleep(std::time::Duration::from_secs(5));
        
        let events: Vec<_> = event_loop.poll_events().collect();
        assert!(!events.is_empty(), "Should detect window events");
        
        event_loop.stop().unwrap();
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core event_loop
```

---

#### Task 3.2: Implement Main Application Entry Point

**Objective:** Create the main application that ties together the event loop and window manager.

**File:** `crates/core/src/main.rs`

**Required Implementations:**

1. **Main function with initialization:**

   ```rust
   mod window_manager;
   mod event_loop;
   mod utils;

   use window_manager::WindowManager;
   use event_loop::{EventLoop, WindowEvent};
   use tracing::{info, error, debug, warn};
   use anyhow::Result;

   fn main() -> Result<()> {
       // Initialize logging
       initialize_logging();
       
       info!("Starting Tiling Window Manager v0.1.0");
       
       // Initialize window manager
       let mut wm = WindowManager::new();
       wm.initialize()?;
       info!("Window manager initialized");
       
       // Set up event loop
       let mut event_loop = EventLoop::new();
       event_loop.start()?;
       info!("Event loop started");
       
       // Main event loop
       run_event_loop(wm, event_loop)?;
       
       Ok(())
   }

   fn initialize_logging();
   fn run_event_loop(wm: WindowManager, event_loop: EventLoop) -> Result<()>;
   fn handle_window_event(wm: &mut WindowManager, event: WindowEvent) -> Result<()>;
   ```

2. **Event handling logic:**

   ```rust
   fn handle_window_event(wm: &mut WindowManager, event: WindowEvent) -> Result<()> {
       match event {
           WindowEvent::WindowCreated(hwnd) => {
               debug!("Window created: {:?}", hwnd);
               let window = utils::win32::WindowHandle::from_hwnd(hwnd);
               if wm.should_manage_window(&window)? {
                   wm.manage_window(window)?;
                   info!("Managing new window: {}", window.get_title()?);
               }
           }
           WindowEvent::WindowDestroyed(hwnd) => {
               debug!("Window destroyed: {:?}", hwnd);
               let window = utils::win32::WindowHandle::from_hwnd(hwnd);
               wm.unmanage_window(&window)?;
           }
           WindowEvent::WindowFocused(hwnd) => {
               debug!("Window focused: {:?}", hwnd);
               // Update focus tracking
           }
           _ => {}
       }
       Ok(())
   }
   ```

3. **Logging setup:**
   ```rust
   fn initialize_logging() {
       tracing_subscriber::fmt()
           .with_env_filter("tiling_wm_core=debug,info")
           .with_target(false)
           .with_thread_ids(true)
           .with_line_number(true)
           .init();
   }
   ```

**Implementation Requirements:**

- Use structured logging with `tracing` crate
- Gracefully handle errors and log them
- Implement clean shutdown on Ctrl+C
- Add small sleep in event loop to prevent 100% CPU usage
- Log startup information and configuration
- Handle panics gracefully with panic hook

**Acceptance Criteria:**
- [ ] Application compiles and runs without errors
- [ ] Logging is properly initialized and outputs to console
- [ ] Window creation events are detected and logged
- [ ] New windows are automatically managed and tiled
- [ ] Application can run for extended period without crashing
- [ ] CPU usage is reasonable (< 5% idle)
- [ ] Memory usage is stable (no leaks)
- [ ] Clean shutdown works with Ctrl+C

**Testing Requirements:**

**Manual Testing Procedure:**

1. Build and run the application:
   ```bash
   cargo run -p tiling-wm-core
   ```

2. Open several windows (e.g., Notepad, Calculator, Explorer)

3. Verify in logs:
   - Window creation events are detected
   - Windows are being managed
   - Tiling is applied

4. Close windows and verify:
   - Window destruction events are detected
   - Windows are unmanaged

5. Monitor resource usage:
   ```powershell
   Get-Process tiling-wm-core | Select-Object CPU, WorkingSet
   ```

6. Let run for 10 minutes and verify:
   - No crashes
   - Stable memory usage
   - Reasonable CPU usage

**Validation Commands:**
```bash
cargo build -p tiling-wm-core --release
cargo run -p tiling-wm-core
```

**Expected Output:**
```
[INFO] Starting Tiling Window Manager v0.1.0
[INFO] Window manager initialized
[INFO] Event loop started
[DEBUG] Window created: HWND(0x12345)
[INFO] Managing new window: Notepad
[DEBUG] Window created: HWND(0x67890)
[INFO] Managing new window: Calculator
...
```

---

#### Task 3.3: Create Basic Configuration File

**Objective:** Create a default TOML configuration file with basic settings.

**File:** `config/config.toml`

**Required Content:**

```toml
# Tiling Window Manager Configuration
# Version: 0.1.0

[general]
# Gap size between windows (pixels)
gaps_in = 5

# Gap size around screen edges (pixels)
gaps_out = 10

# Border size around windows (pixels)
border_size = 2

# Active window border color (hex)
active_border_color = "#89b4fa"

# Inactive window border color (hex)
inactive_border_color = "#585b70"

[decoration]
# Corner rounding radius (pixels)
rounding = 10

# Active window opacity (0.0 - 1.0)
active_opacity = 1.0

# Inactive window opacity (0.0 - 1.0)
inactive_opacity = 0.9

[animations]
# Enable animations
enabled = true

# Animation speed multiplier (1.0 = normal)
speed = 1.0

[layouts]
# Default layout for new workspaces
default = "dwindle"

[layouts.dwindle]
# Automatically choose split direction based on window dimensions
smart_split = true

# Remove gaps when only one window
no_gaps_when_only = false

[layouts.master]
# Size ratio for master window (0.0 - 1.0)
master_factor = 0.55

# Number of windows in master area
master_count = 1

# Example window rules (not yet implemented)
# [[window_rules]]
# match_process = "firefox.exe"
# actions = ["workspace:2"]

# [[window_rules]]
# match_title = ".*Calculator.*"
# actions = ["float"]
```

**Acceptance Criteria:**
- [ ] File is valid TOML
- [ ] All settings have reasonable defaults
- [ ] Comments explain each setting
- [ ] File can be parsed by `toml` crate
- [ ] Settings match the schema in DETAILED_ROADMAP.md

**Validation Commands:**
```bash
# Test TOML parsing
cargo run --example parse_config config/config.toml
```

Create example: `crates/core/examples/parse_config.rs`:
```rust
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1).expect("Usage: parse_config <config.toml>");
    
    let content = fs::read_to_string(config_path).expect("Failed to read config");
    let config: toml::Value = toml::from_str(&content).expect("Failed to parse TOML");
    
    println!("✓ Configuration is valid TOML");
    println!("{:#?}", config);
}
```

---

#### Task 3.4: Add Documentation and README

**Objective:** Create comprehensive documentation for Phase 1 deliverables.

**Files to Create/Update:**

1. **Update root README.md** with build instructions:

   ```markdown
   ## Building

   ### Prerequisites
   - Rust 1.75 or later
   - Windows 10/11
   - Visual Studio Build Tools

   ### Build Instructions
   ```bash
   # Clone the repository
   git clone https://github.com/yourusername/tiling-wm.git
   cd tiling-wm

   # Build all crates
   cargo build --workspace

   # Build release version
   cargo build --workspace --release

   # Run the window manager
   cargo run -p tiling-wm-core

   # Run tests
   cargo test --workspace
   ```

   ### Development
   ```bash
   # Check code
   cargo check --workspace

   # Run clippy
   cargo clippy --workspace -- -D warnings

   # Format code
   cargo fmt --workspace
   ```
   ```

2. **Create docs/BUILDING.md** with detailed build instructions

3. **Create docs/ARCHITECTURE.md** documenting Phase 1 architecture:
   - Module structure
   - Data flow diagrams
   - Tree structure explanation
   - Event flow

4. **Add inline documentation** to all public APIs:
   - Module-level documentation
   - Struct documentation
   - Function documentation with examples

**Acceptance Criteria:**
- [ ] README.md has clear build instructions
- [ ] All public APIs have documentation comments
- [ ] Documentation builds with `cargo doc --no-deps`
- [ ] Architecture document clearly explains system design
- [ ] Code examples in documentation compile

**Validation Commands:**
```bash
cargo doc --no-deps --open
cargo test --doc
```

---

#### Task 3.5: Set Up Testing Infrastructure

**Objective:** Establish comprehensive testing infrastructure for the project.

**Required Implementations:**

1. **Create test modules for each component:**
   - `crates/core/tests/integration_tests.rs`
   - `crates/core/benches/performance.rs`

2. **Integration test suite:**

   ```rust
   // crates/core/tests/integration_tests.rs
   use tiling_wm_core::window_manager::WindowManager;
   use tiling_wm_core::utils::win32;

   #[test]
   fn test_full_initialization() {
       let mut wm = WindowManager::new();
       assert!(wm.initialize().is_ok());
   }

   #[test]
   fn test_window_enumeration() {
       let windows = win32::enumerate_windows().unwrap();
       assert!(!windows.is_empty());
       
       for window in windows.iter().take(5) {
           let title = window.get_title();
           assert!(title.is_ok());
       }
   }

   #[test]
   #[ignore] // Requires interactive testing
   fn test_window_management_lifecycle() {
       // Test managing, tiling, and unmanaging windows
   }
   ```

3. **Performance benchmarks:**

   ```rust
   // crates/core/benches/performance.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use tiling_wm_core::window_manager::tree::Rect;

   fn benchmark_rect_split(c: &mut Criterion) {
       c.bench_function("rect_split_horizontal", |b| {
           let rect = Rect::new(0, 0, 1920, 1080);
           b.iter(|| {
               black_box(rect.split_horizontal(0.5));
           });
       });
   }

   criterion_group!(benches, benchmark_rect_split);
   criterion_main!(benches);
   ```

4. **Add benchmark dependencies to Cargo.toml:**
   ```toml
   [dev-dependencies]
   criterion = "0.5"

   [[bench]]
   name = "performance"
   harness = false
   ```

**Acceptance Criteria:**
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Integration tests are comprehensive
- [ ] Benchmarks run successfully: `cargo bench`
- [ ] Test coverage is adequate (>70% for critical code)
- [ ] CI-friendly tests (can run in automated environment)

**Validation Commands:**
```bash
cargo test --workspace --all-targets
cargo bench
cargo test --workspace -- --nocapture
```

---

## Phase 1 Completion Checklist

### Build & Compilation
- [ ] `cargo build --workspace` succeeds without errors
- [ ] `cargo build --workspace --release` succeeds
- [ ] No warnings from `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted with `cargo fmt --workspace --check`

### Core Functionality
- [ ] Windows can be enumerated using Win32 API wrappers
- [ ] WindowHandle correctly retrieves window properties (title, class, process)
- [ ] Binary tree structure correctly manages window layout
- [ ] Rect split operations produce correct dimensions
- [ ] WindowManager can manage and tile multiple windows
- [ ] Event loop detects window creation/destruction events
- [ ] Main application runs and manages windows automatically

### Testing
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Integration tests pass (or are marked ignored if requiring manual verification)
- [ ] No test failures or panics
- [ ] Performance benchmarks run successfully

### Documentation
- [ ] README.md has build instructions
- [ ] All public APIs have doc comments
- [ ] Architecture documented in docs/ARCHITECTURE.md
- [ ] `cargo doc --no-deps` builds successfully
- [ ] Examples in documentation compile and work

### Configuration
- [ ] config/config.toml is valid TOML
- [ ] Default configuration values are sensible
- [ ] Configuration file is well-commented

### Code Quality
- [ ] No unsafe code outside of Win32 API calls
- [ ] Proper error handling throughout
- [ ] No unwrap() calls in production code (use proper Result types)
- [ ] Memory safety verified (no leaks detected)
- [ ] Thread safety where applicable

### Manual Validation
- [ ] Application starts without errors
- [ ] Windows are detected when created
- [ ] Windows are automatically tiled
- [ ] Application runs stable for 10+ minutes
- [ ] CPU usage is reasonable (<5% idle)
- [ ] Memory usage is stable (no leaks)
- [ ] Logs show proper event detection and handling

---

## Success Criteria

Phase 1 is considered complete when:

1. **All tasks are completed** as defined in this document
2. **All acceptance criteria** are met for each task
3. **All tests pass** without failures
4. **Manual validation** confirms expected behavior
5. **Documentation is complete** and accurate
6. **Code quality** meets project standards (clippy, fmt)
7. **Performance baselines** are established with benchmarks

---

## Deliverables

At the end of Phase 1, the following should be delivered:

1. **Working codebase:**
   - Rust workspace with three crates (core, cli, status-bar)
   - Compiled binaries for core window manager
   - All source code in Git repository

2. **Functional capabilities:**
   - Window enumeration via Win32 API
   - Binary tree layout structure
   - Basic window management (auto-tiling new windows)
   - Event loop detecting window events
   - Initial logging and diagnostics

3. **Testing infrastructure:**
   - Unit tests for core components
   - Integration test suite
   - Performance benchmarks
   - Test documentation

4. **Documentation:**
   - Updated README.md
   - API documentation (cargo doc)
   - Architecture overview
   - Build instructions

5. **Configuration:**
   - Default config.toml
   - Configuration parsing example

---

## Next Steps

After completing Phase 1, proceed to **Phase 2: Core Window Management** (Weeks 4-8), which will implement:

- Complete dwindle layout algorithm
- Master-stack layout
- Window state management (tiled/floating/fullscreen)
- Focus management
- Window operations (close, move, resize)
- Window filtering and rules

See DETAILED_ROADMAP.md for Phase 2 specifications.

---

## Troubleshooting

### Common Issues

**Issue: Compilation fails with Windows API errors**
- Solution: Ensure Windows SDK is installed
- Verify `windows` crate features are correctly configured
- Check Rust version (need 1.75+)

**Issue: Tests fail with "No windows found"**
- Solution: Tests requiring windows need open applications
- Mark these tests as `#[ignore]` for CI
- Run manually with `cargo test -- --ignored`

**Issue: Event loop doesn't detect events**
- Solution: Verify hook registration with Windows
- Check event loop is running in proper thread
- Ensure Windows message pump is processing messages

**Issue: High CPU usage**
- Solution: Add appropriate sleep/yield in main loop
- Use blocking operations where appropriate
- Profile with `perf` or Windows Performance Analyzer

---

## Notes for Autonomous Agents

When executing this task list:

1. **Follow order strictly**: Tasks build on each other
2. **Validate each step**: Run acceptance criteria after each task
3. **Fix errors immediately**: Don't proceed if tests fail
4. **Document changes**: Add inline comments for complex logic
5. **Commit frequently**: Use Git commits after each completed task
6. **Test incrementally**: Run tests after each significant change
7. **Check resource usage**: Monitor memory and CPU during testing
8. **Handle errors gracefully**: Use proper Result types, avoid panics
9. **Keep code clean**: Run rustfmt and clippy regularly
10. **Reference DETAILED_ROADMAP.md**: For additional context and examples

---

**End of Phase 1 Task Document**

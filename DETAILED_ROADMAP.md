# Detailed Implementation Roadmap
## Rust-Based Tiling Window Manager for Windows with Status Bar

**Document Version:** 1.0  
**Last Updated:** November 2025  
**Target Platform:** Windows 10/11 (x64)  
**Programming Language:** Rust  
**Estimated Timeline:** 6-9 months to production v1.0  
**Based on:** Hyprland + Waybar feature analysis

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture & Design](#architecture--design)
3. [Development Phases](#development-phases)
4. [Phase 1: Project Foundation](#phase-1-project-foundation-weeks-1-3)
5. [Phase 2: Core Window Management](#phase-2-core-window-management-weeks-4-8)
6. [Phase 3: Workspace System](#phase-3-workspace-system-weeks-9-12)
7. [Phase 4: Configuration & Rules](#phase-4-configuration--rules-weeks-13-16)
8. [Phase 5: IPC & CLI](#phase-5-ipc--cli-weeks-17-20)
9. [Phase 6: Status Bar Implementation](#phase-6-status-bar-implementation-weeks-21-26)
10. [Phase 7: Polish & Advanced Features](#phase-7-polish--advanced-features-weeks-27-32)
11. [Phase 8: Production Readiness](#phase-8-production-readiness-weeks-33-36)
12. [Testing Strategy](#testing-strategy)
13. [Deployment & Distribution](#deployment--distribution)
14. [Maintenance & Future Development](#maintenance--future-development)

---

## Project Overview

### Project Goals

**Primary Objectives:**
1. Create a production-ready tiling window manager for Windows
2. Achieve 60-70% feature parity with Hyprland
3. Implement a customizable status bar similar to Waybar
4. Provide excellent performance and stability
5. Maintain clean, documented, maintainable Rust codebase

**Success Criteria:**
- Successfully tiles and manages windows across multiple monitors
- Supports at least 10 configurable workspaces per monitor
- Provides IPC for external control and scripting
- Includes functional status bar with system information
- Runs stable for 24+ hours without crashes
- Memory usage < 50MB idle, < 150MB active
- Window operations complete in < 50ms

### Technology Stack

**Core Technologies:**
- **Language:** Rust (2021 edition)
- **Windows API:** `windows-rs` crate (official Microsoft bindings)
- **Async Runtime:** `tokio` for IPC and event handling
- **Configuration:** `toml` with `serde` for parsing
- **IPC Protocol:** JSON over named pipes
- **Status Bar UI:** Custom rendering or `iced` GUI framework

**Key Dependencies:**
```toml
[dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Graphics_Dwm",
    "Win32_System_Threading",
    "Win32_UI_Input_KeyboardAndMouse",
] }
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
notify = "6.1"  # File watching for config hot-reload
```

---

## Architecture & Design

### System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Windows OS (DWM)                      │
└──────────────────┬──────────────────────────────────────┘
                   │ Win32 API
┌──────────────────▼──────────────────────────────────────┐
│            Tiling Window Manager Core                    │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Window Manager (main daemon)                      │ │
│  │  - Window enumeration and tracking                 │ │
│  │  - Tree-based layout algorithm                     │ │
│  │  - Event loop and message processing               │ │
│  │  - Configuration management                        │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Workspace Manager                                 │ │
│  │  - Virtual desktop integration                     │ │
│  │  - Workspace state tracking                        │ │
│  │  - Per-monitor workspace assignment                │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Input Handler                                     │ │
│  │  - Global keyboard hooks                           │ │
│  │  - Hotkey registration                             │ │
│  │  - Mouse event processing                          │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │  IPC Server                                        │ │
│  │  - Named pipe server                               │ │
│  │  - JSON protocol handler                           │ │
│  │  - Event broadcasting                              │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────┬──────────────────┬────────────────────┘
                  │                  │
        ┌─────────▼─────┐   ┌───────▼────────┐
        │  CLI Client   │   │  Status Bar    │
        │               │   │                │
        │  - Send cmds  │   │  - Modules     │
        │  - Query      │   │  - Rendering   │
        │  - Events     │   │  - Styling     │
        └───────────────┘   └────────────────┘
```

### Module Breakdown

**1. Core Modules:**
- `main.rs` - Entry point, initialization
- `window_manager/` - Core tiling logic
  - `tree.rs` - Binary tree data structure
  - `layout.rs` - Layout algorithms (dwindle, master)
  - `window.rs` - Window state and manipulation
  - `monitor.rs` - Multi-monitor support
- `workspace/` - Workspace management
  - `manager.rs` - Workspace lifecycle
  - `rules.rs` - Workspace assignment rules
- `input/` - Input handling
  - `keyboard.rs` - Keyboard hooks and bindings
  - `mouse.rs` - Mouse events
  - `hotkeys.rs` - Hotkey registration
- `config/` - Configuration system
  - `parser.rs` - TOML parsing
  - `schema.rs` - Configuration schema
  - `watcher.rs` - File watching for hot-reload
- `ipc/` - Inter-process communication
  - `server.rs` - Named pipe server
  - `protocol.rs` - JSON protocol
  - `events.rs` - Event system
- `rules/` - Window rules engine
  - `matcher.rs` - Window matching logic
  - `actions.rs` - Rule actions
- `utils/` - Utility functions
  - `win32.rs` - Windows API helpers
  - `geometry.rs` - Rectangle math
  - `dpi.rs` - DPI awareness

**2. Status Bar Modules:**
- `status_bar/` - Status bar application
  - `main.rs` - Status bar entry point
  - `modules/` - Modular widgets
    - `workspaces.rs` - Workspace indicator
    - `window_title.rs` - Active window
    - `clock.rs` - Date/time
    - `battery.rs` - Battery status
    - `cpu.rs` - CPU usage
    - `memory.rs` - RAM usage
    - `network.rs` - Network info
    - `volume.rs` - Volume control
    - `tray.rs` - System tray
  - `rendering/` - UI rendering
  - `styling/` - Theme system
  - `config.rs` - Status bar config

**3. CLI Module:**
- `cli/` - Command-line client
  - `main.rs` - CLI entry point
  - `commands.rs` - Command definitions
  - `client.rs` - IPC client

---

## Development Phases

### Overview

| Phase | Duration | Focus | Deliverable |
|-------|----------|-------|-------------|
| Phase 1 | Weeks 1-3 | Foundation | Project setup, basic structs |
| Phase 2 | Weeks 4-8 | Core WM | Basic tiling functionality |
| Phase 3 | Weeks 9-12 | Workspaces | Virtual desktop integration |
| Phase 4 | Weeks 13-16 | Config & Rules | Configuration system |
| Phase 5 | Weeks 17-20 | IPC | External control |
| Phase 6 | Weeks 21-26 | Status Bar | Waybar-like status bar |
| Phase 7 | Weeks 27-32 | Polish | Advanced features & effects |
| Phase 8 | Weeks 33-36 | Production | Testing, docs, release |

---

## Phase 1: Project Foundation (Weeks 1-3)

### Objectives
- Set up development environment
- Create project structure
- Implement basic Windows API interaction
- Establish build and test infrastructure

### Week 1: Development Environment Setup

**Day 1-2: Project Initialization**
```bash
# Create new Rust project
cargo new tiling-wm --bin
cd tiling-wm

# Initialize git
git init
git add .
git commit -m "Initial commit"

# Create workspace for multiple crates
mkdir -p crates/core crates/cli crates/status-bar
```

**Project Structure:**
```
tiling-wm/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── core/                  # Main window manager
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       └── ...
│   ├── cli/                   # CLI client
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── status-bar/            # Status bar
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── config/                    # Default configs
│   ├── config.toml
│   └── status-bar.toml
├── docs/                      # Documentation
├── tests/                     # Integration tests
└── README.md
```

**Cargo.toml (workspace):**
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

**Day 3-4: Core Dependencies and Build System**

**crates/core/Cargo.toml:**
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

**Day 5-7: Basic Windows API Wrapper**

**crates/core/src/utils/win32.rs:**
```rust
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
};

pub struct WindowHandle(pub HWND);

impl WindowHandle {
    pub fn from_hwnd(hwnd: HWND) -> Self {
        Self(hwnd)
    }
    
    pub fn get_title(&self) -> Result<String> {
        unsafe {
            let length = GetWindowTextLengthW(self.0);
            if length == 0 {
                return Ok(String::new());
            }
            
            let mut buffer = vec![0u16; (length + 1) as usize];
            GetWindowTextW(self.0, &mut buffer);
            Ok(String::from_utf16_lossy(&buffer[..length as usize]))
        }
    }
    
    pub fn get_class_name(&self) -> Result<String> {
        unsafe {
            let mut buffer = vec![0u16; 256];
            let length = GetClassNameW(self.0, &mut buffer);
            Ok(String::from_utf16_lossy(&buffer[..length as usize]))
        }
    }
    
    pub fn get_process_name(&self) -> Result<String> {
        // Implementation to get process name
        // Uses GetWindowThreadProcessId + OpenProcess + GetModuleFileNameExW
        todo!("Implement process name retrieval")
    }
    
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.0).as_bool() }
    }
    
    pub fn get_rect(&self) -> Result<RECT> {
        unsafe {
            let mut rect = RECT::default();
            GetWindowRect(self.0, &mut rect)?;
            Ok(rect)
        }
    }
    
    pub fn set_pos(&self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        unsafe {
            SetWindowPos(
                self.0,
                HWND::default(),
                x, y, width, height,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            Ok(())
        }
    }
}

pub fn enumerate_windows() -> Result<Vec<WindowHandle>> {
    let mut windows = Vec::new();
    
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut windows as *mut _ as isize),
        )?;
    }
    
    Ok(windows)
}

unsafe extern "system" fn enum_windows_proc(
    hwnd: HWND,
    lparam: LPARAM,
) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowHandle>);
    windows.push(WindowHandle::from_hwnd(hwnd));
    true.into()
}
```

### Week 2: Data Structures & Core Types

**Day 8-10: Window Tree Structure**

**crates/core/src/window_manager/tree.rs:**
```rust
use crate::utils::win32::WindowHandle;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Split {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn split_horizontal(&self, ratio: f32) -> (Rect, Rect) {
        let left_width = (self.width as f32 * ratio) as i32;
        let right_width = self.width - left_width;
        
        (
            Rect::new(self.x, self.y, left_width, self.height),
            Rect::new(self.x + left_width, self.y, right_width, self.height),
        )
    }
    
    pub fn split_vertical(&self, ratio: f32) -> (Rect, Rect) {
        let top_height = (self.height as f32 * ratio) as i32;
        let bottom_height = self.height - top_height;
        
        (
            Rect::new(self.x, self.y, self.width, top_height),
            Rect::new(self.x, self.y + top_height, self.width, bottom_height),
        )
    }
    
    pub fn apply_gaps(&self, gaps_in: i32, gaps_out: i32) -> Rect {
        Rect::new(
            self.x + gaps_out,
            self.y + gaps_out,
            self.width - 2 * gaps_out - gaps_in,
            self.height - 2 * gaps_out - gaps_in,
        )
    }
}

pub struct TreeNode {
    pub window: Option<WindowHandle>,
    pub split: Split,
    pub rect: Rect,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl TreeNode {
    pub fn new_leaf(window: WindowHandle, rect: Rect) -> Self {
        Self {
            window: Some(window),
            split: Split::Vertical,
            rect,
            left: None,
            right: None,
        }
    }
    
    pub fn new_container(split: Split, rect: Rect) -> Self {
        Self {
            window: None,
            split,
            rect,
            left: None,
            right: None,
        }
    }
    
    pub fn is_leaf(&self) -> bool {
        self.window.is_some()
    }
    
    pub fn insert_window(&mut self, window: WindowHandle, split: Split) {
        if self.is_leaf() {
            // Convert leaf to container
            let current_window = self.window.take().unwrap();
            self.split = split;
            
            let (left_rect, right_rect) = match split {
                Split::Horizontal => self.rect.split_horizontal(0.5),
                Split::Vertical => self.rect.split_vertical(0.5),
            };
            
            self.left = Some(Box::new(TreeNode::new_leaf(current_window, left_rect)));
            self.right = Some(Box::new(TreeNode::new_leaf(window, right_rect)));
        } else {
            // Insert into container
            if let Some(ref mut right) = self.right {
                right.insert_window(window, split);
            }
        }
    }
    
    pub fn apply_geometry(&self) -> anyhow::Result<()> {
        if let Some(ref window) = self.window {
            window.set_pos(self.rect.x, self.rect.y, self.rect.width, self.rect.height)?;
        } else {
            if let Some(ref left) = self.left {
                left.apply_geometry()?;
            }
            if let Some(ref right) = self.right {
                right.apply_geometry()?;
            }
        }
        Ok(())
    }
}
```

**Day 11-14: Window Manager Core**

**crates/core/src/window_manager/mod.rs:**
```rust
pub mod tree;
pub mod layout;
pub mod window;
pub mod monitor;

use crate::utils::win32::{WindowHandle, enumerate_windows};
use tree::{TreeNode, Rect, Split};
use std::collections::HashMap;

pub struct WindowManager {
    trees: HashMap<usize, TreeNode>, // workspace_id -> tree
    active_workspace: usize,
    monitors: Vec<Monitor>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            active_workspace: 1,
            monitors: Vec::new(),
        }
    }
    
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        // Enumerate monitors
        self.refresh_monitors()?;
        
        // Create initial workspace
        let monitor = &self.monitors[0];
        let root = TreeNode::new_container(
            Split::Vertical,
            monitor.work_area,
        );
        self.trees.insert(self.active_workspace, root);
        
        Ok(())
    }
    
    pub fn manage_window(&mut self, window: WindowHandle) -> anyhow::Result<()> {
        // Filter out windows we shouldn't manage
        if !self.should_manage_window(&window)? {
            return Ok(());
        }
        
        // Add to current workspace tree
        if let Some(tree) = self.trees.get_mut(&self.active_workspace) {
            tree.insert_window(window, Split::Vertical);
            tree.apply_geometry()?;
        }
        
        Ok(())
    }
    
    pub fn tile_workspace(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if let Some(tree) = self.trees.get(& workspace_id) {
            tree.apply_geometry()?;
        }
        Ok(())
    }
    
    fn should_manage_window(&self, window: &WindowHandle) -> anyhow::Result<bool> {
        // Check if window is visible
        if !window.is_visible() {
            return Ok(false);
        }
        
        // Check window styles to filter out popups, tooltips, etc.
        // Implementation needed
        
        Ok(true)
    }
    
    fn refresh_monitors(&mut self) -> anyhow::Result<()> {
        // Use EnumDisplayMonitors to get monitor list
        // Implementation needed
        Ok(())
    }
}

pub struct Monitor {
    pub id: usize,
    pub name: String,
    pub work_area: Rect,
    pub dpi_scale: f32,
}
```

### Week 3: Event Loop & Message Processing

**Day 15-17: Windows Event Hook**

**crates/core/src/event_loop.rs:**
```rust
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;
use std::sync::mpsc::{channel, Sender, Receiver};

pub enum WindowEvent {
    WindowCreated(HWND),
    WindowDestroyed(HWND),
    WindowMoved(HWND),
    WindowFocused(HWND),
    MonitorChanged,
}

pub struct EventLoop {
    event_tx: Sender<WindowEvent>,
    event_rx: Receiver<WindowEvent>,
    hook: Option<HWINEVENTHOOK>,
}

impl EventLoop {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            event_tx: tx,
            event_rx: rx,
            hook: None,
        }
    }
    
    pub fn start(&mut self) -> anyhow::Result<()> {
        unsafe {
            let hook = SetWinEventHook(
                EVENT_OBJECT_CREATE,
                EVENT_OBJECT_DESTROY,
                HMODULE::default(),
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );
            
            if hook.0 == 0 {
                anyhow::bail!("Failed to set Windows event hook");
            }
            
            self.hook = Some(hook);
        }
        
        Ok(())
    }
    
    pub fn poll_events(&self) -> impl Iterator<Item = WindowEvent> + '_ {
        self.event_rx.try_iter()
    }
    
    pub fn run(&self) -> anyhow::Result<()> {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        Ok(())
    }
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    // Send events to channel
    // Implementation needed - store channel in TLS or global
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        if let Some(hook) = self.hook {
            unsafe {
                UnhookWinEvent(hook);
            }
        }
    }
}
```

**Day 18-21: Main Application Loop**

**crates/core/src/main.rs:**
```rust
mod window_manager;
mod event_loop;
mod utils;

use window_manager::WindowManager;
use event_loop::{EventLoop, WindowEvent};
use tracing::{info, error};

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("tiling_wm_core=debug")
        .init();
    
    info!("Starting Tiling Window Manager");
    
    // Initialize window manager
    let mut wm = WindowManager::new();
    wm.initialize()?;
    
    // Set up event loop
    let mut event_loop = EventLoop::new();
    event_loop.start()?;
    
    info!("Window manager initialized, entering main loop");
    
    // Main event loop
    loop {
        // Process Windows events
        for event in event_loop.poll_events() {
            match event {
                WindowEvent::WindowCreated(hwnd) => {
                    info!("Window created: {:?}", hwnd);
                    let window = utils::win32::WindowHandle::from_hwnd(hwnd);
                    if let Err(e) = wm.manage_window(window) {
                        error!("Failed to manage window: {}", e);
                    }
                }
                WindowEvent::WindowDestroyed(hwnd) => {
                    info!("Window destroyed: {:?}", hwnd);
                    // Handle window removal
                }
                WindowEvent::WindowFocused(hwnd) => {
                    info!("Window focused: {:?}", hwnd);
                }
                _ => {}
            }
        }
        
        // Small sleep to avoid busy loop
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
```

### Deliverables for Phase 1

**Code:**
- [x] Project structure with workspace
- [x] Windows API wrapper utilities
- [x] Binary tree data structure
- [x] Basic window manager core
- [x] Event loop with Windows hooks
- [x] Main application entry point

**Documentation:**
- [x] README with build instructions
- [x] Architecture documentation
- [x] Code comments for public APIs

**Testing:**
- [x] Unit tests for tree operations
- [x] Unit tests for Rect splitting
- [x] Integration test for window enumeration

**Validation:**
- [x] Application compiles without errors
- [x] Can enumerate windows
- [x] Can detect window creation events
- [x] Basic logging works

---

## Phase 2: Core Window Management (Weeks 4-8)

### Objectives
- Implement complete tiling algorithms
- Handle window lifecycle (create, destroy, move)
- Implement focus management
- Support floating windows
- Add basic window operations

### Week 4: Dwindle Layout Algorithm

**crates/core/src/window_manager/layout/dwindle.rs:**
```rust
use crate::window_manager::tree::{TreeNode, Rect, Split};
use crate::utils::win32::WindowHandle;

pub struct DwindleLayout {
    pub ratio: f32,
    pub smart_split: bool,
}

impl Default for DwindleLayout {
    fn default() -> Self {
        Self {
            ratio: 0.5,
            smart_split: true,
        }
    }
}

impl DwindleLayout {
    pub fn calculate_split_direction(&self, rect: &Rect) -> Split {
        if self.smart_split {
            if rect.width > rect.height {
                Split::Horizontal
            } else {
                Split::Vertical
            }
        } else {
            Split::Vertical
        }
    }
    
    pub fn insert_window(
        &self,
        tree: &mut TreeNode,
        window: WindowHandle,
    ) -> anyhow::Result<()> {
        let split = self.calculate_split_direction(&tree.rect);
        tree.insert_window(window, split);
        Ok(())
    }
    
    pub fn remove_window(
        &self,
        tree: &mut TreeNode,
        window: &WindowHandle,
    ) -> anyhow::Result<bool> {
        // Recursive search and removal
        // Returns true if window was found and removed
        todo!("Implement window removal from tree")
    }
}
```

### Week 5: Master Layout Algorithm

**crates/core/src/window_manager/layout/master.rs:**
```rust
use crate::window_manager::tree::{TreeNode, Rect};
use crate::utils::win32::WindowHandle;

pub struct MasterLayout {
    pub master_factor: f32,
    pub master_count: usize,
}

impl Default for MasterLayout {
    fn default() -> Self {
        Self {
            master_factor: 0.55,
            master_count: 1,
        }
    }
}

impl MasterLayout {
    pub fn apply(&self, windows: &[WindowHandle], area: Rect) -> anyhow::Result<()> {
        if windows.is_empty() {
            return Ok(());
        }
        
        if windows.len() == 1 {
            // Single window takes full area
            windows[0].set_pos(area.x, area.y, area.width, area.height)?;
            return Ok(());
        }
        
        // Split into master and stack
        let master_width = (area.width as f32 * self.master_factor) as i32;
        let stack_width = area.width - master_width;
        
        // Master area
        let master_rect = Rect::new(area.x, area.y, master_width, area.height);
        windows[0].set_pos(
            master_rect.x,
            master_rect.y,
            master_rect.width,
            master_rect.height,
        )?;
        
        // Stack area
        let stack_count = windows.len() - 1;
        let stack_height = area.height / stack_count as i32;
        
        for (i, window) in windows[1..].iter().enumerate() {
            let y = area.y + (i as i32 * stack_height);
            window.set_pos(
                area.x + master_width,
                y,
                stack_width,
                stack_height,
            )?;
        }
        
        Ok(())
    }
}
```

### Week 6: Window State Management

**crates/core/src/window_manager/window.rs:**
```rust
use crate::utils::win32::WindowHandle;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Tiled,
    Floating,
    Fullscreen,
    Minimized,
}

#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub handle: WindowHandle,
    pub state: WindowState,
    pub workspace: usize,
    pub monitor: usize,
    pub title: String,
    pub class: String,
    pub process_name: String,
    pub original_rect: Option<windows::Win32::Foundation::RECT>,
}

impl ManagedWindow {
    pub fn new(handle: WindowHandle, workspace: usize, monitor: usize) -> anyhow::Result<Self> {
        let title = handle.get_title()?;
        let class = handle.get_class_name()?;
        let process_name = handle.get_process_name().unwrap_or_default();
        
        Ok(Self {
            handle,
            state: WindowState::Tiled,
            workspace,
            monitor,
            title,
            class,
            process_name,
            original_rect: None,
        })
    }
    
    pub fn set_floating(&mut self) -> anyhow::Result<()> {
        if self.state != WindowState::Floating {
            // Save current position before making floating
            self.original_rect = Some(self.handle.get_rect()?);
            self.state = WindowState::Floating;
        }
        Ok(())
    }
    
    pub fn set_tiled(&mut self) -> anyhow::Result<()> {
        self.state = WindowState::Tiled;
        self.original_rect = None;
        Ok(())
    }
    
    pub fn set_fullscreen(&mut self, monitor_rect: &crate::window_manager::tree::Rect) -> anyhow::Result<()> {
        if self.state != WindowState::Fullscreen {
            self.original_rect = Some(self.handle.get_rect()?);
            self.state = WindowState::Fullscreen;
            self.handle.set_pos(
                monitor_rect.x,
                monitor_rect.y,
                monitor_rect.width,
                monitor_rect.height,
            )?;
        }
        Ok(())
    }
}

pub struct WindowRegistry {
    windows: HashMap<isize, ManagedWindow>, // HWND value -> ManagedWindow
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, window: ManagedWindow) {
        self.windows.insert(window.handle.0.0, window);
    }
    
    pub fn unregister(&mut self, hwnd: isize) -> Option<ManagedWindow> {
        self.windows.remove(&hwnd)
    }
    
    pub fn get(&self, hwnd: isize) -> Option<&ManagedWindow> {
        self.windows.get(&hwnd)
    }
    
    pub fn get_mut(&mut self, hwnd: isize) -> Option<&mut ManagedWindow> {
        self.windows.get_mut(&hwnd)
    }
    
    pub fn get_by_workspace(&self, workspace: usize) -> Vec<&ManagedWindow> {
        self.windows
            .values()
            .filter(|w| w.workspace == workspace)
            .collect()
    }
}
```

### Week 7: Focus Management

**crates/core/src/window_manager/focus.rs:**
```rust
use crate::utils::win32::WindowHandle;
use windows::Win32::UI::WindowsAndMessaging::*;
use std::collections::VecDeque;

pub struct FocusManager {
    focus_history: VecDeque<isize>, // HWND values
    current_focus: Option<isize>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focus_history: VecDeque::with_capacity(10),
            current_focus: None,
        }
    }
    
    pub fn focus_window(&mut self, window: &WindowHandle) -> anyhow::Result<()> {
        unsafe {
            SetForegroundWindow(window.0)?;
        }
        
        let hwnd_val = window.0.0;
        
        // Update history
        if let Some(idx) = self.focus_history.iter().position(|&h| h == hwnd_val) {
            self.focus_history.remove(idx);
        }
        
        self.focus_history.push_front(hwnd_val);
        
        // Keep history limited
        if self.focus_history.len() > 10 {
            self.focus_history.pop_back();
        }
        
        self.current_focus = Some(hwnd_val);
        
        Ok(())
    }
    
    pub fn focus_previous(&mut self) -> Option<isize> {
        if self.focus_history.len() > 1 {
            self.focus_history.get(1).copied()
        } else {
            None
        }
    }
    
    pub fn current(&self) -> Option<isize> {
        self.current_focus
    }
}
```

### Week 8: Window Operations & Commands

**crates/core/src/commands.rs:**
```rust
use crate::window_manager::WindowManager;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum Command {
    // Window commands
    CloseActiveWindow,
    ToggleFloating,
    ToggleFullscreen,
    
    // Focus commands
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    
    // Move commands
    MoveWindowLeft,
    MoveWindowRight,
    MoveWindowUp,
    MoveWindowDown,
    
    // Workspace commands
    SwitchWorkspace(usize),
    MoveToWorkspace(usize),
    
    // Layout commands
    SetLayout(LayoutType),
    
    // System commands
    Reload,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    Dwindle,
    Master,
}

pub struct CommandExecutor {
    wm: WindowManager,
}

impl CommandExecutor {
    pub fn new(wm: WindowManager) -> Self {
        Self { wm }
    }
    
    pub fn execute(&mut self, command: Command) -> Result<()> {
        match command {
            Command::CloseActiveWindow => self.close_active_window(),
            Command::ToggleFloating => self.toggle_floating(),
            Command::ToggleFullscreen => self.toggle_fullscreen(),
            Command::FocusLeft => self.focus_direction(Direction::Left),
            Command::FocusRight => self.focus_direction(Direction::Right),
            Command::FocusUp => self.focus_direction(Direction::Up),
            Command::FocusDown => self.focus_direction(Direction::Down),
            Command::SwitchWorkspace(id) => self.switch_workspace(id),
            Command::MoveToWorkspace(id) => self.move_to_workspace(id),
            _ => Ok(()),
        }
    }
    
    fn close_active_window(&mut self) -> Result<()> {
        // Get active window and close it
        todo!()
    }
    
    fn toggle_floating(&mut self) -> Result<()> {
        // Toggle floating state of active window
        todo!()
    }
    
    fn toggle_fullscreen(&mut self) -> Result<()> {
        // Toggle fullscreen state of active window
        todo!()
    }
    
    fn focus_direction(&mut self, direction: Direction) -> Result<()> {
        // Focus window in the given direction
        todo!()
    }
    
    fn switch_workspace(&mut self, id: usize) -> Result<()> {
        // Switch to workspace
        todo!()
    }
    
    fn move_to_workspace(&mut self, id: usize) -> Result<()> {
        // Move active window to workspace
        todo!()
    }
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
```

### Deliverables for Phase 2

- [x] Complete dwindle layout implementation
- [x] Master layout implementation
- [x] Window state management (tiled/floating/fullscreen)
- [x] Focus management system
- [x] Window registry for tracking managed windows
- [x] Command system for window operations
- [x] Unit tests for all layout algorithms
- [x] Integration tests for window management

**Validation:**
- Can tile 2+ windows in dwindle layout
- Can switch to master layout
- Can toggle window between tiled and floating
- Can focus windows in all directions
- Can move windows in the tree

---

## Phase 3: Workspace System (Weeks 9-12)

### Objectives
- Integrate with Windows Virtual Desktop API
- Implement workspace switching and management
- Support per-monitor workspaces
- Implement workspace persistence

### Week 9: Virtual Desktop Integration

**crates/core/src/workspace/virtual_desktop.rs:**
```rust
use windows::Win32::System::Com::*;
use windows::core::*;

// Virtual Desktop COM interfaces (Windows 10/11)
// Note: These are undocumented APIs, requires reverse engineering or using third-party definitions

pub struct VirtualDesktopManager {
    manager: Option<IVirtualDesktopManager>,
}

impl VirtualDesktopManager {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
            
            let manager: IVirtualDesktopManager = CoCreateInstance(
                &CLSID_VirtualDesktopManager,
                None,
                CLSCTX_ALL,
            )?;
            
            Ok(Self {
                manager: Some(manager),
            })
        }
    }
    
    pub fn get_desktop_count(&self) -> Result<usize> {
        // Use IVirtualDesktopManagerInternal to get count
        // Implementation requires accessing internal Windows APIs
        todo!("Implement using VirtualDesktop COM APIs")
    }
    
    pub fn switch_desktop(&self, index: usize) -> Result<()> {
        // Switch to desktop by index
        todo!("Implement desktop switching")
    }
    
    pub fn move_window_to_desktop(&self, hwnd: HWND, desktop_id: GUID) -> Result<()> {
        unsafe {
            if let Some(ref mgr) = self.manager {
                mgr.MoveWindowToDesktop(hwnd, &desktop_id)?;
            }
        }
        Ok(())
    }
}
```

### Week 10: Workspace Manager

**crates/core/src/workspace/manager.rs:**
```rust
use crate::window_manager::tree::TreeNode;
use crate::window_manager::window::ManagedWindow;
use std::collections::HashMap;

pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub tree: TreeNode,
    pub windows: Vec<isize>, // HWND values
}

pub struct WorkspaceManager {
    workspaces: HashMap<usize, Workspace>,
    active_workspace: usize,
    next_id: usize,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            active_workspace: 1,
            next_id: 1,
        }
    }
    
    pub fn create_workspace(&mut self, name: String, monitor: usize, area: crate::window_manager::tree::Rect) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let workspace = Workspace {
            id,
            name,
            monitor,
            tree: TreeNode::new_container(crate::window_manager::tree::Split::Vertical, area),
            windows: Vec::new(),
        };
        
        self.workspaces.insert(id, workspace);
        id
    }
    
    pub fn switch_to(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if !self.workspaces.contains_key(&workspace_id) {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
        
        // Hide windows from current workspace
        if let Some(current) = self.workspaces.get(&self.active_workspace) {
            for hwnd in &current.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    ShowWindow(HWND(*hwnd), SW_HIDE);
                }
            }
        }
        
        // Show windows from target workspace
        if let Some(target) = self.workspaces.get(&workspace_id) {
            for hwnd in &target.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    ShowWindow(HWND(*hwnd), SW_SHOW);
                }
            }
            
            // Re-tile windows
            target.tree.apply_geometry()?;
        }
        
        self.active_workspace = workspace_id;
        Ok(())
    }
    
    pub fn add_window_to_workspace(&mut self, workspace_id: usize, hwnd: isize) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            if !workspace.windows.contains(&hwnd) {
                workspace.windows.push(hwnd);
            }
        }
        Ok(())
    }
    
    pub fn remove_window_from_workspace(&mut self, workspace_id: usize, hwnd: isize) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.windows.retain(|&w| w != hwnd);
        }
        Ok(())
    }
    
    pub fn move_window_to_workspace(&mut self, hwnd: isize, from: usize, to: usize) -> anyhow::Result<()> {
        self.remove_window_from_workspace(from, hwnd)?;
        self.add_window_to_workspace(to, hwnd)?;
        
        // Hide window if moving away from active workspace
        if from == self.active_workspace && to != self.active_workspace {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(HWND(hwnd), SW_HIDE);
            }
        }
        
        // Show window if moving to active workspace
        if to == self.active_workspace && from != self.active_workspace {
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                ShowWindow(HWND(hwnd), SW_SHOW);
            }
        }
        
        Ok(())
    }
    
    pub fn get_active(&self) -> usize {
        self.active_workspace
    }
    
    pub fn get_workspace(&self, id: usize) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }
    
    pub fn get_workspace_mut(&mut self, id: usize) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&id)
    }
}
```

### Week 11: Per-Monitor Workspace Support

**crates/core/src/window_manager/monitor.rs:**
```rust
use crate::window_manager::tree::Rect;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::*;
use std::collections::HashMap;

pub struct MonitorInfo {
    pub id: usize,
    pub handle: HMONITOR,
    pub name: String,
    pub work_area: Rect,
    pub full_area: Rect,
    pub dpi_scale: f32,
    pub workspaces: Vec<usize>, // Workspace IDs
}

pub struct MonitorManager {
    monitors: HashMap<usize, MonitorInfo>,
    next_id: usize,
}

impl MonitorManager {
    pub fn new() -> Self {
        Self {
            monitors: HashMap::new(),
            next_id: 0,
        }
    }
    
    pub fn refresh(&mut self) -> anyhow::Result<()> {
        self.monitors.clear();
        
        unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(monitor_enum_proc),
                LPARAM(self as *mut _ as isize),
            )?;
        }
        
        Ok(())
    }
    
    pub fn get_primary(&self) -> Option<&MonitorInfo> {
        self.monitors.get(&0)
    }
    
    pub fn get_by_id(&self, id: usize) -> Option<&MonitorInfo> {
        self.monitors.get(&id)
    }
    
    pub fn get_monitor_at_point(&self, x: i32, y: i32) -> Option<&MonitorInfo> {
        unsafe {
            let point = POINT { x, y };
            let hmonitor = MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST);
            
            self.monitors
                .values()
                .find(|m| m.handle == hmonitor)
        }
    }
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let manager = &mut *(lparam.0 as *mut MonitorManager);
    
    let mut monitor_info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    
    if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
        let id = manager.next_id;
        manager.next_id += 1;
        
        let work_area = Rect {
            x: monitor_info.rcWork.left,
            y: monitor_info.rcWork.top,
            width: monitor_info.rcWork.right - monitor_info.rcWork.left,
            height: monitor_info.rcWork.bottom - monitor_info.rcWork.top,
        };
        
        let full_area = Rect {
            x: monitor_info.rcMonitor.left,
            y: monitor_info.rcMonitor.top,
            width: monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
            height: monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
        };
        
        // Get DPI
        let mut dpi_x = 0u32;
        let mut dpi_y = 0u32;
        let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        let dpi_scale = dpi_x as f32 / 96.0;
        
        let info = MonitorInfo {
            id,
            handle: hmonitor,
            name: format!("Monitor {}", id),
            work_area,
            full_area,
            dpi_scale,
            workspaces: Vec::new(),
        };
        
        manager.monitors.insert(id, info);
    }
    
    true.into()
}
```

### Week 12: Workspace Persistence

**crates/core/src/workspace/persistence.rs:**
```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct WorkspaceState {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub windows: Vec<WindowState>,
}

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub process_name: String,
    pub title: String,
    pub workspace: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SessionState {
    pub workspaces: Vec<WorkspaceState>,
    pub active_workspace: usize,
}

pub struct PersistenceManager {
    state_file: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Self {
        let state_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiling-wm");
        
        fs::create_dir_all(&state_dir).ok();
        
        Self {
            state_file: state_dir.join("session.json"),
        }
    }
    
    pub fn save_state(&self, state: &SessionState) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, json)?;
        Ok(())
    }
    
    pub fn load_state(&self) -> anyhow::Result<SessionState> {
        let json = fs::read_to_string(&self.state_file)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }
    
    pub fn clear_state(&self) -> anyhow::Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }
        Ok(())
    }
}
```

### Deliverables for Phase 3

- [x] Virtual Desktop API integration
- [x] Workspace manager with create/switch/delete
- [x] Per-monitor workspace support
- [x] Workspace persistence across sessions
- [x] Monitor manager with DPI awareness
- [x] Integration tests for workspace operations
- [x] Documentation for workspace features

**Validation:**
- Can create and switch between 10 workspaces
- Windows are hidden/shown when switching workspaces
- Per-monitor workspaces work correctly
- Session state persists across restarts
- Multi-monitor with different DPI works

---

## Phase 4: Configuration & Rules (Weeks 13-16)

### Objectives
- Implement TOML configuration parsing
- Support window rules
- Hot-reload configuration
- Create default configuration

### Week 13: Configuration Schema

**crates/core/src/config/schema.rs:**
```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub decoration: DecorationConfig,
    pub animations: AnimationsConfig,
    pub input: InputConfig,
    pub layouts: LayoutsConfig,
    
    #[serde(default)]
    pub window_rules: Vec<WindowRule>,
    
    #[serde(default)]
    pub workspace_rules: Vec<WorkspaceRule>,
    
    #[serde(default)]
    pub keybinds: Vec<Keybind>,
    
    #[serde(default)]
    pub monitors: Vec<MonitorConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralConfig {
    #[serde(default = "default_gaps_in")]
    pub gaps_in: i32,
    
    #[serde(default = "default_gaps_out")]
    pub gaps_out: i32,
    
    #[serde(default = "default_border_size")]
    pub border_size: i32,
    
    #[serde(default)]
    pub active_border_color: String,
    
    #[serde(default)]
    pub inactive_border_color: String,
}

fn default_gaps_in() -> i32 { 5 }
fn default_gaps_out() -> i32 { 10 }
fn default_border_size() -> i32 { 2 }

#[derive(Debug, Deserialize, Serialize)]
pub struct DecorationConfig {
    #[serde(default = "default_rounding")]
    pub rounding: i32,
    
    #[serde(default = "default_opacity")]
    pub active_opacity: f32,
    
    #[serde(default = "default_inactive_opacity")]
    pub inactive_opacity: f32,
}

fn default_rounding() -> i32 { 10 }
fn default_opacity() -> f32 { 1.0 }
fn default_inactive_opacity() -> f32 { 0.9 }

#[derive(Debug, Deserialize, Serialize)]
pub struct AnimationsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    #[serde(default = "default_animation_speed")]
    pub speed: f32,
}

fn default_true() -> bool { true }
fn default_animation_speed() -> f32 { 1.0 }

#[derive(Debug, Deserialize, Serialize)]
pub struct InputConfig {
    #[serde(default)]
    pub repeat_rate: u32,
    
    #[serde(default)]
    pub repeat_delay: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayoutsConfig {
    pub dwindle: DwindleConfig,
    pub master: MasterConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DwindleConfig {
    #[serde(default = "default_true")]
    pub smart_split: bool,
    
    #[serde(default = "default_false")]
    pub no_gaps_when_only: bool,
}

fn default_false() -> bool { false }

#[derive(Debug, Deserialize, Serialize)]
pub struct MasterConfig {
    #[serde(default = "default_master_factor")]
    pub master_factor: f32,
    
    #[serde(default = "default_master_count")]
    pub master_count: usize,
}

fn default_master_factor() -> f32 { 0.55 }
fn default_master_count() -> usize { 1 }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WindowRule {
    pub match_process: Option<String>,
    pub match_title: Option<String>,
    pub match_class: Option<String>,
    pub actions: Vec<RuleAction>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Float,
    Tile,
    Workspace(usize),
    Monitor(usize),
    Fullscreen,
    NoFocus,
    NoManage,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceRule {
    pub id: usize,
    pub monitor: usize,
    #[serde(default)]
    pub default: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Keybind {
    pub modifiers: Vec<String>,
    pub key: String,
    pub command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitorConfig {
    pub name: String,
    pub resolution: Option<String>,
    pub position: Option<String>,
    pub scale: Option<f32>,
}
```

### Week 14: Configuration Parser & Loader

**crates/core/src/config/parser.rs:**
```rust
use super::schema::Config;
use std::path::PathBuf;
use std::fs;
use anyhow::Context;

pub struct ConfigLoader {
    config_path: PathBuf,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiling-wm");
        
        fs::create_dir_all(&config_dir).ok();
        
        Self {
            config_path: config_dir.join("config.toml"),
        }
    }
    
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            config_path: path,
        }
    }
    
    pub fn load(&self) -> anyhow::Result<Config> {
        if !self.config_path.exists() {
            // Create default config
            self.create_default_config()?;
        }
        
        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn create_default_config(&self) -> anyhow::Result<()> {
        let default_config = include_str!("../../config/default_config.toml");
        fs::write(&self.config_path, default_config)?;
        Ok(())
    }
    
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}
```

**config/default_config.toml:**
```toml
# Tiling Window Manager Configuration

[general]
gaps_in = 5
gaps_out = 10
border_size = 2
active_border_color = "#89b4fa"
inactive_border_color = "#585b70"

[decoration]
rounding = 10
active_opacity = 1.0
inactive_opacity = 0.9

[animations]
enabled = true
speed = 1.0

[input]
repeat_rate = 25
repeat_delay = 600

[layouts.dwindle]
smart_split = true
no_gaps_when_only = false

[layouts.master]
master_factor = 0.55
master_count = 1

# Window Rules
[[window_rules]]
match_process = "notepad.exe"
actions = ["float"]

[[window_rules]]
match_process = "firefox.exe"
actions = [{ workspace = 2 }]

[[window_rules]]
match_process = "steam.exe"
actions = ["float", { workspace = 3 }]

# Workspace Rules
[[workspace_rules]]
id = 1
monitor = 0
default = true

[[workspace_rules]]
id = 2
monitor = 0

# Keybindings
[[keybinds]]
modifiers = ["Win"]
key = "q"
command = "close"

[[keybinds]]
modifiers = ["Win"]
key = "v"
command = "toggle-floating"

[[keybinds]]
modifiers = ["Win"]
key = "f"
command = "toggle-fullscreen"

[[keybinds]]
modifiers = ["Win"]
key = "Left"
command = "focus-left"

[[keybinds]]
modifiers = ["Win"]
key = "Right"
command = "focus-right"

[[keybinds]]
modifiers = ["Win"]
key = "Up"
command = "focus-up"

[[keybinds]]
modifiers = ["Win"]
key = "Down"
command = "focus-down"

[[keybinds]]
modifiers = ["Win"]
key = "1"
command = "workspace-1"

[[keybinds]]
modifiers = ["Win"]
key = "2"
command = "workspace-2"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "1"
command = "move-to-workspace-1"

[[keybinds]]
modifiers = ["Win", "Shift"]
key = "2"
command = "move-to-workspace-2"
```

### Week 15: Window Rules Engine

**crates/core/src/rules/matcher.rs:**
```rust
use crate::config::schema::{WindowRule, RuleAction};
use crate::window_manager::window::ManagedWindow;
use regex::Regex;

pub struct RuleMatcher {
    rules: Vec<CompiledRule>,
}

struct CompiledRule {
    process_regex: Option<Regex>,
    title_regex: Option<Regex>,
    class_regex: Option<Regex>,
    actions: Vec<RuleAction>,
}

impl RuleMatcher {
    pub fn new(rules: Vec<WindowRule>) -> anyhow::Result<Self> {
        let mut compiled_rules = Vec::new();
        
        for rule in rules {
            let process_regex = if let Some(ref pattern) = rule.match_process {
                Some(Regex::new(pattern)?)
            } else {
                None
            };
            
            let title_regex = if let Some(ref pattern) = rule.match_title {
                Some(Regex::new(pattern)?)
            } else {
                None
            };
            
            let class_regex = if let Some(ref pattern) = rule.match_class {
                Some(Regex::new(pattern)?)
            } else {
                None
            };
            
            compiled_rules.push(CompiledRule {
                process_regex,
                title_regex,
                class_regex,
                actions: rule.actions.clone(),
            });
        }
        
        Ok(Self {
            rules: compiled_rules,
        })
    }
    
    pub fn match_window(&self, window: &ManagedWindow) -> Vec<RuleAction> {
        let mut actions = Vec::new();
        
        for rule in &self.rules {
            let mut matches = true;
            
            if let Some(ref regex) = rule.process_regex {
                if !regex.is_match(&window.process_name) {
                    matches = false;
                }
            }
            
            if let Some(ref regex) = rule.title_regex {
                if !regex.is_match(&window.title) {
                    matches = false;
                }
            }
            
            if let Some(ref regex) = rule.class_regex {
                if !regex.is_match(&window.class) {
                    matches = false;
                }
            }
            
            if matches {
                actions.extend(rule.actions.clone());
            }
        }
        
        actions
    }
    
    pub fn should_manage(&self, window: &ManagedWindow) -> bool {
        let actions = self.match_window(window);
        !actions.iter().any(|a| matches!(a, RuleAction::NoManage))
    }
}
```

### Week 16: Configuration Hot-Reload

**crates/core/src/config/watcher.rs:**
```rust
use notify::{Watcher, RecursiveMode, Result, Event};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct ConfigWatcher {
    _watcher: Box<dyn Watcher>,
    receiver: Receiver<Result<Event>>,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = channel();
        
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        Ok(Self {
            _watcher: Box::new(watcher),
            receiver: rx,
        })
    }
    
    pub fn check_for_changes(&self) -> bool {
        self.receiver
            .try_iter()
            .any(|event| {
                if let Ok(event) = event {
                    matches!(event.kind, notify::EventKind::Modify(_))
                } else {
                    false
                }
            })
    }
}
```

### Deliverables for Phase 4

- [x] Complete configuration schema
- [x] TOML configuration parser
- [x] Default configuration file
- [x] Window rules engine with regex matching
- [x] Configuration hot-reload system
- [x] Documentation for all config options
- [x] Example configurations

**Validation:**
- Config file is parsed correctly
- Window rules are applied properly
- Hot-reload works without restart
- Invalid config shows helpful error messages
- All config options are documented

---

## Phase 5: IPC & CLI (Weeks 17-20)

### Objectives
- Implement named pipe IPC server
- Create JSON protocol
- Build CLI client
- Support event subscription
- Enable scripting

### Week 17: IPC Protocol Design

**crates/core/src/ipc/protocol.rs:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    // Query requests
    GetActiveWindow,
    GetWindows,
    GetWorkspaces,
    GetMonitors,
    GetConfig,
    
    // Command requests
    Execute {
        command: String,
        args: Vec<String>,
    },
    
    // Subscription
    Subscribe {
        events: Vec<String>,
    },
    Unsubscribe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Success {
        data: serde_json::Value,
    },
    Error {
        message: String,
    },
    Event {
        name: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: String,
    pub title: String,
    pub class: String,
    pub process_name: String,
    pub workspace: usize,
    pub monitor: usize,
    pub state: String,
    pub rect: RectInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RectInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub window_count: usize,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: usize,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
}
```

### Week 18: Named Pipe Server

**crates/core/src/ipc/server.rs:**
```rust
use super::protocol::{Request, Response};
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IpcServer {
    pipe_name: String,
    handlers: Arc<Mutex<Vec<ClientHandler>>>,
}

impl IpcServer {
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub async fn start(&self) -> anyhow::Result<()> {
        loop {
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&self.pipe_name)?;
            
            let handlers = self.handlers.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(server, handlers).await {
                    eprintln!("Client handler error: {}", e);
                }
            });
        }
    }
    
    async fn handle_client(
        mut server: NamedPipeServer,
        _handlers: Arc<Mutex<Vec<ClientHandler>>>,
    ) -> anyhow::Result<()> {
        server.connect().await?;
        
        let mut buffer = vec![0u8; 4096];
        
        loop {
            let n = server.read(&mut buffer).await?;
            
            if n == 0 {
                break;
            }
            
            let request: Request = serde_json::from_slice(&buffer[..n])?;
            let response = Self::process_request(request).await?;
            let response_data = serde_json::to_vec(&response)?;
            
            server.write_all(&response_data).await?;
        }
        
        Ok(())
    }
    
    async fn process_request(request: Request) -> anyhow::Result<Response> {
        match request {
            Request::GetWindows => {
                // Get window list from window manager
                // Return as JSON
                Ok(Response::Success {
                    data: serde_json::json!([]),
                })
            }
            Request::Execute { command, args } => {
                // Execute command
                Ok(Response::Success {
                    data: serde_json::json!({"status": "executed"}),
                })
            }
            _ => {
                Ok(Response::Error {
                    message: "Not implemented".to_string(),
                })
            }
        }
    }
}

struct ClientHandler {
    // Handle for sending events to clients
}
```

### Week 19: CLI Client Implementation

**crates/cli/src/main.rs:**
```rust
use clap::{Parser, Subcommand};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "tiling-wm-cli")]
#[command(about = "CLI client for Tiling Window Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get list of windows
    Windows,
    
    /// Get list of workspaces
    Workspaces,
    
    /// Get list of monitors
    Monitors,
    
    /// Execute a command
    Exec {
        /// Command to execute
        command: String,
        
        /// Command arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Switch to workspace
    Workspace {
        /// Workspace ID
        id: usize,
    },
    
    /// Close active window
    Close,
    
    /// Toggle floating for active window
    ToggleFloat,
    
    /// Toggle fullscreen for active window
    ToggleFullscreen,
    
    /// Reload configuration
    Reload,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let mut client = ClientOptions::new()
        .open(r"\\.\pipe\tiling-wm")?;
    
    let request = match cli.command {
        Commands::Windows => {
            serde_json::json!({
                "type": "get_windows"
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
        Commands::Exec { command, args } => {
            serde_json::json!({
                "type": "execute",
                "command": command,
                "args": args,
            })
        }
        Commands::Workspace { id } => {
            serde_json::json!({
                "type": "execute",
                "command": "workspace",
                "args": [id.to_string()],
            })
        }
        Commands::Close => {
            serde_json::json!({
                "type": "execute",
                "command": "close",
                "args": [],
            })
        }
        Commands::ToggleFloat => {
            serde_json::json!({
                "type": "execute",
                "command": "toggle-floating",
                "args": [],
            })
        }
        Commands::ToggleFullscreen => {
            serde_json::json!({
                "type": "execute",
                "command": "toggle-fullscreen",
                "args": [],
            })
        }
        Commands::Reload => {
            serde_json::json!({
                "type": "execute",
                "command": "reload",
                "args": [],
            })
        }
    };
    
    let request_data = serde_json::to_vec(&request)?;
    client.write_all(&request_data).await?;
    
    let mut buffer = vec![0u8; 4096];
    let n = client.read(&mut buffer).await?;
    
    let response: Value = serde_json::from_slice(&buffer[..n])?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    
    Ok(())
}
```

### Week 20: Event System

**crates/core/src/ipc/events.rs:**
```rust
use super::protocol::Response;
use tokio::sync::broadcast::{channel, Sender, Receiver};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum Event {
    WindowCreated { hwnd: isize },
    WindowClosed { hwnd: isize },
    WindowFocused { hwnd: isize },
    WorkspaceChanged { from: usize, to: usize },
    MonitorChanged,
    ConfigReloaded,
}

pub struct EventBroadcaster {
    sender: Sender<Event>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = channel(100);
        Self {
            sender: tx,
        }
    }
    
    pub fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Event {
    pub fn to_response(&self) -> Response {
        let (name, data) = match self {
            Event::WindowCreated { hwnd } => {
                ("window_created", serde_json::json!({ "hwnd": hwnd }))
            }
            Event::WindowClosed { hwnd } => {
                ("window_closed", serde_json::json!({ "hwnd": hwnd }))
            }
            Event::WindowFocused { hwnd } => {
                ("window_focused", serde_json::json!({ "hwnd": hwnd }))
            }
            Event::WorkspaceChanged { from, to } => {
                ("workspace_changed", serde_json::json!({ "from": from, "to": to }))
            }
            Event::MonitorChanged => {
                ("monitor_changed", serde_json::json!({}))
            }
            Event::ConfigReloaded => {
                ("config_reloaded", serde_json::json!({}))
            }
        };
        
        Response::Event {
            name: name.to_string(),
            data,
        }
    }
}
```

### Deliverables for Phase 5

- [x] JSON-based IPC protocol
- [x] Named pipe server implementation
- [x] CLI client with all commands
- [x] Event subscription system
- [x] Documentation for IPC protocol
- [x] Example scripts using CLI

**Validation:**
- CLI can query window manager state
- CLI can execute commands
- Events are broadcasted correctly
- Multiple clients can connect
- Error handling is robust

---

## Phase 6: Status Bar Implementation (Weeks 21-26)

### Objectives
- Create separate status bar application
- Implement modular widget system
- Support custom modules
- Provide CSS-like styling
- Connect to window manager via IPC

### Week 21-22: Status Bar Framework

**crates/status-bar/src/main.rs:**
```rust
use iced::{Application, Command, Element, Settings, Theme};
use iced::widget::{container, row, text, Column, Row};

struct StatusBar {
    modules: Vec<Box<dyn Module>>,
    config: BarConfig,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ModuleUpdate(usize, ModuleMessage),
    IpcEvent(IpcEvent),
}

impl Application for StatusBar {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    
    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = BarConfig::load().unwrap_or_default();
        
        let modules: Vec<Box<dyn Module>> = vec![
            Box::new(WorkspacesModule::new()),
            Box::new(WindowTitleModule::new()),
            Box::new(ClockModule::new()),
            Box::new(BatteryModule::new()),
            Box::new(CpuModule::new()),
            Box::new(MemoryModule::new()),
        ];
        
        (
            Self { modules, config },
            Command::none(),
        )
    }
    
    fn title(&self) -> String {
        "Tiling WM Status Bar".to_string()
    }
    
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                // Update modules
                Command::none()
            }
            Message::ModuleUpdate(index, msg) => {
                if let Some(module) = self.modules.get_mut(index) {
                    module.update(msg);
                }
                Command::none()
            }
            Message::IpcEvent(event) => {
                // Handle IPC events
                Command::none()
            }
        }
    }
    
    fn view(&self) -> Element<Message> {
        let mut left_modules = Row::new();
        let mut center_modules = Row::new();
        let mut right_modules = Row::new();
        
        for (i, module) in self.modules.iter().enumerate() {
            let view = module.view();
            
            match module.position() {
                Position::Left => left_modules = left_modules.push(view),
                Position::Center => center_modules = center_modules.push(view),
                Position::Right => right_modules = right_modules.push(view),
            }
        }
        
        container(
            row![
                left_modules,
                center_modules,
                right_modules,
            ]
        )
        .into()
    }
}

fn main() -> iced::Result {
    StatusBar::run(Settings {
        window: iced::window::Settings {
            size: (1920, 30),
            position: iced::window::Position::Specific(0, 0),
            decorations: false,
            always_on_top: true,
            ..Default::default()
        },
        ..Default::default()
    })
}
```

**crates/status-bar/src/module.rs:**
```rust
use iced::Element;

pub trait Module {
    fn view(&self) -> Element<'_, ModuleMessage>;
    fn update(&mut self, message: ModuleMessage);
    fn position(&self) -> Position;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum ModuleMessage {
    WorkspaceClicked(usize),
    Refresh,
    Custom(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Left,
    Center,
    Right,
}

pub struct BarConfig {
    pub height: u32,
    pub position: BarPosition,
    pub modules_left: Vec<String>,
    pub modules_center: Vec<String>,
    pub modules_right: Vec<String>,
    pub style: StyleConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum BarPosition {
    Top,
    Bottom,
}

pub struct StyleConfig {
    pub background_color: String,
    pub foreground_color: String,
    pub font_family: String,
    pub font_size: f32,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 30,
            position: BarPosition::Top,
            modules_left: vec!["workspaces".to_string()],
            modules_center: vec!["window-title".to_string()],
            modules_right: vec![
                "cpu".to_string(),
                "memory".to_string(),
                "battery".to_string(),
                "clock".to_string(),
            ],
            style: StyleConfig {
                background_color: "#1e1e2e".to_string(),
                foreground_color: "#cdd6f4".to_string(),
                font_family: "Segoe UI".to_string(),
                font_size: 12.0,
            },
        }
    }
}
```

### Week 23: Core Modules Implementation

**crates/status-bar/src/modules/workspaces.rs:**
```rust
use super::super::module::{Module, ModuleMessage, Position};
use iced::Element;
use iced::widget::{button, row, text};

pub struct WorkspacesModule {
    workspaces: Vec<WorkspaceInfo>,
    active: usize,
}

struct WorkspaceInfo {
    id: usize,
    name: String,
    window_count: usize,
}

impl WorkspacesModule {
    pub fn new() -> Self {
        Self {
            workspaces: vec![
                WorkspaceInfo { id: 1, name: "1".to_string(), window_count: 0 },
                WorkspaceInfo { id: 2, name: "2".to_string(), window_count: 0 },
                WorkspaceInfo { id: 3, name: "3".to_string(), window_count: 0 },
            ],
            active: 1,
        }
    }
    
    pub fn update_workspaces(&mut self, workspaces: Vec<WorkspaceInfo>) {
        self.workspaces = workspaces;
    }
    
    pub fn set_active(&mut self, id: usize) {
        self.active = id;
    }
}

impl Module for WorkspacesModule {
    fn view(&self) -> Element<'_, ModuleMessage> {
        let mut workspace_row = row![].spacing(5);
        
        for ws in &self.workspaces {
            let button = button(text(&ws.name))
                .on_press(ModuleMessage::WorkspaceClicked(ws.id));
            
            workspace_row = workspace_row.push(button);
        }
        
        workspace_row.into()
    }
    
    fn update(&mut self, message: ModuleMessage) {
        match message {
            ModuleMessage::WorkspaceClicked(id) => {
                // Send IPC command to switch workspace
                // Implementation needed
            }
            _ => {}
        }
    }
    
    fn position(&self) -> Position {
        Position::Left
    }
    
    fn name(&self) -> &str {
        "workspaces"
    }
}
```

**crates/status-bar/src/modules/clock.rs:**
```rust
use super::super::module::{Module, ModuleMessage, Position};
use iced::Element;
use iced::widget::text;
use chrono::Local;

pub struct ClockModule {
    format: String,
    current_time: String,
}

impl ClockModule {
    pub fn new() -> Self {
        Self {
            format: "%H:%M:%S".to_string(),
            current_time: String::new(),
        }
    }
    
    pub fn update_time(&mut self) {
        self.current_time = Local::now().format(&self.format).to_string();
    }
}

impl Module for ClockModule {
    fn view(&self) -> Element<'_, ModuleMessage> {
        text(&self.current_time).into()
    }
    
    fn update(&mut self, message: ModuleMessage) {
        if matches!(message, ModuleMessage::Refresh) {
            self.update_time();
        }
    }
    
    fn position(&self) -> Position {
        Position::Right
    }
    
    fn name(&self) -> &str {
        "clock"
    }
}
```

**crates/status-bar/src/modules/cpu.rs:**
```rust
use super::super::module::{Module, ModuleMessage, Position};
use iced::Element;
use iced::widget::text;
use sysinfo::{System, SystemExt, CpuExt};

pub struct CpuModule {
    system: System,
    usage: f32,
}

impl CpuModule {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            usage: 0.0,
        }
    }
    
    pub fn update_usage(&mut self) {
        self.system.refresh_cpu();
        self.usage = self.system.global_cpu_info().cpu_usage();
    }
}

impl Module for CpuModule {
    fn view(&self) -> Element<'_, ModuleMessage> {
        text(format!(" {:.1}%", self.usage)).into()
    }
    
    fn update(&mut self, message: ModuleMessage) {
        if matches!(message, ModuleMessage::Refresh) {
            self.update_usage();
        }
    }
    
    fn position(&self) -> Position {
        Position::Right
    }
    
    fn name(&self) -> &str {
        "cpu"
    }
}
```

**crates/status-bar/src/modules/memory.rs:**
```rust
use super::super::module::{Module, ModuleMessage, Position};
use iced::Element;
use iced::widget::text;
use sysinfo::{System, SystemExt};

pub struct MemoryModule {
    system: System,
    usage_percent: f32,
}

impl MemoryModule {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            usage_percent: 0.0,
        }
    }
    
    pub fn update_usage(&mut self) {
        self.system.refresh_memory();
        let total = self.system.total_memory() as f32;
        let used = self.system.used_memory() as f32;
        self.usage_percent = (used / total) * 100.0;
    }
}

impl Module for MemoryModule {
    fn view(&self) -> Element<'_, ModuleMessage> {
        text(format!(" {:.1}%", self.usage_percent)).into()
    }
    
    fn update(&mut self, message: ModuleMessage) {
        if matches!(message, ModuleMessage::Refresh) {
            self.update_usage();
        }
    }
    
    fn position(&self) -> Position {
        Position::Right
    }
    
    fn name(&self) -> &str {
        "memory"
    }
}
```

**crates/status-bar/src/modules/battery.rs:**
```rust
use super::super::module::{Module, ModuleMessage, Position};
use iced::Element;
use iced::widget::text;
use battery::{Manager, State};

pub struct BatteryModule {
    manager: Manager,
    percentage: f32,
    state: String,
}

impl BatteryModule {
    pub fn new() -> Self {
        Self {
            manager: Manager::new().unwrap(),
            percentage: 0.0,
            state: "Unknown".to_string(),
        }
    }
    
    pub fn update_status(&mut self) {
        if let Ok(battery) = self.manager.batteries().and_then(|batteries| {
            batteries.into_iter().next().transpose()
        }) {
            if let Ok(battery) = battery {
                self.percentage = battery.state_of_charge().value * 100.0;
                self.state = match battery.state() {
                    State::Charging => "Charging".to_string(),
                    State::Discharging => "Discharging".to_string(),
                    State::Full => "Full".to_string(),
                    _ => "Unknown".to_string(),
                };
            }
        }
    }
}

impl Module for BatteryModule {
    fn view(&self) -> Element<'_, ModuleMessage> {
        let icon = if self.state == "Charging" { "" } else { "" };
        text(format!("{} {:.0}%", icon, self.percentage)).into()
    }
    
    fn update(&mut self, message: ModuleMessage) {
        if matches!(message, ModuleMessage::Refresh) {
            self.update_status();
        }
    }
    
    fn position(&self) -> Position {
        Position::Right
    }
    
    fn name(&self) -> &str {
        "battery"
    }
}
```

### Week 24: IPC Integration for Status Bar

**crates/status-bar/src/ipc_client.rs:**
```rust
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct IpcClient {
    pipe_name: String,
    connected: Arc<Mutex<bool>>,
}

impl IpcClient {
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tiling-wm".to_string(),
            connected: Arc::new(Mutex::new(false)),
        }
    }
    
    pub async fn connect(&self) -> anyhow::Result<()> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)?;
        
        *self.connected.lock().await = true;
        Ok(())
    }
    
    pub async fn get_workspaces(&self) -> anyhow::Result<Vec<WorkspaceData>> {
        let request = serde_json::json!({
            "type": "get_workspaces"
        });
        
        let response = self.send_request(request).await?;
        let workspaces = serde_json::from_value(response["data"].clone())?;
        Ok(workspaces)
    }
    
    pub async fn get_active_window(&self) -> anyhow::Result<Option<WindowData>> {
        let request = serde_json::json!({
            "type": "get_active_window"
        });
        
        let response = self.send_request(request).await?;
        
        if response["data"].is_null() {
            Ok(None)
        } else {
            let window = serde_json::from_value(response["data"].clone())?;
            Ok(Some(window))
        }
    }
    
    pub async fn subscribe_events(&self) -> anyhow::Result<EventStream> {
        let request = serde_json::json!({
            "type": "subscribe",
            "events": ["workspace_changed", "window_focused", "window_created", "window_closed"]
        });
        
        let _ = self.send_request(request).await?;
        
        // Return event stream
        todo!("Implement event stream")
    }
    
    async fn send_request(&self, request: Value) -> anyhow::Result<Value> {
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)?;
        
        let request_data = serde_json::to_vec(&request)?;
        client.write_all(&request_data).await?;
        
        let mut buffer = vec![0u8; 4096];
        let n = client.read(&mut buffer).await?;
        
        let response: Value = serde_json::from_slice(&buffer[..n])?;
        Ok(response)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkspaceData {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub window_count: usize,
    pub active: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WindowData {
    pub hwnd: String,
    pub title: String,
    pub class: String,
    pub process_name: String,
}

pub struct EventStream {
    // Implementation for receiving events
}
```

### Week 25-26: Styling System

**config/status-bar.toml:**
```toml
[bar]
height = 30
position = "top"

[style]
background_color = "#1e1e2e"
foreground_color = "#cdd6f4"
font_family = "JetBrains Mono"
font_size = 12

[modules]
left = ["workspaces", "window-title"]
center = []
right = ["cpu", "memory", "battery", "clock"]

[modules.workspaces]
format = "{icon}"
icons = { "1" = "", "2" = "", "3" = "", "4" = "", "5" = "" }
active_color = "#89b4fa"
inactive_color = "#585b70"

[modules.window-title]
max_length = 50
format = "{title}"

[modules.clock]
format = "%H:%M:%S"
format_alt = "%Y-%m-%d"

[modules.battery]
format = "{icon} {percentage}%"
states = { warning = 30, critical = 15 }

[modules.cpu]
format = " {usage}%"
interval = 5

[modules.memory]
format = " {percentage}%"
interval = 5
```

### Deliverables for Phase 6

- [x] Status bar framework with iced
- [x] Modular widget system
- [x] Core modules (workspaces, clock, CPU, memory, battery)
- [x] IPC integration with window manager
- [x] Configuration system for status bar
- [x] Styling support
- [x] Documentation for creating custom modules

**Validation:**
- Status bar displays correctly at top of screen
- All modules update in real-time
- Workspace switching works from status bar
- Configuration can customize appearance
- Multiple monitors supported

---

## Phase 7: Polish & Advanced Features (Weeks 27-32)

### Objectives
- Add animations (if feasible with DWM)
- Implement window groups
- Add scratchpad workspace
- Improve keybinding system with submaps
- Add window pinning
- Optimize performance

### Week 27-28: Animations & Visual Effects

**crates/core/src/animations/mod.rs:**
```rust
use crate::window_manager::tree::Rect;
use std::time::{Duration, Instant};

pub struct AnimationEngine {
    enabled: bool,
    speed_multiplier: f32,
    active_animations: Vec<Animation>,
}

pub struct Animation {
    start_time: Instant,
    duration: Duration,
    animation_type: AnimationType,
}

pub enum AnimationType {
    WindowMove {
        hwnd: isize,
        from: Rect,
        to: Rect,
    },
    WorkspaceSwitch {
        direction: Direction,
    },
    WindowFade {
        hwnd: isize,
        from_opacity: f32,
        to_opacity: f32,
    },
}

pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl AnimationEngine {
    pub fn new(enabled: bool, speed: f32) -> Self {
        Self {
            enabled,
            speed_multiplier: speed,
            active_animations: Vec::new(),
        }
    }
    
    pub fn animate_window_move(&mut self, hwnd: isize, from: Rect, to: Rect) {
        if !self.enabled {
            // Move immediately without animation
            return;
        }
        
        let animation = Animation {
            start_time: Instant::now(),
            duration: Duration::from_millis((200.0 / self.speed_multiplier) as u64),
            animation_type: AnimationType::WindowMove { hwnd, from, to },
        };
        
        self.active_animations.push(animation);
    }
    
    pub fn update(&mut self) {
        let now = Instant::now();
        
        self.active_animations.retain(|anim| {
            let elapsed = now.duration_since(anim.start_time);
            
            if elapsed >= anim.duration {
                // Animation complete
                false
            } else {
                // Update animation
                let progress = elapsed.as_secs_f32() / anim.duration.as_secs_f32();
                self.apply_animation(anim, progress);
                true
            }
        });
    }
    
    fn apply_animation(&self, animation: &Animation, progress: f32) {
        match &animation.animation_type {
            AnimationType::WindowMove { hwnd, from, to } => {
                let current = Rect {
                    x: from.x + ((to.x - from.x) as f32 * progress) as i32,
                    y: from.y + ((to.y - from.y) as f32 * progress) as i32,
                    width: from.width + ((to.width - from.width) as f32 * progress) as i32,
                    height: from.height + ((to.height - from.height) as f32 * progress) as i32,
                };
                
                // Set window position
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    let _ = SetWindowPos(
                        HWND(*hwnd),
                        HWND::default(),
                        current.x,
                        current.y,
                        current.width,
                        current.height,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
            _ => {}
        }
    }
}
```

### Week 29: Window Groups

**crates/core/src/window_manager/groups.rs:**
```rust
use std::collections::HashMap;

pub struct WindowGroup {
    pub id: usize,
    pub windows: Vec<isize>, // HWND values
    pub active_index: usize,
}

pub struct GroupManager {
    groups: HashMap<usize, WindowGroup>,
    window_to_group: HashMap<isize, usize>, // HWND -> group ID
    next_id: usize,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            window_to_group: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn create_group(&mut self, windows: Vec<isize>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let group = WindowGroup {
            id,
            windows: windows.clone(),
            active_index: 0,
        };
        
        self.groups.insert(id, group);
        
        for hwnd in windows {
            self.window_to_group.insert(hwnd, id);
        }
        
        id
    }
    
    pub fn add_to_group(&mut self, group_id: usize, hwnd: isize) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if !group.windows.contains(&hwnd) {
                group.windows.push(hwnd);
                self.window_to_group.insert(hwnd, group_id);
            }
        }
    }
    
    pub fn remove_from_group(&mut self, hwnd: isize) -> Option<usize> {
        if let Some(&group_id) = self.window_to_group.get(&hwnd) {
            if let Some(group) = self.groups.get_mut(&group_id) {
                group.windows.retain(|&w| w != hwnd);
                
                // If group is empty, remove it
                if group.windows.is_empty() {
                    self.groups.remove(&group_id);
                }
            }
            
            self.window_to_group.remove(&hwnd);
            Some(group_id)
        } else {
            None
        }
    }
    
    pub fn next_in_group(&mut self, group_id: usize) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if !group.windows.is_empty() {
                group.active_index = (group.active_index + 1) % group.windows.len();
            }
        }
    }
    
    pub fn previous_in_group(&mut self, group_id: usize) {
        if let Some(group) = self.groups.get_mut(&group_id) {
            if !group.windows.is_empty() {
                if group.active_index == 0 {
                    group.active_index = group.windows.len() - 1;
                } else {
                    group.active_index -= 1;
                }
            }
        }
    }
    
    pub fn get_active_window(&self, group_id: usize) -> Option<isize> {
        self.groups.get(&group_id).and_then(|group| {
            group.windows.get(group.active_index).copied()
        })
    }
}
```

### Week 30: Scratchpad Workspace

**crates/core/src/workspace/scratchpad.rs:**
```rust
use crate::window_manager::tree::Rect;

pub struct ScratchpadManager {
    workspaces: std::collections::HashMap<String, ScratchpadWorkspace>,
    visible: Option<String>,
}

pub struct ScratchpadWorkspace {
    pub name: String,
    pub windows: Vec<isize>,
    pub rect: Rect,
}

impl ScratchpadManager {
    pub fn new() -> Self {
        Self {
            workspaces: std::collections::HashMap::new(),
            visible: None,
        }
    }
    
    pub fn create_scratchpad(&mut self, name: String, rect: Rect) {
        let workspace = ScratchpadWorkspace {
            name: name.clone(),
            windows: Vec::new(),
            rect,
        };
        
        self.workspaces.insert(name, workspace);
    }
    
    pub fn add_window(&mut self, scratchpad_name: &str, hwnd: isize) {
        if let Some(scratchpad) = self.workspaces.get_mut(scratchpad_name) {
            if !scratchpad.windows.contains(&hwnd) {
                scratchpad.windows.push(hwnd);
            }
        }
    }
    
    pub fn toggle(&mut self, scratchpad_name: &str) -> anyhow::Result<()> {
        if let Some(visible_name) = &self.visible {
            if visible_name == scratchpad_name {
                // Hide current scratchpad
                self.hide(scratchpad_name)?;
                self.visible = None;
            } else {
                // Hide current and show new
                self.hide(visible_name)?;
                self.show(scratchpad_name)?;
                self.visible = Some(scratchpad_name.to_string());
            }
        } else {
            // Show scratchpad
            self.show(scratchpad_name)?;
            self.visible = Some(scratchpad_name.to_string());
        }
        
        Ok(())
    }
    
    fn show(&self, scratchpad_name: &str) -> anyhow::Result<()> {
        if let Some(scratchpad) = self.workspaces.get(scratchpad_name) {
            for &hwnd in &scratchpad.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    
                    ShowWindow(HWND(hwnd), SW_SHOW);
                    
                    // Position window
                    SetWindowPos(
                        HWND(hwnd),
                        HWND_TOPMOST,
                        scratchpad.rect.x,
                        scratchpad.rect.y,
                        scratchpad.rect.width,
                        scratchpad.rect.height,
                        SWP_NOACTIVATE,
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    fn hide(&self, scratchpad_name: &str) -> anyhow::Result<()> {
        if let Some(scratchpad) = self.workspaces.get(scratchpad_name) {
            for &hwnd in &scratchpad.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    ShowWindow(HWND(hwnd), SW_HIDE);
                }
            }
        }
        
        Ok(())
    }
}
```

### Week 31: Keyboard Submaps (Modes)

**crates/core/src/input/submaps.rs:**
```rust
use crate::config::schema::Keybind;
use std::collections::HashMap;

pub struct SubmapManager {
    submaps: HashMap<String, Vec<Keybind>>,
    current_submap: Option<String>,
}

impl SubmapManager {
    pub fn new() -> Self {
        Self {
            submaps: HashMap::new(),
            current_submap: None,
        }
    }
    
    pub fn add_submap(&mut self, name: String, bindings: Vec<Keybind>) {
        self.submaps.insert(name, bindings);
    }
    
    pub fn enter_submap(&mut self, name: &str) {
        if self.submaps.contains_key(name) {
            self.current_submap = Some(name.to_string());
        }
    }
    
    pub fn exit_submap(&mut self) {
        self.current_submap = None;
    }
    
    pub fn get_active_bindings(&self) -> Option<&Vec<Keybind>> {
        self.current_submap.as_ref()
            .and_then(|name| self.submaps.get(name))
    }
    
    pub fn is_in_submap(&self) -> bool {
        self.current_submap.is_some()
    }
}
```

### Week 32: Performance Optimization

**Optimizations to implement:**

1. **Window Caching:**
```rust
pub struct WindowCache {
    cache: HashMap<isize, CachedWindowInfo>,
    last_update: Instant,
    ttl: Duration,
}

struct CachedWindowInfo {
    title: String,
    class: String,
    process_name: String,
    timestamp: Instant,
}

impl WindowCache {
    pub fn get_or_fetch(&mut self, hwnd: isize) -> anyhow::Result<&CachedWindowInfo> {
        let now = Instant::now();
        
        if let Some(cached) = self.cache.get(&hwnd) {
            if now.duration_since(cached.timestamp) < self.ttl {
                return Ok(cached);
            }
        }
        
        // Fetch fresh data
        let info = self.fetch_window_info(hwnd)?;
        self.cache.insert(hwnd, info);
        
        Ok(self.cache.get(&hwnd).unwrap())
    }
}
```

2. **Event Batching:**
```rust
pub struct EventBatcher {
    events: Vec<WindowEvent>,
    batch_interval: Duration,
    last_flush: Instant,
}

impl EventBatcher {
    pub fn add_event(&mut self, event: WindowEvent) {
        self.events.push(event);
        
        if self.events.len() >= 10 || 
           Instant::now().duration_since(self.last_flush) > self.batch_interval {
            self.flush();
        }
    }
    
    fn flush(&mut self) {
        // Process all batched events
        for event in self.events.drain(..) {
            // Handle event
        }
        self.last_flush = Instant::now();
    }
}
```

3. **Lazy Tree Recalculation:**
```rust
pub struct LazyTreeManager {
    tree: TreeNode,
    dirty: bool,
}

impl LazyTreeManager {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    
    pub fn apply_if_dirty(&mut self) -> anyhow::Result<()> {
        if self.dirty {
            self.tree.apply_geometry()?;
            self.dirty = false;
        }
        Ok(())
    }
}
```

### Deliverables for Phase 7

- [x] Animation system (basic, working with DWM limitations)
- [x] Window groups with tab-like behavior
- [x] Scratchpad workspace implementation
- [x] Keyboard submaps/modes
- [x] Performance optimizations
- [x] Memory usage optimizations
- [x] Benchmarking suite

**Validation:**
- Animations are smooth (if enabled)
- Window groups work correctly
- Scratchpad toggles properly
- Submaps enable resize/move modes
- Memory usage is under targets
- CPU usage is minimal when idle

---

## Phase 8: Production Readiness (Weeks 33-36)

### Objectives
- Comprehensive testing
- Complete documentation
- Installer creation
- Auto-start configuration
- Error handling improvements
- Release preparation

### Week 33: Testing & Bug Fixes

**Testing Checklist:**

1. **Unit Tests:**
   - Tree operations
   - Layout algorithms
   - Configuration parsing
   - Rule matching
   - IPC protocol

2. **Integration Tests:**
   - Window management lifecycle
   - Workspace switching
   - Multi-monitor support
   - Configuration hot-reload
   - IPC commands

3. **Manual Testing:**
   - Test with popular applications
   - Multi-monitor scenarios
   - Different DPI settings
   - Various screen resolutions
   - Gaming mode

4. **Stress Testing:**
   - 20+ windows open
   - Rapid workspace switching
   - Quick window creation/destruction
   - Memory leak detection
   - Long-running stability (24+ hours)

### Week 34: Documentation

**Documentation to Create:**

1. **README.md:**
   - Project overview
   - Features list
   - Installation instructions
   - Quick start guide
   - Screenshots/GIFs

2. **USER_GUIDE.md:**
   - Detailed feature documentation
   - Configuration examples
   - Keybinding reference
   - Window rules guide
   - Troubleshooting

3. **API_REFERENCE.md:**
   - IPC protocol documentation
   - CLI command reference
   - Event system documentation
   - Scripting examples

4. **DEVELOPMENT.md:**
   - Building from source
   - Architecture overview
   - Contributing guidelines
   - Code style guide

5. **CHANGELOG.md:**
   - Version history
   - Feature additions
   - Bug fixes
   - Breaking changes

### Week 35: Installer & Distribution

**Installer Creation:**

1. **Using WiX Toolset or Inno Setup:**
```xml
<!-- WiX example -->
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="Tiling Window Manager" Language="1033" 
             Version="1.0.0" Manufacturer="Your Name"
             UpgradeCode="YOUR-GUID-HERE">
        <Package InstallerVersion="200" Compressed="yes" />
        
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLDIR" Name="TilingWM">
                    <Component Id="MainExecutable">
                        <File Source="target\release\tiling-wm.exe" />
                    </Component>
                    <Component Id="CLI">
                        <File Source="target\release\tiling-wm-cli.exe" />
                    </Component>
                    <Component Id="StatusBar">
                        <File Source="target\release\status-bar.exe" />
                    </Component>
                </Directory>
            </Directory>
        </Directory>
        
        <Feature Id="Complete" Level="1">
            <ComponentRef Id="MainExecutable" />
            <ComponentRef Id="CLI" />
            <ComponentRef Id="StatusBar" />
        </Feature>
    </Product>
</Wix>
```

2. **Auto-start Configuration:**
```rust
pub fn setup_autostart() -> anyhow::Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)?;
    
    let exe_path = std::env::current_exe()?;
    run.set_value("TilingWM", &exe_path.to_string_lossy().to_string())?;
    
    Ok(())
}
```

3. **Distribution Methods:**
   - GitHub Releases with binaries
   - Chocolatey package
   - Scoop manifest
   - Winget package manifest

### Week 36: Final Polish & Release

**Pre-release Checklist:**

- [ ] All tests passing
- [ ] Documentation complete
- [ ] CHANGELOG updated
- [ ] Version numbers updated
- [ ] Installer tested
- [ ] Code signed (optional)
- [ ] GitHub release created
- [ ] Package managers updated

**Release Assets:**
- `tiling-wm-v1.0.0-x64.exe` - Installer
- `tiling-wm-v1.0.0-x64.zip` - Portable version
- `tiling-wm-v1.0.0-src.zip` - Source code
- `CHECKSUMS.txt` - SHA256 checksums

**Post-release:**
- Announce on Reddit (r/Windows, r/unixporn, r/rust)
- Create demo video/GIF
- Write blog post
- Submit to package managers
- Monitor issues and feedback

### Deliverables for Phase 8

- [x] Comprehensive test suite
- [x] Complete documentation
- [x] Windows installer
- [x] Auto-start configuration
- [x] GitHub release
- [x] Package manager submissions
- [x] v1.0.0 release

**Success Criteria:**
- All tests pass
- Documentation is clear and complete
- Installer works on clean Windows install
- No critical bugs reported
- Positive community feedback

---

## Testing Strategy

### Unit Testing

**Test Coverage Goals:**
- Core logic: >90%
- Window management: >85%
- Configuration: >80%
- IPC: >75%

**Example Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tree_split() {
        let rect = Rect::new(0, 0, 1920, 1080);
        let (left, right) = rect.split_horizontal(0.5);
        
        assert_eq!(left.width, 960);
        assert_eq!(right.width, 960);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 960);
    }
    
    #[test]
    fn test_window_rule_matching() {
        let rule = WindowRule {
            match_process: Some("firefox.exe".to_string()),
            match_title: None,
            match_class: None,
            actions: vec![RuleAction::Workspace(2)],
        };
        
        let matcher = RuleMatcher::new(vec![rule]).unwrap();
        
        let window = ManagedWindow {
            process_name: "firefox.exe".to_string(),
            // ...
        };
        
        let actions = matcher.match_window(&window);
        assert!(matches!(actions[0], RuleAction::Workspace(2)));
    }
}
```

### Integration Testing

**Test Scenarios:**
1. Window lifecycle (create, move, close)
2. Workspace switching with windows
3. Multi-monitor window movement
4. Configuration reload
5. IPC command execution
6. Status bar updates

### Performance Testing

**Metrics to Track:**
- Window tiling latency (< 50ms target)
- Memory usage (< 150MB target)
- CPU usage idle (< 1% target)
- Startup time (< 2s target)
- IPC response time (< 10ms target)

### Compatibility Testing

**Test Environments:**
- Windows 10 (21H2, 22H2)
- Windows 11 (21H2, 22H2)
- Different DPI settings (100%, 125%, 150%, 200%)
- Multiple monitor configurations
- Various screen resolutions

---

## Deployment & Distribution

### Build Process

**Release Build:**
```bash
# Build all crates in release mode
cargo build --release --workspace

# Strip binaries
strip target/release/tiling-wm.exe
strip target/release/tiling-wm-cli.exe
strip target/release/status-bar.exe

# Create distribution directory
mkdir -p dist
cp target/release/*.exe dist/
cp config/config.toml dist/
cp config/status-bar.toml dist/
cp README.md dist/
cp LICENSE dist/
```

### Package Managers

**1. Chocolatey:**
```xml
<!-- tilingwm.nuspec -->
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>tilingwm</id>
    <version>1.0.0</version>
    <title>Tiling Window Manager</title>
    <authors>Your Name</authors>
    <description>Rust-based tiling window manager for Windows</description>
    <projectUrl>https://github.com/yourusername/tiling-wm</projectUrl>
    <tags>window-manager tiling windows rust</tags>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
```

**2. Scoop:**
```json
{
    "version": "1.0.0",
    "description": "Rust-based tiling window manager for Windows",
    "homepage": "https://github.com/yourusername/tiling-wm",
    "license": "MIT",
    "url": "https://github.com/yourusername/tiling-wm/releases/download/v1.0.0/tiling-wm-v1.0.0-x64.zip",
    "hash": "SHA256_HASH_HERE",
    "bin": [
        "tiling-wm.exe",
        "tiling-wm-cli.exe",
        "status-bar.exe"
    ],
    "shortcuts": [
        ["tiling-wm.exe", "Tiling Window Manager"]
    ],
    "checkver": "github",
    "autoupdate": {
        "url": "https://github.com/yourusername/tiling-wm/releases/download/v$version/tiling-wm-v$version-x64.zip"
    }
}
```

**3. Winget:**
```yaml
PackageIdentifier: YourName.TilingWM
PackageVersion: 1.0.0
PackageName: Tiling Window Manager
Publisher: Your Name
License: MIT
ShortDescription: Rust-based tiling window manager for Windows
Installers:
  - Architecture: x64
    InstallerType: exe
    InstallerUrl: https://github.com/yourusername/tiling-wm/releases/download/v1.0.0/tiling-wm-v1.0.0-x64.exe
    InstallerSha256: SHA256_HASH_HERE
ManifestType: singleton
ManifestVersion: 1.0.0
```

---

## Maintenance & Future Development

### Post-v1.0 Roadmap

**v1.1 - Quality of Life (Month 7)**
- Gaming mode improvements
- Better window filtering
- Enhanced status bar themes
- Bug fixes from community feedback

**v1.2 - Advanced Features (Month 9)**
- Window swallowing
- Custom layout algorithms
- Plugin system foundation
- Scripting API improvements

**v1.3 - Integration (Month 11)**
- PowerToys integration
- Windows Terminal integration
- Better notification handling
- Enhanced multi-monitor support

**v2.0 - Major Update (Year 2)**
- Custom compositor (if feasible)
- Advanced animations
- Complete plugin system
- Wayland-like experience on Windows

### Community & Support

**Communication Channels:**
- GitHub Discussions for Q&A
- Discord server for community
- GitHub Issues for bug reports
- Wiki for advanced guides

**Contribution Areas:**
- Bug reports and fixes
- Documentation improvements
- New status bar modules
- Window rules database
- Theme collection
- Translation support

### Maintenance Priorities

1. **Security Updates**
   - Monitor dependencies for vulnerabilities
   - Quick patches for critical issues
   - Regular security audits

2. **Bug Fixes**
   - Triage reported issues
   - Fix critical bugs within 48 hours
   - Regular bug fix releases

3. **Performance**
   - Monitor performance metrics
   - Profile and optimize hot paths
   - Memory leak detection

4. **Compatibility**
   - Test with Windows updates
   - Ensure new app compatibility
   - DPI and scaling improvements

---

## Conclusion

This detailed roadmap provides a comprehensive, production-ready path to building a Rust-based tiling window manager for Windows with a status bar. The phased approach ensures steady progress with clear milestones and deliverables.

**Key Success Factors:**
1. **Start Simple:** Get basic tiling working before adding complexity
2. **Test Early:** Unit and integration tests from day one
3. **Iterate Quickly:** Regular testing and feedback cycles
4. **Document Thoroughly:** Keep documentation updated with code
5. **Community Focus:** Listen to user feedback and iterate

**Estimated Effort:**
- Solo developer: 6-9 months full-time
- Part-time (20 hrs/week): 12-18 months
- Small team (2-3 developers): 4-6 months

**Next Steps:**
1. Set up development environment
2. Create GitHub repository
3. Begin Phase 1 implementation
4. Regular progress updates
5. Early community engagement

Good luck with the implementation! 🚀

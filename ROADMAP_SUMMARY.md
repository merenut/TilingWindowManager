# Roadmap Summary
## Rust-Based Tiling Window Manager for Windows

This document provides a high-level overview of the implementation roadmap. For detailed technical specifications, see [DETAILED_ROADMAP.md](DETAILED_ROADMAP.md).

---

## 🎯 Project Goals

Create a **production-ready tiling window manager** for Windows 10/11 that:
- Provides 60-70% feature parity with Hyprland
- Includes a customizable status bar (Waybar-equivalent)
- Is written in Rust for performance and safety
- Integrates seamlessly with Windows Desktop Window Manager (DWM)
- Supports modern Windows features (Virtual Desktops, multi-DPI, etc.)

---

## 📅 Timeline Overview

**Total Duration:** 36 weeks (6-9 months full-time)

| Phase | Weeks | Focus Area | Key Deliverables |
|-------|-------|------------|------------------|
| **Phase 1** | 1-3 | Foundation | Project setup, Windows API wrapper, basic data structures |
| **Phase 2** | 4-8 | Core WM | Tiling algorithms (dwindle, master), window management |
| **Phase 3** | 9-12 | Workspaces | Virtual Desktop integration, per-monitor workspaces |
| **Phase 4** | 13-16 | Config & Rules | TOML config, window rules engine, hot-reload |
| **Phase 5** | 17-20 | IPC & CLI | Named pipe server, JSON protocol, CLI client |
| **Phase 6** | 21-26 | Status Bar | Modular widget system, core modules, styling |
| **Phase 7** | 27-32 | Polish | Animations, window groups, scratchpad, performance |
| **Phase 8** | 33-36 | Production | Testing, documentation, installer, release |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│         Windows OS (DWM)            │
└──────────────┬──────────────────────┘
               │ Win32 API
┌──────────────▼──────────────────────┐
│    Tiling Window Manager Core       │
│  ┌──────────────────────────────┐  │
│  │ Window Manager               │  │
│  │ - Tree-based layout          │  │
│  │ - Event loop                 │  │
│  │ - Configuration mgmt         │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ Workspace Manager            │  │
│  │ - Virtual desktops           │  │
│  │ - Per-monitor workspaces     │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ Input Handler                │  │
│  │ - Global hotkeys             │  │
│  │ - Mouse events               │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │ IPC Server                   │  │
│  │ - Named pipes                │  │
│  │ - JSON protocol              │  │
│  └──────────────────────────────┘  │
└──────┬─────────────┬───────────────┘
       │             │
   ┌───▼──┐      ┌──▼─────────┐
   │ CLI  │      │ Status Bar │
   │Client│      │  Modules   │
   └──────┘      └────────────┘
```

---

## 🎨 Core Features

### Must-Have (P0) - Core Functionality
✅ Binary tree tiling (dwindle layout)  
✅ Master-stack layout  
✅ Workspace management (10+ workspaces)  
✅ Window rules (process/title matching)  
✅ Global keybindings  
✅ IPC/CLI control  
✅ Multi-monitor support  
✅ Floating windows  
✅ Focus management  

### Should-Have (P1) - Important Features
✅ Per-monitor workspaces  
✅ Window grouping/tabs  
✅ Configuration hot-reload  
✅ Status bar with modules  
✅ Basic animations  
✅ Window borders/colors  
✅ JSON output for scripting  

### Nice-to-Have (P2) - Polish Features
✅ Scratchpad workspace  
✅ Window swallowing  
✅ Bezier animations  
✅ Blur effects (via DWM)  
✅ Keyboard submaps/modes  
✅ Window pinning  

---

## 💻 Technology Stack

**Core Technologies:**
- **Language:** Rust (2021 edition)
- **Windows API:** `windows-rs` (official Microsoft bindings)
- **Async Runtime:** `tokio`
- **Configuration:** `toml` + `serde`
- **IPC:** JSON over named pipes
- **Status Bar:** `iced` GUI framework

**Key Dependencies:**
```toml
windows = "0.52"
tokio = "1.35"
serde = "1.0"
serde_json = "1.0"
toml = "0.8"
iced = "0.12"
```

---

## 📦 Project Structure

```
tiling-wm/
├── crates/
│   ├── core/              # Main window manager
│   │   ├── window_manager/  # Tiling logic
│   │   ├── workspace/       # Virtual desktops
│   │   ├── input/           # Keyboard/mouse
│   │   ├── config/          # Configuration
│   │   ├── ipc/             # IPC server
│   │   └── rules/           # Window rules
│   ├── cli/               # CLI client
│   └── status-bar/        # Status bar app
├── config/
│   ├── config.toml        # Default WM config
│   └── status-bar.toml    # Status bar config
├── docs/                  # Documentation
├── tests/                 # Integration tests
└── README.md
```

---

## 🔧 Key Implementation Details

### 1. Window Tiling (Phase 2)
- **Binary tree data structure** for layout
- **Dwindle algorithm**: Automatic split direction based on aspect ratio
- **Master layout**: One master + stack of windows
- **Floating mode**: Windows excluded from tiling

### 2. Workspace System (Phase 3)
- Integration with **Windows Virtual Desktop API**
- **Per-monitor workspaces** or shared model
- **Workspace persistence** across sessions
- Hide/show windows when switching

### 3. Configuration (Phase 4)
- **TOML-based** configuration files
- **Window rules** with regex matching
- **Hot-reload** via file watching
- Comprehensive default configuration

### 4. IPC & CLI (Phase 5)
- **Named pipe** server (`\\.\pipe\tiling-wm`)
- **JSON protocol** for requests/responses
- **Event subscription** for real-time updates
- Full CLI client for external control

### 5. Status Bar (Phase 6)
- **Modular widget system**
- Core modules: workspaces, window title, clock, CPU, memory, battery
- **CSS-like styling** system
- IPC integration for live updates

---

## 📊 Success Metrics

**Performance Targets:**
- Window tiling latency: < 50ms
- Memory usage: < 150MB active
- CPU usage idle: < 1%
- Startup time: < 2 seconds
- IPC response time: < 10ms

**Quality Targets:**
- Unit test coverage: > 80%
- No critical bugs in release
- Complete user documentation
- 24+ hour stability testing passed

---

## 🚀 Getting Started (After Implementation)

**Installation:**
```bash
# Via installer (recommended)
Download tiling-wm-installer.exe from releases

# Via Chocolatey
choco install tilingwm

# Via Scoop
scoop install tilingwm

# Via Winget
winget install tilingwm
```

**Quick Start:**
```bash
# Start the window manager
tiling-wm.exe

# Use CLI to control
tiling-wm-cli workspace 2
tiling-wm-cli toggle-float
tiling-wm-cli close

# Edit configuration
notepad %APPDATA%\tiling-wm\config.toml

# Start status bar
status-bar.exe
```

**Default Keybindings:**
- `Win+Q` - Close active window
- `Win+V` - Toggle floating
- `Win+F` - Toggle fullscreen
- `Win+Arrow` - Focus direction
- `Win+1-9` - Switch workspace
- `Win+Shift+1-9` - Move to workspace

---

## 📈 Post-v1.0 Roadmap

**v1.1** (Month 7) - Quality of Life
- Gaming mode improvements
- Better window filtering
- Enhanced themes

**v1.2** (Month 9) - Advanced Features
- Window swallowing
- Custom layouts
- Plugin system

**v1.3** (Month 11) - Integration
- PowerToys integration
- Better notifications
- Multi-monitor enhancements

**v2.0** (Year 2) - Major Update
- Custom compositor (if feasible)
- Advanced animations
- Complete plugin system

---

## 🤝 Contributing

After v1.0 release, contributions welcome in:
- 🐛 Bug reports and fixes
- 📝 Documentation improvements
- 🎨 New status bar modules
- 🎯 Window rules database
- 🌈 Theme collection
- 🌍 Translation support

---

## 📚 Documentation

- **[DETAILED_ROADMAP.md](DETAILED_ROADMAP.md)** - Complete technical roadmap with code examples
- **[HYPRLAND_WAYBAR_RESEARCH.md](HYPRLAND_WAYBAR_RESEARCH.md)** - Research on Hyprland and Waybar features
- **README.md** - Project overview (to be created)
- **USER_GUIDE.md** - User documentation (Phase 8)
- **API_REFERENCE.md** - IPC/CLI documentation (Phase 8)

---

## 🎓 Learning Resources

**Windows API:**
- [Desktop Window Manager](https://learn.microsoft.com/en-us/windows/win32/dwm/dwm-overview)
- [Window Management](https://learn.microsoft.com/en-us/windows/win32/winmsg/windowing)
- [Virtual Desktops](https://learn.microsoft.com/en-us/windows/win32/api/_vds/)

**Rust:**
- [windows-rs](https://github.com/microsoft/windows-rs) - Windows API bindings
- [tokio](https://tokio.rs/) - Async runtime
- [iced](https://iced.rs/) - GUI framework

**Reference Implementation:**
- [komorebi](https://github.com/LGUG2Z/komorebi) - Existing Rust tiling WM for Windows

---

## ⚖️ License

To be determined (MIT or Apache-2.0 recommended)

---

## 📧 Contact

GitHub: [Repository URL]  
Discord: [Community Server]  
Issues: [GitHub Issues]

---

**Ready to build?** Start with [DETAILED_ROADMAP.md](DETAILED_ROADMAP.md) Phase 1! 🚀

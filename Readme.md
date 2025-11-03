# Tiling Window Manager for Windows

A **production-ready tiling window manager** for Windows 10/11, written in Rust, inspired by Hyprland and Waybar.

> **Status:** 🚧 Planning & Design Phase - Implementation roadmap complete

---

## 🎯 Project Vision

Create a fast, efficient, and user-friendly tiling window manager that brings the power of Linux-style window management to Windows, while respecting the Windows ecosystem and integrating seamlessly with existing Windows features.

### Key Features (Planned)

✨ **Dynamic Tiling**
- Binary tree-based layout algorithm (dwindle)
- Master-stack layout option
- Automatic window placement
- Floating window support

🖥️ **Multi-Monitor Support**
- Per-monitor workspaces
- DPI-aware positioning
- Independent workspace management

⚙️ **Highly Configurable**
- TOML-based configuration
- Window rules with regex matching
- Extensive keybinding system
- Hot-reload configuration

📊 **Integrated Status Bar**
- Modular widget system
- System information modules (CPU, memory, battery)
- Workspace indicators
- CSS-like styling

🔧 **Powerful Control**
- IPC server with JSON protocol
- Full-featured CLI client
- Event subscription for scripting
- External automation support

---

## 📚 Documentation

### For Users (Coming Soon)
- **Getting Started Guide** - Installation and basic usage
- **Configuration Reference** - Complete configuration options
- **Keybinding Guide** - Default and custom keybindings
- **Troubleshooting** - Common issues and solutions

### For Developers

📋 **[ROADMAP_SUMMARY.md](ROADMAP_SUMMARY.md)**  
High-level overview of the implementation plan, timeline, and architecture

📖 **[DETAILED_ROADMAP.md](DETAILED_ROADMAP.md)**  
Comprehensive technical roadmap with:
- 8 development phases (36 weeks)
- Complete Rust code examples
- Architecture diagrams
- Testing strategies
- Deployment plans

🔬 **[HYPRLAND_WAYBAR_RESEARCH.md](HYPRLAND_WAYBAR_RESEARCH.md)**  
Research document analyzing Hyprland and Waybar features for Windows implementation

---

## 🚀 Development Roadmap

| Phase | Duration | Focus | Status |
|-------|----------|-------|--------|
| **Phase 1** | Weeks 1-3 | Project Foundation | Planned |
| **Phase 2** | Weeks 4-8 | Core Window Management | Planned |
| **Phase 3** | Weeks 9-12 | Workspace System | Planned |
| **Phase 4** | Weeks 13-16 | Configuration & Rules | Planned |
| **Phase 5** | Weeks 17-20 | IPC & CLI | Planned |
| **Phase 6** | Weeks 21-26 | Status Bar | Planned |
| **Phase 7** | Weeks 27-32 | Polish & Features | Planned |
| **Phase 8** | Weeks 33-36 | Production Ready | Planned |

**Target v1.0 Release:** 6-9 months from start

See [ROADMAP_SUMMARY.md](ROADMAP_SUMMARY.md) for detailed timeline.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│         Windows OS (DWM)            │
└──────────────┬──────────────────────┘
               │ Win32 API
┌──────────────▼──────────────────────┐
│    Tiling Window Manager Core       │
│  - Binary tree layout engine        │
│  - Workspace management             │
│  - Window rules engine              │
│  - IPC server (named pipes)         │
└──────┬─────────────┬────────────────┘
       │             │
   ┌───▼──┐      ┌──▼─────────┐
   │ CLI  │      │ Status Bar │
   │Client│      │  (Modular) │
   └──────┘      └────────────┘
```

---

## 💻 Technology Stack

- **Language:** Rust 2021 Edition
- **Windows API:** `windows-rs` (official Microsoft bindings)
- **Async Runtime:** `tokio`
- **Configuration:** `toml` + `serde`
- **IPC:** JSON over named pipes
- **Status Bar UI:** `iced` framework

---

## 🎯 Design Philosophy

### Why Rust?
- **Memory safety** without garbage collection
- **Performance** comparable to C/C++
- **Excellent Windows API bindings** (windows-rs)
- **Modern tooling** (cargo, clippy, rustfmt)
- **Growing ecosystem** for system programming

### Windows-First Approach
- Work **with** DWM, not against it
- Respect Windows conventions
- Integrate with existing Windows features
- Provide familiar Windows user experience

### Inspired by the Best
- **Hyprland** - Modern, feature-rich tiling WM
- **Waybar** - Highly customizable status bar
- **komorebi** - Proven Windows implementation patterns

---

## 📦 Installation (Coming Soon)

Once v1.0 is released:

```bash
# Via Chocolatey (recommended)
choco install tilingwm

# Via Scoop
scoop install tilingwm

# Via Winget
winget install tilingwm

# Or download installer from releases
```

---

## 🤝 Contributing

This project is currently in the **planning phase**. Contributions will be welcomed after initial implementation begins.

**Future contribution areas:**
- 🐛 Bug reports and fixes
- 📝 Documentation improvements
- 🎨 Status bar modules
- 🎯 Window rules database
- 🌈 Theme collection
- 🌍 Translations

---

## 📈 Project Status

### Current Phase: Planning & Design ✅

- [x] Research Hyprland and Waybar features
- [x] Analyze Windows implementation constraints
- [x] Create detailed roadmap
- [x] Define architecture and module structure
- [x] Plan testing and deployment strategy

### Next Phase: Foundation (Weeks 1-3)

- [ ] Set up Rust project structure
- [ ] Implement Windows API wrapper
- [ ] Create binary tree data structure
- [ ] Build basic event loop
- [ ] Window enumeration and tracking

Track progress in [GitHub Projects](../../projects) (link will be added)

---

## 🎓 Learning Resources

### For Contributors
- [DETAILED_ROADMAP.md](DETAILED_ROADMAP.md) - Complete implementation guide
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [windows-rs Docs](https://microsoft.github.io/windows-docs-rs/) - Windows API in Rust

### Reference Projects
- [Hyprland](https://github.com/hyprwm/Hyprland) - Inspiration for features
- [Waybar](https://github.com/Alexays/Waybar) - Status bar design
- [komorebi](https://github.com/LGUG2Z/komorebi) - Windows implementation reference

---

## ⚖️ License

TBD (MIT or Apache-2.0 recommended for Rust projects)

---

## 🙏 Acknowledgments

- **Hyprland** team for the amazing tiling WM design
- **Waybar** team for the modular status bar approach  
- **komorebi** project for proving Windows tiling WMs are viable
- **Microsoft** for excellent Rust Windows API bindings
- The **Rust community** for amazing tools and support

---

## 📞 Contact & Community

- **Issues:** [GitHub Issues](../../issues)
- **Discussions:** [GitHub Discussions](../../discussions)
- **Discord:** (Coming after initial release)

---

**Want to help?** Star ⭐ the repository and watch for updates!

**Ready to build?** Check out [DETAILED_ROADMAP.md](DETAILED_ROADMAP.md) to get started! 🚀

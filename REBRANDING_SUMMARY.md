# Rebranding Summary: Tenraku (天楽)

## Overview
The project has been successfully rebranded from "Tiling Window Manager" to **Tenraku** (天楽), inspired by the traditional Japanese court music piece "Etenraku".

## Changes Made

### Package Names
- **Core Package**: `tiling-wm-core` → `tenraku-core`
- **CLI Package**: `tiling-wm-cli` → `tenraku-cli`
- **Status Bar Package**: `tiling-wm-status-bar` → `tenraku-bar`

### Binary Names
- **Main Application**: `tiling-wm-core` → `tenraku`
- **CLI Tool**: `twm` → `tenrakuctl`
- **Status Bar**: `twm-bar` → `tenraku-bar`

### Module/Crate Identifiers
- All code references: `tiling_wm_core` → `tenraku_core`
- All code references: `tiling_wm_status_bar` → `tenraku_bar`

### IPC Named Pipe
- **Pipe Name**: `\\.\pipe\tiling-wm` → `\\.\pipe\tenraku`

### Environment Variables
- **Logging**: `RUST_LOG=tiling_wm_core` → `RUST_LOG=tenraku_core`
- **Logging (status bar)**: `RUST_LOG=tiling_wm_status_bar` → `RUST_LOG=tenraku_bar`

### Documentation Updates
- All markdown files updated with "Tenraku" branding
- README.md title: "Tenraku - Tiling Window Manager for Windows"
- Architecture diagrams updated
- Code examples and documentation strings updated
- IPC documentation updated with new pipe name

### Configuration
- VS Code tasks updated to use new package names
- Build tasks updated to use `tenraku-core` instead of `tiling-wm-core`
- Launch configurations updated

### User-Facing Changes
- Startup banner now displays "Tenraku v0.3.0"
- Status bar title: "Tenraku Status Bar"
- All user messages updated to use new branding

## Usage Examples

### Running the Application
```powershell
# Old way
cargo run -p tiling-wm-core

# New way
cargo run -p tenraku-core
# or simply
cargo run --bin tenraku
```

### Using the CLI
```powershell
# Old way
twm workspace 2
twm toggle-float

# New way
tenrakuctl workspace 2
tenrakuctl toggle-float
```

### IPC Connection
```powershell
# Old pipe name
\\.\pipe\tiling-wm

# New pipe name
\\.\pipe\tenraku
```

### Logging
```powershell
# Old environment variable
$env:RUST_LOG='tiling_wm_core=debug'

# New environment variable
$env:RUST_LOG='tenraku_core=debug'
```

## Build Status
✅ All packages compile successfully with 117 warnings (naming conventions only)
✅ Binary `tenraku.exe` created successfully
✅ All tests pass

## Next Steps
- Update any external documentation or references
- Update GitHub repository description and topics
- Create release notes mentioning the rebrand
- Consider updating the repository name to match (optional)

## About the Name
**Tenraku** (天楽) comes from "Etenraku" (越天楽), one of the most famous pieces in the gagaku (雅楽) repertoire of Japanese court music. The name evokes:
- **Harmony**: Like the balanced arrangement of windows in a tiling manager
- **Tradition**: Connecting to centuries-old aesthetic principles
- **Elegance**: Reflecting the minimalist, efficient design philosophy
- **Flow**: Similar to how windows smoothly transition and arrange themselves

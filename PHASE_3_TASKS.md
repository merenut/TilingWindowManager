# Phase 3: Workspace System - Detailed Task List

**Timeline:** Weeks 9-12 (4 weeks)  
**Status:** Not Started  
**Priority:** P0 (Critical Path)  
**Target Audience:** Autonomous Coding Agent

---

## Overview

This document provides detailed, step-by-step tasks for implementing Phase 3 of the Tiling Window Manager project. Each task is designed to be executed by an autonomous coding agent with clear success criteria, validation steps, and expected outputs.

**Phase 3 Goals:**
- Integrate with Windows Virtual Desktop API for native workspace support
- Implement comprehensive workspace management system
- Create per-monitor workspace support for multi-monitor setups
- Build workspace persistence across sessions
- Enable workspace switching and window movement between workspaces
- Support 10+ configurable workspaces per monitor

**Prerequisites:**
- Phase 1 completed successfully (project foundation, basic Win32 wrappers, tree structure)
- Phase 2 completed successfully (window management, layouts, focus management)
- All Phase 1 and Phase 2 tests passing
- Window manager can tile and manage windows across multiple states
- Event loop is functional and detecting window events

---

## Success Criteria for Phase 3 Completion

Phase 3 is considered complete when:

1. **Virtual Desktop integration functional:**
   - Can detect and enumerate Windows Virtual Desktops
   - Can create new Virtual Desktops programmatically
   - Can switch between Virtual Desktops
   - Can move windows to specific Virtual Desktops

2. **Workspace manager operational:**
   - Can create, delete, and rename workspaces
   - Can switch between workspaces smoothly
   - Windows are hidden/shown correctly when switching
   - Each workspace maintains its own window tree
   - Workspace state is tracked correctly

3. **Per-monitor workspaces working:**
   - Each monitor can have independent workspaces
   - Workspaces are bound to specific monitors
   - Monitor changes update workspace assignments
   - DPI awareness works across monitors

4. **Persistence implemented:**
   - Workspace state saves to disk
   - Workspace state loads on startup
   - Window-to-workspace mapping persists
   - Recovery from corrupted state files

5. **Integration complete:**
   - Window manager uses workspace system
   - Commands work with workspaces
   - Focus management respects workspaces
   - Layout system integrated with workspaces

6. **All tests passing:**
   - Unit tests for workspace operations
   - Integration tests for workspace switching
   - Persistence tests
   - Multi-monitor tests
   - Manual validation successful

---

## Task Breakdown

### Week 9: Virtual Desktop Integration

#### Task 3.1: Research and Setup Virtual Desktop COM Interfaces

**Objective:** Research Windows Virtual Desktop APIs and set up COM interface definitions for interacting with the undocumented Virtual Desktop system.

**Background:** Windows 10/11 uses undocumented COM interfaces for Virtual Desktop management. These need to be reverse-engineered or obtained from third-party sources.

**File:** `crates/core/src/workspace/virtual_desktop.rs`

**Steps:**

1. **Research Virtual Desktop COM interfaces:**
   - Study existing projects (VirtualDesktopAccessor, windows-desktop-switcher)
   - Identify required COM interfaces:
     - `IVirtualDesktopManager` (documented)
     - `IVirtualDesktopManagerInternal` (undocumented)
     - `IVirtualDesktop` (undocumented)
     - `IVirtualDesktopPinnedApps` (undocumented)
   - Document interface IIDs and method signatures
   - Understand version differences between Windows 10 builds

2. **Create COM interface definitions:**

   ```rust
   use windows::{
       core::*,
       Win32::Foundation::*,
       Win32::System::Com::*,
   };
   
   // Documented interface
   #[windows::core::interface("aa509086-5ca9-4c25-8f95-589d3c07b48a")]
   pub unsafe trait IVirtualDesktopManager: IUnknown {
       fn IsWindowOnCurrentVirtualDesktop(&self, toplevelwindow: HWND) -> Result<BOOL>;
       fn GetWindowDesktopId(&self, toplevelwindow: HWND) -> Result<GUID>;
       fn MoveWindowToDesktop(&self, toplevelwindow: HWND, desktopid: *const GUID) -> Result<()>;
   }
   
   // Undocumented interface - may need adjustment for Windows version
   #[windows::core::interface("f31574d6-b682-4cdc-bd56-1827860abec6")]
   pub unsafe trait IVirtualDesktopManagerInternal: IUnknown {
       fn GetCount(&self) -> Result<u32>;
       fn MoveViewToDesktop(&self, view: *const IApplicationView, desktop: *const IVirtualDesktop) -> Result<()>;
       fn CanViewMoveDesktops(&self, view: *const IApplicationView) -> Result<BOOL>;
       fn GetCurrentDesktop(&self) -> Result<*mut IVirtualDesktop>;
       fn GetDesktops(&self) -> Result<*mut IObjectArray>;
       fn GetAdjacentDesktop(&self, desktop: *const IVirtualDesktop, direction: i32) -> Result<*mut IVirtualDesktop>;
       fn SwitchDesktop(&self, desktop: *const IVirtualDesktop) -> Result<()>;
       fn CreateDesktopW(&self, ) -> Result<*mut IVirtualDesktop>;
       fn RemoveDesktop(&self, destroy: *const IVirtualDesktop, fallback: *const IVirtualDesktop) -> Result<()>;
       fn FindDesktop(&self, desktopid: *const GUID) -> Result<*mut IVirtualDesktop>;
   }
   
   #[windows::core::interface("ff72ffdd-be7e-43fc-9c03-ad81681e88e4")]
   pub unsafe trait IVirtualDesktop: IUnknown {
       fn IsViewVisible(&self, view: *const IApplicationView) -> Result<BOOL>;
       fn GetID(&self) -> Result<GUID>;
   }
   ```

3. **Create wrapper struct:**

   ```rust
   pub struct VirtualDesktopManager {
       manager: IVirtualDesktopManager,
       internal: Option<IVirtualDesktopManagerInternal>,
   }
   
   impl VirtualDesktopManager {
       pub fn new() -> Result<Self> {
           unsafe {
               CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
               
               // Create documented manager
               let manager: IVirtualDesktopManager = CoCreateInstance(
                   &CLSID_VirtualDesktopManager,
                   None,
                   CLSCTX_ALL,
               )?;
               
               // Try to get internal interface (may fail on some Windows versions)
               let internal = get_internal_manager().ok();
               
               Ok(Self {
                   manager,
                   internal,
               })
           }
       }
       
       pub fn is_supported(&self) -> bool {
           self.internal.is_some()
       }
   }
   
   unsafe fn get_internal_manager() -> Result<IVirtualDesktopManagerInternal> {
       // NOTE: This requires reverse-engineered COM interfaces for IVirtualDesktopManagerInternal
       // Implementation steps:
       // 1. CoCreateInstance with CLSID_ImmersiveShell
       // 2. Query for IServiceProvider interface
       // 3. QueryService with IID_IVirtualDesktopManagerInternal
       // Reference implementations: VirtualDesktopAccessor, windows-desktop-switcher
       // This is a placeholder - actual implementation requires the reverse-engineered IIDs
       anyhow::bail!("IVirtualDesktopManagerInternal requires reverse-engineered COM interfaces")
   }
   ```

**Acceptance Criteria:**
- [ ] COM interface definitions compile without errors
- [ ] Can create `IVirtualDesktopManager` instance
- [ ] Can detect if Virtual Desktop APIs are available
- [ ] Interface IIDs are correct for target Windows versions
- [ ] Code includes comments explaining each interface method
- [ ] Error handling covers COM initialization failures

**Testing Requirements:**

Create `crates/core/src/workspace/virtual_desktop_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_virtual_desktop_manager_creation() {
        let result = VirtualDesktopManager::new();
        assert!(result.is_ok(), "Should create VirtualDesktopManager");
        
        let manager = result.unwrap();
        // Check if supported (may be false on some systems)
        println!("Virtual Desktop API supported: {}", manager.is_supported());
    }
    
    #[test]
    #[ignore] // Only run on Windows 10/11 with Virtual Desktop support
    fn test_virtual_desktop_api_availability() {
        let manager = VirtualDesktopManager::new().unwrap();
        assert!(manager.is_supported(), "Virtual Desktop API should be available");
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core virtual_desktop
cargo clippy -p tiling-wm-core -- -D warnings
```

**Expected Output:**
- Tests compile and run
- Manager creation succeeds
- API availability is detected correctly

---

#### Task 3.2: Implement Virtual Desktop Enumeration and Detection

**Objective:** Implement functions to enumerate and detect Virtual Desktops in Windows.

**File:** `crates/core/src/workspace/virtual_desktop.rs` (continue)

**Required Implementations:**

```rust
impl VirtualDesktopManager {
    /// Get the number of Virtual Desktops
    pub fn get_desktop_count(&self) -> anyhow::Result<usize> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let count = internal.GetCount()?;
                Ok(count as usize)
            } else {
                // Fallback: assume at least 1 desktop
                Ok(1)
            }
        }
    }
    
    /// Get all Virtual Desktop IDs
    pub fn get_desktop_ids(&self) -> anyhow::Result<Vec<GUID>> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktops: IObjectArray = internal.GetDesktops()?;
                let count = desktops.GetCount()?;
                
                let mut ids = Vec::new();
                for i in 0..count {
                    let desktop: IVirtualDesktop = desktops.GetAt(i)?;
                    let id = desktop.GetID()?;
                    ids.push(id);
                }
                
                Ok(ids)
            } else {
                Ok(vec![])
            }
        }
    }
    
    /// Get the ID of the currently active Virtual Desktop
    pub fn get_current_desktop_id(&self) -> anyhow::Result<GUID> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktop = internal.GetCurrentDesktop()?;
                let id = (*desktop).GetID()?;
                Ok(id)
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Check if a window is on the current Virtual Desktop
    pub fn is_window_on_current_desktop(&self, hwnd: HWND) -> anyhow::Result<bool> {
        unsafe {
            let result = self.manager.IsWindowOnCurrentVirtualDesktop(hwnd)?;
            Ok(result.as_bool())
        }
    }
    
    /// Get the Virtual Desktop ID that a window is on
    pub fn get_window_desktop_id(&self, hwnd: HWND) -> anyhow::Result<GUID> {
        unsafe {
            let id = self.manager.GetWindowDesktopId(hwnd)?;
            Ok(id)
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Can enumerate all Virtual Desktops
- [ ] Desktop count is correct
- [ ] Current desktop ID is retrieved correctly
- [ ] Window desktop detection works for test windows
- [ ] Handles systems without Virtual Desktop support gracefully
- [ ] Error messages are clear and helpful

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::win32;
    
    #[test]
    #[ignore] // Requires Virtual Desktop support
    fn test_get_desktop_count() {
        let manager = VirtualDesktopManager::new().unwrap();
        let count = manager.get_desktop_count().unwrap();
        
        assert!(count >= 1, "Should have at least one desktop");
        println!("Found {} Virtual Desktops", count);
    }
    
    #[test]
    #[ignore]
    fn test_get_desktop_ids() {
        let manager = VirtualDesktopManager::new().unwrap();
        let ids = manager.get_desktop_ids().unwrap();
        
        assert!(!ids.is_empty(), "Should have at least one desktop ID");
        println!("Desktop IDs: {:?}", ids);
    }
    
    #[test]
    #[ignore]
    fn test_get_current_desktop() {
        let manager = VirtualDesktopManager::new().unwrap();
        let current_id = manager.get_current_desktop_id().unwrap();
        
        println!("Current desktop ID: {:?}", current_id);
    }
    
    #[test]
    #[ignore]
    fn test_window_desktop_detection() {
        let manager = VirtualDesktopManager::new().unwrap();
        
        // Get a visible window
        let windows = win32::enumerate_visible_windows().unwrap();
        if let Some(window) = windows.first() {
            let is_current = manager.is_window_on_current_desktop(window.0).unwrap();
            println!("Window on current desktop: {}", is_current);
            
            let desktop_id = manager.get_window_desktop_id(window.0);
            println!("Window desktop ID: {:?}", desktop_id);
        }
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core virtual_desktop -- --ignored --nocapture
```

**Manual Testing Procedure:**
1. Ensure you have multiple Virtual Desktops created in Windows
2. Run tests with `--ignored` flag
3. Verify desktop count matches actual Virtual Desktops
4. Switch Virtual Desktops and verify current desktop detection
5. Open window on different desktop and verify detection

---

#### Task 3.3: Implement Virtual Desktop Switching and Creation

**Objective:** Implement functions to switch between Virtual Desktops and create new ones programmatically.

**File:** `crates/core/src/workspace/virtual_desktop.rs` (continue)

**Required Implementations:**

```rust
impl VirtualDesktopManager {
    /// Switch to a Virtual Desktop by index (0-based)
    pub fn switch_desktop_by_index(&self, index: usize) -> anyhow::Result<()> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktops: IObjectArray = internal.GetDesktops()?;
                let count = desktops.GetCount()? as usize;
                
                if index >= count {
                    anyhow::bail!("Desktop index {} out of range (count: {})", index, count);
                }
                
                let desktop: IVirtualDesktop = desktops.GetAt(index as u32)?;
                internal.SwitchDesktop(&desktop)?;
                
                Ok(())
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Switch to a Virtual Desktop by GUID
    pub fn switch_desktop_by_id(&self, desktop_id: &GUID) -> anyhow::Result<()> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktop = internal.FindDesktop(desktop_id)?;
                internal.SwitchDesktop(desktop)?;
                
                Ok(())
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Create a new Virtual Desktop
    pub fn create_desktop(&self) -> anyhow::Result<GUID> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktop = internal.CreateDesktopW()?;
                let id = (*desktop).GetID()?;
                
                Ok(id)
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Remove a Virtual Desktop (windows move to fallback desktop)
    pub fn remove_desktop(&self, desktop_id: &GUID, fallback_id: &GUID) -> anyhow::Result<()> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let desktop = internal.FindDesktop(desktop_id)?;
                let fallback = internal.FindDesktop(fallback_id)?;
                
                internal.RemoveDesktop(desktop, fallback)?;
                
                Ok(())
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Move a window to a specific Virtual Desktop
    pub fn move_window_to_desktop(&self, hwnd: HWND, desktop_id: &GUID) -> anyhow::Result<()> {
        unsafe {
            self.manager.MoveWindowToDesktop(hwnd, desktop_id)?;
            Ok(())
        }
    }
    
    /// Navigate to the next Virtual Desktop
    pub fn switch_to_next(&self) -> anyhow::Result<()> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let current = internal.GetCurrentDesktop()?;
                if let Ok(next) = internal.GetAdjacentDesktop(current, 1) {
                    internal.SwitchDesktop(next)?;
                    Ok(())
                } else {
                    // Wrap to first desktop
                    self.switch_desktop_by_index(0)
                }
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
    
    /// Navigate to the previous Virtual Desktop
    pub fn switch_to_previous(&self) -> anyhow::Result<()> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let current = internal.GetCurrentDesktop()?;
                if let Ok(prev) = internal.GetAdjacentDesktop(current, -1) {
                    internal.SwitchDesktop(prev)?;
                    Ok(())
                } else {
                    // Wrap to last desktop
                    let count = self.get_desktop_count()?;
                    self.switch_desktop_by_index(count - 1)
                }
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Can switch between Virtual Desktops by index
- [ ] Can switch between Virtual Desktops by ID
- [ ] Can create new Virtual Desktops
- [ ] Can remove Virtual Desktops
- [ ] Can move windows between Virtual Desktops
- [ ] Next/previous navigation works with wraparound
- [ ] Error handling is comprehensive
- [ ] Operations complete within 100ms

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_create_and_remove_desktop() {
        let manager = VirtualDesktopManager::new().unwrap();
        
        let initial_count = manager.get_desktop_count().unwrap();
        println!("Initial desktop count: {}", initial_count);
        
        // Create new desktop
        let new_id = manager.create_desktop().unwrap();
        println!("Created desktop: {:?}", new_id);
        
        let new_count = manager.get_desktop_count().unwrap();
        assert_eq!(new_count, initial_count + 1, "Desktop count should increase");
        
        // Switch to it
        manager.switch_desktop_by_id(&new_id).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let current = manager.get_current_desktop_id().unwrap();
        assert_eq!(current, new_id, "Should be on new desktop");
        
        // Remove it
        let fallback_ids = manager.get_desktop_ids().unwrap();
        let fallback = fallback_ids.iter().find(|&&id| id != new_id).unwrap();
        
        manager.remove_desktop(&new_id, fallback).unwrap();
        
        let final_count = manager.get_desktop_count().unwrap();
        assert_eq!(final_count, initial_count, "Desktop count should return to initial");
    }
    
    #[test]
    #[ignore]
    fn test_desktop_switching() {
        let manager = VirtualDesktopManager::new().unwrap();
        
        let initial_id = manager.get_current_desktop_id().unwrap();
        println!("Initial desktop: {:?}", initial_id);
        
        // Switch to next
        manager.switch_to_next().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let next_id = manager.get_current_desktop_id().unwrap();
        println!("Next desktop: {:?}", next_id);
        
        // Switch back
        manager.switch_to_previous().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let back_id = manager.get_current_desktop_id().unwrap();
        assert_eq!(back_id, initial_id, "Should return to initial desktop");
    }
    
    #[test]
    #[ignore]
    fn test_move_window_between_desktops() {
        let manager = VirtualDesktopManager::new().unwrap();
        let desktops = manager.get_desktop_ids().unwrap();
        
        if desktops.len() < 2 {
            println!("Need at least 2 desktops for this test");
            return;
        }
        
        // Get a test window
        let windows = win32::enumerate_visible_windows().unwrap();
        if let Some(window) = windows.first() {
            let initial_desktop = manager.get_window_desktop_id(window.0).unwrap();
            
            // Move to different desktop
            let target_desktop = desktops.iter()
                .find(|&&id| id != initial_desktop)
                .unwrap();
            
            manager.move_window_to_desktop(window.0, target_desktop).unwrap();
            
            let new_desktop = manager.get_window_desktop_id(window.0).unwrap();
            assert_eq!(new_desktop, *target_desktop, "Window should be on target desktop");
            
            // Move back
            manager.move_window_to_desktop(window.0, &initial_desktop).unwrap();
        }
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core virtual_desktop -- --ignored --nocapture
```

**Manual Testing Procedure:**
1. Run test suite with multiple Virtual Desktops
2. Verify desktop switching with visual confirmation
3. Test window movement between desktops
4. Verify desktop creation and removal
5. Test wraparound navigation

---

### Week 10: Workspace Manager

#### Task 3.4: Implement Core Workspace Data Structures

**Objective:** Create data structures to represent workspaces and the workspace manager.

**File:** `crates/core/src/workspace/manager.rs`

**Required Implementations:**

1. **Workspace struct:**

   ```rust
   use crate::window_manager::tree::{TreeNode, Rect, Split};
   use crate::utils::win32::WindowHandle;
   use serde::{Serialize, Deserialize};
   
   #[derive(Debug, Clone)]
   pub struct Workspace {
       /// Unique workspace ID
       pub id: usize,
       
       /// Human-readable workspace name
       pub name: String,
       
       /// Monitor this workspace is assigned to
       pub monitor: usize,
       
       /// Layout tree for this workspace
       pub tree: TreeNode,
       
       /// Windows in this workspace (HWND values)
       pub windows: Vec<isize>,
       
       /// Virtual Desktop ID (if using Virtual Desktop integration)
       pub virtual_desktop_id: Option<windows::core::GUID>,
       
       /// Whether this workspace is currently visible
       pub visible: bool,
       
       /// Last time this workspace was active
       pub last_active: std::time::Instant,
   }
   
   impl Workspace {
       pub fn new(id: usize, name: String, monitor: usize, area: Rect) -> Self {
           Self {
               id,
               name,
               monitor,
               tree: TreeNode::new_container(Split::Vertical, area),
               windows: Vec::new(),
               virtual_desktop_id: None,
               visible: false,
               last_active: std::time::Instant::now(),
           }
       }
       
       /// Add a window to this workspace
       pub fn add_window(&mut self, hwnd: isize) {
           if !self.windows.contains(&hwnd) {
               self.windows.push(hwnd);
           }
       }
       
       /// Remove a window from this workspace
       pub fn remove_window(&mut self, hwnd: isize) -> bool {
           if let Some(pos) = self.windows.iter().position(|&w| w == hwnd) {
               self.windows.remove(pos);
               true
           } else {
               false
           }
       }
       
       /// Get the number of windows in this workspace
       pub fn window_count(&self) -> usize {
           self.windows.len()
       }
       
       /// Check if a window is in this workspace
       pub fn contains_window(&self, hwnd: isize) -> bool {
           self.windows.contains(&hwnd)
       }
       
       /// Mark this workspace as active
       pub fn mark_active(&mut self) {
           self.visible = true;
           self.last_active = std::time::Instant::now();
       }
       
       /// Mark this workspace as inactive
       pub fn mark_inactive(&mut self) {
           self.visible = false;
       }
   }
   ```

2. **WorkspaceConfig struct:**

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct WorkspaceConfig {
       /// Default number of workspaces to create
       pub default_count: usize,
       
       /// Workspace names (index = workspace number - 1)
       pub names: Vec<String>,
       
       /// Whether to persist workspace state
       pub persist_state: bool,
       
       /// Whether to create workspaces on demand
       pub create_on_demand: bool,
       
       /// Whether to use Virtual Desktop integration
       pub use_virtual_desktops: bool,
   }
   
   impl Default for WorkspaceConfig {
       fn default() -> Self {
           Self {
               default_count: 10,
               names: (1..=10).map(|i| i.to_string()).collect(),
               persist_state: true,
               create_on_demand: false,
               use_virtual_desktops: false,
           }
       }
   }
   ```

3. **WorkspaceManager struct:**

   ```rust
   use std::collections::HashMap;
   
   pub struct WorkspaceManager {
       /// All workspaces by ID
       workspaces: HashMap<usize, Workspace>,
       
       /// Currently active workspace ID
       active_workspace: usize,
       
       /// Next workspace ID to assign
       next_id: usize,
       
       /// Configuration
       config: WorkspaceConfig,
       
       /// Virtual Desktop manager (if enabled)
       vd_manager: Option<crate::workspace::virtual_desktop::VirtualDesktopManager>,
       
       /// Map of windows to their workspaces
       window_to_workspace: HashMap<isize, usize>,
   }
   
   impl WorkspaceManager {
       pub fn new(config: WorkspaceConfig) -> Self {
           Self {
               workspaces: HashMap::new(),
               active_workspace: 1,
               next_id: 1,
               config,
               vd_manager: None,
               window_to_workspace: HashMap::new(),
           }
       }
       
       pub fn with_virtual_desktops(mut self) -> anyhow::Result<Self> {
           if self.config.use_virtual_desktops {
               let vd_manager = crate::workspace::virtual_desktop::VirtualDesktopManager::new()?;
               self.vd_manager = Some(vd_manager);
           }
           Ok(self)
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Workspace struct properly tracks all required state
- [ ] WorkspaceConfig has sensible defaults
- [ ] WorkspaceManager initializes correctly
- [ ] Can create workspaces with unique IDs
- [ ] Window-to-workspace mapping is maintained
- [ ] All structs are thread-safe where needed
- [ ] Structs serialize/deserialize correctly for persistence

**Testing Requirements:**

Create `crates/core/src/workspace/manager_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_manager::tree::Rect;
    
    #[test]
    fn test_workspace_creation() {
        let area = Rect::new(0, 0, 1920, 1080);
        let ws = Workspace::new(1, "Workspace 1".to_string(), 0, area);
        
        assert_eq!(ws.id, 1);
        assert_eq!(ws.name, "Workspace 1");
        assert_eq!(ws.monitor, 0);
        assert_eq!(ws.window_count(), 0);
        assert!(!ws.visible);
    }
    
    #[test]
    fn test_workspace_add_remove_window() {
        let area = Rect::new(0, 0, 1920, 1080);
        let mut ws = Workspace::new(1, "Test".to_string(), 0, area);
        
        ws.add_window(12345);
        assert_eq!(ws.window_count(), 1);
        assert!(ws.contains_window(12345));
        
        ws.add_window(12345); // Should not add duplicate
        assert_eq!(ws.window_count(), 1);
        
        assert!(ws.remove_window(12345));
        assert_eq!(ws.window_count(), 0);
        assert!(!ws.contains_window(12345));
        
        assert!(!ws.remove_window(99999)); // Non-existent window
    }
    
    #[test]
    fn test_workspace_manager_creation() {
        let config = WorkspaceConfig::default();
        let manager = WorkspaceManager::new(config);
        
        assert_eq!(manager.active_workspace, 1);
        assert_eq!(manager.next_id, 1);
    }
    
    #[test]
    fn test_workspace_config_defaults() {
        let config = WorkspaceConfig::default();
        
        assert_eq!(config.default_count, 10);
        assert_eq!(config.names.len(), 10);
        assert!(config.persist_state);
        assert!(!config.create_on_demand);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 3.5: Implement Workspace Creation and Deletion

**Objective:** Implement methods to create, delete, and manage workspace lifecycle.

**File:** `crates/core/src/workspace/manager.rs` (continue)

**Required Implementations:**

```rust
impl WorkspaceManager {
    /// Initialize the workspace manager with default workspaces
    pub fn initialize(&mut self, monitor_areas: &[(usize, Rect)]) -> anyhow::Result<()> {
        // Create default workspaces for each monitor
        for (monitor_id, area) in monitor_areas {
            for i in 0..self.config.default_count {
                let ws_id = self.next_id;
                self.next_id += 1;
                
                let name = if i < self.config.names.len() {
                    self.config.names[i].clone()
                } else {
                    ws_id.to_string()
                };
                
                let workspace = Workspace::new(ws_id, name, *monitor_id, *area);
                self.workspaces.insert(ws_id, workspace);
                
                // First workspace on primary monitor is active
                if *monitor_id == 0 && i == 0 {
                    self.active_workspace = ws_id;
                    if let Some(ws) = self.workspaces.get_mut(&ws_id) {
                        ws.mark_active();
                    }
                }
            }
        }
        
        // Initialize Virtual Desktop integration if enabled
        if let Some(ref vd_manager) = self.vd_manager {
            self.sync_with_virtual_desktops()?;
        }
        
        Ok(())
    }
    
    /// Create a new workspace
    pub fn create_workspace(&mut self, name: String, monitor: usize, area: Rect) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        let mut workspace = Workspace::new(id, name, monitor, area);
        
        // Create corresponding Virtual Desktop if enabled
        if let Some(ref vd_manager) = self.vd_manager {
            if let Ok(vd_id) = vd_manager.create_desktop() {
                workspace.virtual_desktop_id = Some(vd_id);
            }
        }
        
        self.workspaces.insert(id, workspace);
        id
    }
    
    /// Delete a workspace (moves windows to fallback workspace)
    pub fn delete_workspace(&mut self, workspace_id: usize, fallback_id: usize) -> anyhow::Result<()> {
        if workspace_id == fallback_id {
            anyhow::bail!("Cannot delete workspace into itself");
        }
        
        if !self.workspaces.contains_key(&workspace_id) {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
        
        if !self.workspaces.contains_key(&fallback_id) {
            anyhow::bail!("Fallback workspace {} does not exist", fallback_id);
        }
        
        // Move all windows to fallback workspace
        let workspace = self.workspaces.get(&workspace_id).unwrap();
        let windows_to_move: Vec<isize> = workspace.windows.clone();
        
        for hwnd in windows_to_move {
            self.move_window_to_workspace(hwnd, workspace_id, fallback_id)?;
        }
        
        // Remove Virtual Desktop if using VD integration
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(workspace) = self.workspaces.get(&workspace_id) {
                if let Some(vd_id) = workspace.virtual_desktop_id {
                    if let Some(fallback_ws) = self.workspaces.get(&fallback_id) {
                        if let Some(fallback_vd_id) = fallback_ws.virtual_desktop_id {
                            let _ = vd_manager.remove_desktop(&vd_id, &fallback_vd_id);
                        }
                    }
                }
            }
        }
        
        // Switch to fallback if deleting active workspace
        if self.active_workspace == workspace_id {
            self.switch_to(fallback_id)?;
        }
        
        // Remove workspace
        self.workspaces.remove(&workspace_id);
        
        Ok(())
    }
    
    /// Rename a workspace
    pub fn rename_workspace(&mut self, workspace_id: usize, new_name: String) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.name = new_name;
            Ok(())
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
    }
    
    /// Get a workspace by ID
    pub fn get_workspace(&self, workspace_id: usize) -> Option<&Workspace> {
        self.workspaces.get(&workspace_id)
    }
    
    /// Get a mutable workspace by ID
    pub fn get_workspace_mut(&mut self, workspace_id: usize) -> Option<&mut Workspace> {
        self.workspaces.get_mut(&workspace_id)
    }
    
    /// Get all workspaces
    pub fn get_all_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }
    
    /// Get workspaces for a specific monitor
    pub fn get_workspaces_for_monitor(&self, monitor_id: usize) -> Vec<&Workspace> {
        self.workspaces
            .values()
            .filter(|ws| ws.monitor == monitor_id)
            .collect()
    }
    
    /// Get the currently active workspace ID
    pub fn get_active(&self) -> usize {
        self.active_workspace
    }
    
    /// Sync with Windows Virtual Desktops
    fn sync_with_virtual_desktops(&mut self) -> anyhow::Result<()> {
        if let Some(ref vd_manager) = self.vd_manager {
            let vd_ids = vd_manager.get_desktop_ids()?;
            
            // Match existing workspaces to Virtual Desktops
            let mut workspace_ids: Vec<usize> = self.workspaces.keys().copied().collect();
            workspace_ids.sort();
            
            for (ws_id, vd_id) in workspace_ids.iter().zip(vd_ids.iter()) {
                if let Some(workspace) = self.workspaces.get_mut(ws_id) {
                    workspace.virtual_desktop_id = Some(*vd_id);
                }
            }
        }
        
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Can initialize with default workspaces
- [ ] Can create new workspaces dynamically
- [ ] Can delete workspaces with window migration
- [ ] Can rename workspaces
- [ ] Virtual Desktop integration works when enabled
- [ ] Workspace IDs are unique and sequential
- [ ] Cannot delete the last workspace
- [ ] Error handling is comprehensive

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_initialization() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        
        manager.initialize(&monitor_areas).unwrap();
        
        let workspaces = manager.get_all_workspaces();
        assert_eq!(workspaces.len(), 10);
        assert_eq!(manager.get_active(), 1);
    }
    
    #[test]
    fn test_create_and_delete_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        let initial_count = manager.get_all_workspaces().len();
        
        // Create new workspace
        let new_id = manager.create_workspace("Test".to_string(), 0, area);
        assert_eq!(manager.get_all_workspaces().len(), initial_count + 1);
        
        // Delete it
        manager.delete_workspace(new_id, 1).unwrap();
        assert_eq!(manager.get_all_workspaces().len(), initial_count);
    }
    
    #[test]
    fn test_rename_workspace() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let ws_id = manager.create_workspace("Original".to_string(), 0, area);
        
        manager.rename_workspace(ws_id, "Renamed".to_string()).unwrap();
        
        let workspace = manager.get_workspace(ws_id).unwrap();
        assert_eq!(workspace.name, "Renamed");
    }
    
    #[test]
    fn test_get_workspaces_for_monitor() {
        let config = WorkspaceConfig {
            default_count: 5,
            ..Default::default()
        };
        let mut manager = WorkspaceManager::new(config);
        
        let area1 = Rect::new(0, 0, 1920, 1080);
        let area2 = Rect::new(1920, 0, 1920, 1080);
        let monitor_areas = vec![(0, area1), (1, area2)];
        
        manager.initialize(&monitor_areas).unwrap();
        
        let monitor0_workspaces = manager.get_workspaces_for_monitor(0);
        let monitor1_workspaces = manager.get_workspaces_for_monitor(1);
        
        assert_eq!(monitor0_workspaces.len(), 5);
        assert_eq!(monitor1_workspaces.len(), 5);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
```

---

#### Task 3.6: Implement Workspace Switching

**Objective:** Implement workspace switching with window show/hide logic.

**File:** `crates/core/src/workspace/manager.rs` (continue)

**Required Implementations:**

```rust
impl WorkspaceManager {
    /// Switch to a different workspace
    pub fn switch_to(&mut self, workspace_id: usize) -> anyhow::Result<()> {
        if !self.workspaces.contains_key(&workspace_id) {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
        
        if self.active_workspace == workspace_id {
            // Already on this workspace
            return Ok(());
        }
        
        tracing::info!("Switching from workspace {} to {}", self.active_workspace, workspace_id);
        
        // Hide windows from current workspace
        if let Some(current) = self.workspaces.get_mut(&self.active_workspace) {
            current.mark_inactive();
            
            for &hwnd in &current.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    ShowWindow(HWND(hwnd), SW_HIDE);
                }
            }
        }
        
        // Show windows from target workspace
        if let Some(target) = self.workspaces.get_mut(&workspace_id) {
            target.mark_active();
            
            for &hwnd in &target.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    ShowWindow(HWND(hwnd), SW_SHOW);
                }
            }
            
            // Re-apply layout geometry
            target.tree.apply_geometry()?;
        }
        
        // Switch Virtual Desktop if enabled
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(workspace) = self.workspaces.get(&workspace_id) {
                if let Some(vd_id) = workspace.virtual_desktop_id {
                    vd_manager.switch_desktop_by_id(&vd_id)?;
                }
            }
        }
        
        self.active_workspace = workspace_id;
        
        tracing::info!("Successfully switched to workspace {}", workspace_id);
        Ok(())
    }
    
    /// Switch to the next workspace
    pub fn switch_to_next(&mut self) -> anyhow::Result<()> {
        let current_monitor = self.workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);
        
        // Get workspaces on the same monitor, sorted by ID
        let mut monitor_workspaces: Vec<usize> = self.workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();
        
        if let Some(current_idx) = monitor_workspaces.iter().position(|&id| id == self.active_workspace) {
            let next_idx = (current_idx + 1) % monitor_workspaces.len();
            let next_id = monitor_workspaces[next_idx];
            self.switch_to(next_id)?;
        }
        
        Ok(())
    }
    
    /// Switch to the previous workspace
    pub fn switch_to_previous(&mut self) -> anyhow::Result<()> {
        let current_monitor = self.workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);
        
        // Get workspaces on the same monitor, sorted by ID
        let mut monitor_workspaces: Vec<usize> = self.workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();
        
        if let Some(current_idx) = monitor_workspaces.iter().position(|&id| id == self.active_workspace) {
            let prev_idx = if current_idx == 0 {
                monitor_workspaces.len() - 1
            } else {
                current_idx - 1
            };
            let prev_id = monitor_workspaces[prev_idx];
            self.switch_to(prev_id)?;
        }
        
        Ok(())
    }
    
    /// Switch to a workspace by index on the current monitor (1-based)
    pub fn switch_to_index(&mut self, index: usize) -> anyhow::Result<()> {
        if index == 0 {
            anyhow::bail!("Workspace index must be >= 1");
        }
        
        let current_monitor = self.workspaces
            .get(&self.active_workspace)
            .map(|ws| ws.monitor)
            .unwrap_or(0);
        
        // Get workspaces on the same monitor, sorted by ID
        let mut monitor_workspaces: Vec<usize> = self.workspaces
            .values()
            .filter(|ws| ws.monitor == current_monitor)
            .map(|ws| ws.id)
            .collect();
        monitor_workspaces.sort();
        
        if index > monitor_workspaces.len() {
            anyhow::bail!("Workspace index {} out of range (max: {})", index, monitor_workspaces.len());
        }
        
        let target_id = monitor_workspaces[index - 1];
        self.switch_to(target_id)
    }
}
```

**Acceptance Criteria:**
- [ ] Can switch between workspaces by ID
- [ ] Can switch to next/previous workspace
- [ ] Can switch by index (1-based)
- [ ] Windows are hidden on old workspace
- [ ] Windows are shown on new workspace
- [ ] Layout geometry is re-applied after switch
- [ ] Virtual Desktop switches when enabled
- [ ] Per-monitor workspace switching works correctly
- [ ] Switching completes within 200ms

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workspace_switching() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        assert_eq!(manager.get_active(), 1);
        
        // Switch to workspace 2
        manager.switch_to(2).unwrap();
        assert_eq!(manager.get_active(), 2);
        
        // Switch back
        manager.switch_to(1).unwrap();
        assert_eq!(manager.get_active(), 1);
    }
    
    #[test]
    fn test_next_previous_switching() {
        let config = WorkspaceConfig {
            default_count: 3,
            ..Default::default()
        };
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        assert_eq!(manager.get_active(), 1);
        
        // Go to next
        manager.switch_to_next().unwrap();
        assert_eq!(manager.get_active(), 2);
        
        manager.switch_to_next().unwrap();
        assert_eq!(manager.get_active(), 3);
        
        // Wrap around
        manager.switch_to_next().unwrap();
        assert_eq!(manager.get_active(), 1);
        
        // Go to previous
        manager.switch_to_previous().unwrap();
        assert_eq!(manager.get_active(), 3);
    }
    
    #[test]
    fn test_switch_by_index() {
        let config = WorkspaceConfig {
            default_count: 5,
            ..Default::default()
        };
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        manager.switch_to_index(3).unwrap();
        assert_eq!(manager.get_active(), 3);
        
        manager.switch_to_index(5).unwrap();
        assert_eq!(manager.get_active(), 5);
        
        // Invalid index
        let result = manager.switch_to_index(10);
        assert!(result.is_err());
    }
    
    #[test]
    #[ignore] // Requires actual windows
    fn test_window_visibility_on_switch() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        // Add a window to workspace 1
        manager.add_window_to_workspace(12345, 1).unwrap();
        
        // Switch to workspace 2
        manager.switch_to(2).unwrap();
        
        // Window on workspace 1 should be hidden
        // Manual verification needed
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
```

---

#### Task 3.7: Implement Window-to-Workspace Management

**Objective:** Implement functions to add, remove, and move windows between workspaces.

**File:** `crates/core/src/workspace/manager.rs` (continue)

**Required Implementations:**

```rust
impl WorkspaceManager {
    /// Add a window to a workspace
    pub fn add_window_to_workspace(&mut self, hwnd: isize, workspace_id: usize) -> anyhow::Result<()> {
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.add_window(hwnd);
            self.window_to_workspace.insert(hwnd, workspace_id);
            
            // Show window if on active workspace, hide otherwise
            unsafe {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::Win32::Foundation::HWND;
                
                if workspace_id == self.active_workspace {
                    ShowWindow(HWND(hwnd), SW_SHOW);
                } else {
                    ShowWindow(HWND(hwnd), SW_HIDE);
                }
            }
            
            Ok(())
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
    }
    
    /// Remove a window from its workspace
    pub fn remove_window(&mut self, hwnd: isize) -> anyhow::Result<Option<usize>> {
        if let Some(&workspace_id) = self.window_to_workspace.get(&hwnd) {
            if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
                workspace.remove_window(hwnd);
            }
            self.window_to_workspace.remove(&hwnd);
            Ok(Some(workspace_id))
        } else {
            Ok(None)
        }
    }
    
    /// Move a window from one workspace to another
    pub fn move_window_to_workspace(
        &mut self,
        hwnd: isize,
        from_workspace: usize,
        to_workspace: usize,
    ) -> anyhow::Result<()> {
        if from_workspace == to_workspace {
            return Ok(());
        }
        
        // Remove from source workspace
        if let Some(from_ws) = self.workspaces.get_mut(&from_workspace) {
            from_ws.remove_window(hwnd);
        }
        
        // Add to target workspace
        if let Some(to_ws) = self.workspaces.get_mut(&to_workspace) {
            to_ws.add_window(hwnd);
        } else {
            anyhow::bail!("Target workspace {} does not exist", to_workspace);
        }
        
        // Update mapping
        self.window_to_workspace.insert(hwnd, to_workspace);
        
        // Update window visibility
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::*;
            use windows::Win32::Foundation::HWND;
            
            if to_workspace == self.active_workspace {
                ShowWindow(HWND(hwnd), SW_SHOW);
            } else {
                ShowWindow(HWND(hwnd), SW_HIDE);
            }
        }
        
        // Move to Virtual Desktop if enabled
        if let Some(ref vd_manager) = self.vd_manager {
            if let Some(to_ws) = self.workspaces.get(&to_workspace) {
                if let Some(vd_id) = to_ws.virtual_desktop_id {
                    vd_manager.move_window_to_desktop(
                        windows::Win32::Foundation::HWND(hwnd),
                        &vd_id,
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the workspace ID for a window
    pub fn get_window_workspace(&self, hwnd: isize) -> Option<usize> {
        self.window_to_workspace.get(&hwnd).copied()
    }
    
    /// Move the currently focused window to a different workspace
    pub fn move_active_window_to_workspace(&mut self, target_workspace: usize) -> anyhow::Result<()> {
        // Get the foreground window
        let fg_window = crate::utils::win32::get_foreground_window()
            .ok_or_else(|| anyhow::anyhow!("No foreground window"))?;
        
        let hwnd = fg_window.0 .0;
        
        if let Some(current_workspace) = self.window_to_workspace.get(&hwnd).copied() {
            self.move_window_to_workspace(hwnd, current_workspace, target_workspace)?;
        }
        
        Ok(())
    }
    
    /// Move the active window to a workspace and follow it
    pub fn move_active_window_and_follow(&mut self, target_workspace: usize) -> anyhow::Result<()> {
        self.move_active_window_to_workspace(target_workspace)?;
        self.switch_to(target_workspace)?;
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Can add windows to workspaces
- [ ] Can remove windows from workspaces
- [ ] Can move windows between workspaces
- [ ] Window visibility updates correctly
- [ ] Window-to-workspace mapping is maintained
- [ ] Virtual Desktop moves work when enabled
- [ ] Active window movement works
- [ ] Move and follow works correctly

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_remove_window() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        // Add window to workspace 1
        manager.add_window_to_workspace(12345, 1).unwrap();
        assert_eq!(manager.get_window_workspace(12345), Some(1));
        
        let ws = manager.get_workspace(1).unwrap();
        assert_eq!(ws.window_count(), 1);
        
        // Remove window
        let removed_from = manager.remove_window(12345).unwrap();
        assert_eq!(removed_from, Some(1));
        assert_eq!(manager.get_window_workspace(12345), None);
        
        let ws = manager.get_workspace(1).unwrap();
        assert_eq!(ws.window_count(), 0);
    }
    
    #[test]
    fn test_move_window_between_workspaces() {
        let config = WorkspaceConfig::default();
        let mut manager = WorkspaceManager::new(config);
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        // Add window to workspace 1
        manager.add_window_to_workspace(12345, 1).unwrap();
        assert_eq!(manager.get_window_workspace(12345), Some(1));
        
        // Move to workspace 2
        manager.move_window_to_workspace(12345, 1, 2).unwrap();
        assert_eq!(manager.get_window_workspace(12345), Some(2));
        
        let ws1 = manager.get_workspace(1).unwrap();
        assert_eq!(ws1.window_count(), 0);
        
        let ws2 = manager.get_workspace(2).unwrap();
        assert_eq!(ws2.window_count(), 1);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
```

---

### Week 11: Per-Monitor Workspace Support

#### Task 3.8: Integrate Monitor Manager with Workspace System

**Objective:** Connect the monitor manager with the workspace system to support per-monitor workspaces.

**Files:** 
- `crates/core/src/window_manager/monitor.rs` (from Phase 2, enhance)
- `crates/core/src/workspace/manager.rs` (update)

**Required Implementations:**

1. **Update MonitorInfo to track workspaces:**

```rust
// In monitor.rs
pub struct MonitorInfo {
    pub id: usize,
    pub handle: windows::Win32::Graphics::Gdi::HMONITOR,
    pub name: String,
    pub work_area: crate::window_manager::tree::Rect,
    pub full_area: crate::window_manager::tree::Rect,
    pub dpi_scale: f32,
    pub workspaces: Vec<usize>, // Workspace IDs assigned to this monitor
    pub active_workspace: Option<usize>, // Currently active workspace on this monitor
}
```

2. **Add monitor management methods to WorkspaceManager:**

```rust
impl WorkspaceManager {
    /// Assign workspaces to monitors
    pub fn assign_workspaces_to_monitors(
        &mut self,
        monitor_manager: &mut crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        // Clear existing assignments
        for monitor in monitor_manager.monitors.values_mut() {
            monitor.workspaces.clear();
        }
        
        // Assign each workspace to its monitor
        for workspace in self.workspaces.values() {
            if let Some(monitor) = monitor_manager.get_by_id_mut(workspace.monitor) {
                monitor.workspaces.push(workspace.id);
                
                // Set active workspace for monitor
                if workspace.visible {
                    monitor.active_workspace = Some(workspace.id);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the active workspace for a specific monitor
    pub fn get_active_workspace_for_monitor(&self, monitor_id: usize) -> Option<usize> {
        self.workspaces
            .values()
            .find(|ws| ws.monitor == monitor_id && ws.visible)
            .map(|ws| ws.id)
    }
    
    /// Switch workspace on a specific monitor
    pub fn switch_workspace_on_monitor(
        &mut self,
        monitor_id: usize,
        workspace_id: usize,
    ) -> anyhow::Result<()> {
        // Verify workspace is on the correct monitor
        if let Some(workspace) = self.workspaces.get(&workspace_id) {
            if workspace.monitor != monitor_id {
                anyhow::bail!(
                    "Workspace {} is on monitor {}, not monitor {}",
                    workspace_id,
                    workspace.monitor,
                    monitor_id
                );
            }
        } else {
            anyhow::bail!("Workspace {} does not exist", workspace_id);
        }
        
        // Hide current workspace on this monitor
        let current_on_monitor: Vec<usize> = self.workspaces
            .values()
            .filter(|ws| ws.monitor == monitor_id && ws.visible)
            .map(|ws| ws.id)
            .collect();
        
        for ws_id in current_on_monitor {
            if let Some(ws) = self.workspaces.get_mut(&ws_id) {
                ws.mark_inactive();
                for &hwnd in &ws.windows {
                    unsafe {
                        use windows::Win32::UI::WindowsAndMessaging::*;
                        use windows::Win32::Foundation::HWND;
                        ShowWindow(HWND(hwnd), SW_HIDE);
                    }
                }
            }
        }
        
        // Show target workspace
        if let Some(workspace) = self.workspaces.get_mut(&workspace_id) {
            workspace.mark_active();
            
            for &hwnd in &workspace.windows {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    ShowWindow(HWND(hwnd), SW_SHOW);
                }
            }
            
            workspace.tree.apply_geometry()?;
        }
        
        // Update active workspace if it's on the focused monitor
        // (This would need to check which monitor has focus)
        
        Ok(())
    }
    
    /// Redistribute workspaces when monitors change
    pub fn handle_monitor_change(
        &mut self,
        monitor_manager: &crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        let monitor_count = monitor_manager.monitors.len();
        
        // If monitors were removed, move workspaces to remaining monitors
        for workspace in self.workspaces.values_mut() {
            if workspace.monitor >= monitor_count {
                // Move to primary monitor
                workspace.monitor = 0;
                
                // Update area to match new monitor
                if let Some(monitor) = monitor_manager.get_by_id(0) {
                    workspace.tree.rect = monitor.work_area;
                }
            } else {
                // Update area to match current monitor (in case resolution changed)
                if let Some(monitor) = monitor_manager.get_by_id(workspace.monitor) {
                    workspace.tree.rect = monitor.work_area;
                }
            }
        }
        
        Ok(())
    }
}
```

**Acceptance Criteria:**
- [ ] Workspaces are properly assigned to monitors
- [ ] Can get active workspace for each monitor
- [ ] Can switch workspaces on a specific monitor
- [ ] Monitor changes redistribute workspaces correctly
- [ ] DPI changes update workspace geometries
- [ ] Independent workspace switching per monitor

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_per_monitor_workspaces() {
        let config = WorkspaceConfig {
            default_count: 3,
            ..Default::default()
        };
        let mut manager = WorkspaceManager::new(config);
        
        let area1 = Rect::new(0, 0, 1920, 1080);
        let area2 = Rect::new(1920, 0, 1920, 1080);
        let monitor_areas = vec![(0, area1), (1, area2)];
        
        manager.initialize(&monitor_areas).unwrap();
        
        // Verify workspaces are distributed
        let mon0_ws = manager.get_workspaces_for_monitor(0);
        let mon1_ws = manager.get_workspaces_for_monitor(1);
        
        assert_eq!(mon0_ws.len(), 3);
        assert_eq!(mon1_ws.len(), 3);
    }
    
    #[test]
    fn test_switch_workspace_on_monitor() {
        let config = WorkspaceConfig {
            default_count: 3,
            ..Default::default()
        };
        let mut manager = WorkspaceManager::new(config);
        
        let area1 = Rect::new(0, 0, 1920, 1080);
        let area2 = Rect::new(1920, 0, 1920, 1080);
        let monitor_areas = vec![(0, area1), (1, area2)];
        
        manager.initialize(&monitor_areas).unwrap();
        
        // Get workspace IDs for monitor 1
        let mon1_workspaces: Vec<usize> = manager.get_workspaces_for_monitor(1)
            .iter()
            .map(|ws| ws.id)
            .collect();
        
        let ws1_id = mon1_workspaces[0];
        let ws2_id = mon1_workspaces[1];
        
        // Switch workspace on monitor 1
        manager.switch_workspace_on_monitor(1, ws2_id).unwrap();
        
        // Verify it switched
        let active = manager.get_active_workspace_for_monitor(1);
        assert_eq!(active, Some(ws2_id));
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
```

---

#### Task 3.9: Handle DPI Awareness for Multi-Monitor

**Objective:** Ensure workspace geometries correctly handle different DPI settings across monitors.

**File:** `crates/core/src/workspace/manager.rs` (continue)

**Required Implementations:**

```rust
impl WorkspaceManager {
    /// Update workspace geometries based on DPI scaling
    pub fn update_dpi_scaling(
        &mut self,
        monitor_manager: &crate::window_manager::monitor::MonitorManager,
    ) -> anyhow::Result<()> {
        for workspace in self.workspaces.values_mut() {
            if let Some(monitor) = monitor_manager.get_by_id(workspace.monitor) {
                // Update workspace area with DPI-aware coordinates
                workspace.tree.rect = monitor.work_area;
                
                // Re-apply geometry to all windows
                workspace.tree.apply_geometry()?;
            }
        }
        
        Ok(())
    }
    
    /// Apply DPI scaling to a rect
    fn apply_dpi_scaling(rect: &mut crate::window_manager::tree::Rect, dpi_scale: f32) {
        if (dpi_scale - 1.0).abs() > 0.01 {
            rect.x = (rect.x as f32 * dpi_scale) as i32;
            rect.y = (rect.y as f32 * dpi_scale) as i32;
            rect.width = (rect.width as f32 * dpi_scale) as i32;
            rect.height = (rect.height as f32 * dpi_scale) as i32;
        }
    }
}
```

**Acceptance Criteria:**
- [ ] DPI scaling is applied correctly to workspace geometries
- [ ] Windows position correctly on high-DPI monitors
- [ ] DPI changes trigger geometry updates
- [ ] Mixed DPI environments work correctly

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dpi_scaling() {
        let mut rect = Rect::new(0, 0, 1920, 1080);
        WorkspaceManager::apply_dpi_scaling(&mut rect, 1.5);
        
        assert_eq!(rect.width, 2880);
        assert_eq!(rect.height, 1620);
    }
    
    #[test]
    #[ignore] // Requires multi-monitor setup with different DPI
    fn test_mixed_dpi_workspaces() {
        // Manual testing on system with multiple monitors at different DPI
    }
}
```

---

### Week 12: Workspace Persistence

#### Task 3.10: Implement Workspace State Serialization

**Objective:** Implement serialization and deserialization of workspace state for persistence.

**File:** `crates/core/src/workspace/persistence.rs`

**Required Implementations:**

```rust
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceState {
    pub id: usize,
    pub name: String,
    pub monitor: usize,
    pub windows: Vec<WindowState>,
    pub virtual_desktop_id: Option<String>, // Serialized GUID
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowState {
    pub hwnd: String, // Serialized as string for readability
    pub process_name: String,
    pub title: String,
    pub class_name: String,
    pub workspace: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionState {
    pub version: String,
    pub timestamp: u64,
    pub workspaces: Vec<WorkspaceState>,
    pub active_workspace: usize,
    pub window_to_workspace: std::collections::HashMap<String, usize>,
}

pub struct PersistenceManager {
    state_file: PathBuf,
    backup_file: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Self {
        let state_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tiling-wm");
        
        // Create directory if it doesn't exist
        fs::create_dir_all(&state_dir).ok();
        
        Self {
            state_file: state_dir.join("session.json"),
            backup_file: state_dir.join("session.backup.json"),
        }
    }
    
    pub fn with_custom_path(path: PathBuf) -> Self {
        let backup_path = path.with_extension("backup.json");
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        
        Self {
            state_file: path,
            backup_file: backup_path,
        }
    }
    
    /// Save workspace state to disk
    pub fn save_state(&self, state: &SessionState) -> anyhow::Result<()> {
        // Backup existing file
        if self.state_file.exists() {
            fs::copy(&self.state_file, &self.backup_file)?;
        }
        
        // Write new state
        let json = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, json)?;
        
        tracing::info!("Saved workspace state to {:?}", self.state_file);
        Ok(())
    }
    
    /// Load workspace state from disk
    pub fn load_state(&self) -> anyhow::Result<SessionState> {
        if !self.state_file.exists() {
            anyhow::bail!("State file does not exist");
        }
        
        let json = fs::read_to_string(&self.state_file)?;
        let state: SessionState = serde_json::from_str(&json)?;
        
        tracing::info!("Loaded workspace state from {:?}", self.state_file);
        Ok(state)
    }
    
    /// Try to load state, fallback to backup if corrupted
    pub fn load_state_with_fallback(&self) -> anyhow::Result<SessionState> {
        match self.load_state() {
            Ok(state) => Ok(state),
            Err(e) => {
                tracing::warn!("Failed to load state: {}. Trying backup...", e);
                
                if self.backup_file.exists() {
                    let json = fs::read_to_string(&self.backup_file)?;
                    let state: SessionState = serde_json::from_str(&json)?;
                    
                    tracing::info!("Loaded workspace state from backup");
                    Ok(state)
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Clear saved state
    pub fn clear_state(&self) -> anyhow::Result<()> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file)?;
        }
        if self.backup_file.exists() {
            fs::remove_file(&self.backup_file)?;
        }
        
        tracing::info!("Cleared workspace state");
        Ok(())
    }
    
    /// Check if state file exists
    pub fn has_saved_state(&self) -> bool {
        self.state_file.exists() || self.backup_file.exists()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            workspaces: Vec::new(),
            active_workspace: 1,
            window_to_workspace: std::collections::HashMap::new(),
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Can serialize workspace state to JSON
- [ ] Can deserialize workspace state from JSON
- [ ] Handles corrupted state files gracefully
- [ ] Creates backup before overwriting
- [ ] Includes version information
- [ ] Timestamp tracks last save
- [ ] Error handling is comprehensive

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_save_and_load_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        let persistence = PersistenceManager::with_custom_path(state_path.clone());
        
        // Create test state
        let mut state = SessionState::default();
        state.active_workspace = 3;
        state.workspaces.push(WorkspaceState {
            id: 1,
            name: "Workspace 1".to_string(),
            monitor: 0,
            windows: vec![],
            virtual_desktop_id: None,
        });
        
        // Save
        persistence.save_state(&state).unwrap();
        assert!(state_path.exists());
        
        // Load
        let loaded = persistence.load_state().unwrap();
        assert_eq!(loaded.active_workspace, 3);
        assert_eq!(loaded.workspaces.len(), 1);
        assert_eq!(loaded.workspaces[0].name, "Workspace 1");
    }
    
    #[test]
    fn test_backup_on_save() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        let persistence = PersistenceManager::with_custom_path(state_path.clone());
        
        // Save first state
        let state1 = SessionState::default();
        persistence.save_state(&state1).unwrap();
        
        // Save second state (should create backup)
        let mut state2 = SessionState::default();
        state2.active_workspace = 5;
        persistence.save_state(&state2).unwrap();
        
        // Check backup exists
        assert!(persistence.backup_file.exists());
    }
    
    #[test]
    fn test_load_with_fallback() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        let persistence = PersistenceManager::with_custom_path(state_path.clone());
        
        // Save valid state
        let state = SessionState::default();
        persistence.save_state(&state).unwrap();
        
        // Corrupt main file
        fs::write(&state_path, "invalid json").unwrap();
        
        // Should load from backup
        let loaded = persistence.load_state_with_fallback();
        assert!(loaded.is_ok());
    }
    
    #[test]
    fn test_clear_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        
        let persistence = PersistenceManager::with_custom_path(state_path.clone());
        
        // Save state
        let state = SessionState::default();
        persistence.save_state(&state).unwrap();
        
        assert!(persistence.has_saved_state());
        
        // Clear
        persistence.clear_state().unwrap();
        
        assert!(!persistence.has_saved_state());
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core persistence
cargo clippy -p tiling-wm-core -- -D warnings
```

---

#### Task 3.11: Integrate Persistence with WorkspaceManager

**Objective:** Add methods to WorkspaceManager to save and restore state.

**File:** `crates/core/src/workspace/manager.rs` (continue)

**Required Implementations:**

```rust
use crate::workspace::persistence::{PersistenceManager, SessionState, WorkspaceState, WindowState};

impl WorkspaceManager {
    /// Save current workspace state to disk
    pub fn save_state(&self, persistence: &PersistenceManager) -> anyhow::Result<()> {
        if !self.config.persist_state {
            return Ok(());
        }
        
        let mut state = SessionState::default();
        state.active_workspace = self.active_workspace;
        
        // Serialize workspaces
        for workspace in self.workspaces.values() {
            let ws_state = WorkspaceState {
                id: workspace.id,
                name: workspace.name.clone(),
                monitor: workspace.monitor,
                windows: workspace.windows
                    .iter()
                    .filter_map(|&hwnd| {
                        let handle = crate::utils::win32::WindowHandle::from_hwnd(
                            windows::Win32::Foundation::HWND(hwnd)
                        );
                        
                        // Try to get window info
                        if let (Ok(title), Ok(class), Ok(process)) = (
                            handle.get_title(),
                            handle.get_class_name(),
                            handle.get_process_name(),
                        ) {
                            Some(WindowState {
                                hwnd: format!("{}", hwnd),
                                process_name: process,
                                title,
                                class_name: class,
                                workspace: workspace.id,
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
                virtual_desktop_id: workspace.virtual_desktop_id
                    .map(|guid| format!("{:?}", guid)),
            };
            
            state.workspaces.push(ws_state);
        }
        
        // Serialize window-to-workspace mapping
        for (&hwnd, &workspace_id) in &self.window_to_workspace {
            state.window_to_workspace.insert(format!("{}", hwnd), workspace_id);
        }
        
        persistence.save_state(&state)?;
        Ok(())
    }
    
    /// Restore workspace state from disk
    pub fn restore_state(
        &mut self,
        persistence: &PersistenceManager,
        monitor_areas: &[(usize, crate::window_manager::tree::Rect)],
    ) -> anyhow::Result<()> {
        if !self.config.persist_state {
            return Ok(());
        }
        
        let state = persistence.load_state_with_fallback()?;
        
        tracing::info!("Restoring workspace state (version: {})", state.version);
        
        // Clear existing workspaces
        self.workspaces.clear();
        self.window_to_workspace.clear();
        
        // Restore workspaces
        for ws_state in state.workspaces {
            // Find monitor area
            let area = monitor_areas
                .iter()
                .find(|(id, _)| *id == ws_state.monitor)
                .map(|(_, area)| *area)
                .unwrap_or_else(|| monitor_areas[0].1);
            
            let mut workspace = Workspace::new(
                ws_state.id,
                ws_state.name,
                ws_state.monitor,
                area,
            );
            
            // Parse Virtual Desktop ID if present
            if let Some(vd_id_str) = ws_state.virtual_desktop_id {
                // Parse GUID from Debug format string (e.g., "{12345678-...}")
                // Implementation note: Use uuid crate or parse manually
                // Example: let guid = uuid::Uuid::parse_str(&vd_id_str).ok()
                //                    .map(|u| windows::core::GUID::from_u128(u.as_u128()));
                // For now, skip GUID restoration as VDs may have changed since save
                workspace.virtual_desktop_id = None;
            }
            
            // Note: Windows are not restored here because HWNDs are not persistent
            // Window tracking will happen as windows are rediscovered
            
            self.workspaces.insert(workspace.id, workspace);
            
            // Update next_id
            if ws_state.id >= self.next_id {
                self.next_id = ws_state.id + 1;
            }
        }
        
        // Restore active workspace
        if self.workspaces.contains_key(&state.active_workspace) {
            self.active_workspace = state.active_workspace;
            
            if let Some(ws) = self.workspaces.get_mut(&state.active_workspace) {
                ws.mark_active();
            }
        }
        
        tracing::info!("Restored {} workspaces", self.workspaces.len());
        Ok(())
    }
    
    /// Auto-save workspace state (called periodically)
    pub fn auto_save(&self, persistence: &PersistenceManager) {
        if let Err(e) = self.save_state(persistence) {
            tracing::error!("Failed to auto-save workspace state: {}", e);
        }
    }
}
```

**Acceptance Criteria:**
- [ ] Can save workspace state successfully
- [ ] Can restore workspace state on startup
- [ ] Handles missing/corrupted state gracefully
- [ ] Window information is preserved where possible
- [ ] Virtual Desktop IDs are preserved
- [ ] Auto-save works without blocking
- [ ] State versioning is handled

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_save_and_restore_workspace_state() {
        let temp_dir = tempdir().unwrap();
        let state_path = temp_dir.path().join("test_session.json");
        let persistence = PersistenceManager::with_custom_path(state_path);
        
        let mut config = WorkspaceConfig::default();
        config.persist_state = true;
        config.default_count = 3;
        
        let mut manager = WorkspaceManager::new(config.clone());
        
        let area = Rect::new(0, 0, 1920, 1080);
        let monitor_areas = vec![(0, area)];
        manager.initialize(&monitor_areas).unwrap();
        
        // Switch to workspace 2
        manager.switch_to(2).unwrap();
        
        // Save state
        manager.save_state(&persistence).unwrap();
        
        // Create new manager and restore
        let mut manager2 = WorkspaceManager::new(config);
        manager2.restore_state(&persistence, &monitor_areas).unwrap();
        
        assert_eq!(manager2.get_active(), 2);
        assert_eq!(manager2.get_all_workspaces().len(), 3);
    }
}
```

**Validation Commands:**
```bash
cargo test -p tiling-wm-core workspace::manager
```

---

#### Task 3.12: Implement Periodic Auto-Save

**Objective:** Add automatic periodic saving of workspace state.

**File:** `crates/core/src/workspace/auto_save.rs`

**Required Implementations:**

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

pub struct AutoSaver {
    manager: Arc<Mutex<crate::workspace::manager::WorkspaceManager>>,
    persistence: Arc<crate::workspace::persistence::PersistenceManager>,
    interval: Duration,
    running: Arc<Mutex<bool>>,
}

impl AutoSaver {
    pub fn new(
        manager: Arc<Mutex<crate::workspace::manager::WorkspaceManager>>,
        persistence: Arc<crate::workspace::persistence::PersistenceManager>,
        interval_secs: u64,
    ) -> Self {
        Self {
            manager,
            persistence,
            interval: Duration::from_secs(interval_secs),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the auto-save task
    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        if *running {
            return;
        }
        *running = true;
        drop(running);
        
        let manager = self.manager.clone();
        let persistence = self.persistence.clone();
        let interval = self.interval;
        let running = self.running.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                let should_continue = *running.lock().await;
                if !should_continue {
                    break;
                }
                
                // Save state
                let manager_guard = manager.lock().await;
                manager_guard.auto_save(&persistence);
                drop(manager_guard);
                
                tracing::debug!("Auto-saved workspace state");
            }
        });
    }
    
    /// Stop the auto-save task
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }
}
```

**Acceptance Criteria:**
- [ ] Auto-save runs periodically without blocking
- [ ] Auto-save can be started and stopped
- [ ] Errors in auto-save don't crash the application
- [ ] Auto-save interval is configurable
- [ ] Performance impact is minimal

**Testing Requirements:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auto_saver() {
        // Test that auto-saver starts and stops
        // Note: Full test requires async runtime
    }
}
```

---

## Phase 3 Completion Checklist

### Build & Compilation
- [ ] `cargo build --workspace` succeeds without errors
- [ ] `cargo build --workspace --release` succeeds
- [ ] No warnings from `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted with `cargo fmt --workspace --check`

### Core Functionality
- [ ] Virtual Desktop COM interfaces work correctly
- [ ] Can enumerate Virtual Desktops
- [ ] Can create and delete Virtual Desktops
- [ ] Can switch between Virtual Desktops
- [ ] Can move windows to Virtual Desktops
- [ ] Workspace manager initializes correctly
- [ ] Can create and delete workspaces
- [ ] Can switch between workspaces
- [ ] Windows show/hide correctly on workspace switch
- [ ] Per-monitor workspaces work independently
- [ ] DPI scaling works across monitors
- [ ] Workspace state saves to disk
- [ ] Workspace state loads on startup
- [ ] Auto-save runs without issues

### Testing
- [ ] All unit tests pass: `cargo test --workspace`
- [ ] Virtual Desktop tests pass (on compatible systems)
- [ ] Workspace management tests pass
- [ ] Persistence tests pass
- [ ] No test failures or panics
- [ ] Integration tests pass

### Integration
- [ ] Window manager uses workspace system
- [ ] Windows are tracked per-workspace
- [ ] Layout system works with workspaces
- [ ] Focus management respects workspaces
- [ ] Commands work with workspaces
- [ ] IPC reports workspace state correctly

### Documentation
- [ ] All new public APIs have doc comments
- [ ] `cargo doc --no-deps` builds successfully
- [ ] README updated with Phase 3 features
- [ ] Examples in documentation work
- [ ] Workspace system is documented

### Manual Validation
- [ ] Create 10 workspaces successfully
- [ ] Switch between workspaces smoothly
- [ ] Open windows and distribute across workspaces
- [ ] Windows hide/show correctly on switch
- [ ] Move windows between workspaces
- [ ] Per-monitor workspaces work (multi-monitor setup)
- [ ] State persists across application restart
- [ ] Virtual Desktop integration works (if enabled)
- [ ] Application runs stable for 15+ minutes with workspace switching
- [ ] CPU usage remains reasonable
- [ ] Memory usage is stable

---

## Deliverables for Phase 3

At the end of Phase 3, you should have:

1. **Virtual Desktop Integration:**
   - COM interface definitions for Virtual Desktop API
   - Functions to enumerate, create, delete, and switch Virtual Desktops
   - Window movement between Virtual Desktops
   - Fallback for systems without Virtual Desktop support

2. **Comprehensive Workspace System:**
   - Workspace manager with full lifecycle management
   - Create, delete, rename workspaces
   - Switch between workspaces with window show/hide
   - Window-to-workspace mapping and movement
   - Support for 10+ workspaces per monitor

3. **Per-Monitor Workspaces:**
   - Independent workspaces for each monitor
   - Monitor-aware workspace switching
   - DPI-aware workspace geometries
   - Workspace redistribution on monitor changes

4. **Workspace Persistence:**
   - Save workspace state to JSON
   - Load workspace state on startup
   - Backup and recovery for corrupted state
   - Auto-save with configurable interval
   - Version tracking for state files

5. **Integration with Existing Systems:**
   - Window manager integrated with workspaces
   - Layout system works per-workspace
   - Focus management respects workspaces
   - Commands support workspace operations

6. **Quality Assurance:**
   - Comprehensive unit tests
   - Integration tests
   - Manual validation procedures
   - Performance benchmarks
   - Documentation complete

---

## Success Criteria Summary

Phase 3 is complete when:

1. ✅ **Virtual Desktop integration works:**
   - Can enumerate, create, switch, and move windows
   - Graceful degradation without Virtual Desktop support
   - Stable integration with Windows 10/11

2. ✅ **Workspace system is fully functional:**
   - Create, delete, rename, switch workspaces
   - Windows are properly managed per-workspace
   - Switching is smooth and responsive
   - 10+ workspaces work without issues

3. ✅ **Per-monitor support is solid:**
   - Independent workspaces per monitor
   - DPI awareness across monitors
   - Monitor changes handled gracefully
   - Multi-monitor setups work correctly

4. ✅ **Persistence is reliable:**
   - State saves and loads correctly
   - Corrupted files are recovered
   - Auto-save doesn't impact performance
   - State survives application restarts

5. ✅ **Integration is complete:**
   - All existing features work with workspaces
   - No regressions from Phase 1-2
   - Commands support workspace operations
   - IPC exposes workspace information

6. ✅ **Quality standards met:**
   - All tests passing
   - No clippy warnings
   - Stable operation
   - Good performance (<200ms workspace switch)
   - Memory usage is reasonable

---

## Next Steps

After completing Phase 3, proceed to **Phase 4: Configuration & Rules** (Weeks 13-16), which will implement:

- TOML configuration parsing with hot-reload
- Window rules engine (process/title/class matching)
- Workspace assignment rules
- Keybinding configuration
- Rule-based window management
- Configuration validation and error handling

See DETAILED_ROADMAP.md for Phase 4 specifications.

---

## Troubleshooting

### Common Issues

**Issue: Virtual Desktop COM interfaces fail to initialize**
- Solution: Check Windows version compatibility (requires Win10 1803+)
- Verify COM initialization with COINIT_APARTMENTTHREADED
- Check that IIDs are correct for your Windows build
- Try running as administrator if permission issues

**Issue: Workspace switching is slow**
- Solution: Profile window show/hide operations
- Check for excessive geometry recalculations
- Optimize window enumeration
- Consider caching window information

**Issue: Windows don't hide/show correctly**
- Solution: Verify ShowWindow calls are working
- Check window styles and flags
- Ensure windows are being tracked correctly
- Test with different window types

**Issue: State file gets corrupted**
- Solution: Check disk space and permissions
- Verify JSON serialization
- Use backup/recovery mechanism
- Add validation to state loading

**Issue: Per-monitor workspaces don't work**
- Solution: Verify monitor enumeration
- Check workspace-to-monitor assignments
- Ensure DPI scaling is applied
- Test monitor hotplug scenarios

**Issue: Memory leaks with workspace switching**
- Solution: Profile memory usage over time
- Check for window handle leaks
- Verify tree nodes are properly dropped
- Use leak detection tools

---

## Notes for Autonomous Agents

When executing this task list:

1. **Follow order strictly**: Tasks build on each other within weeks
2. **Validate each step**: Run acceptance criteria after each task
3. **Test incrementally**: Run tests after each significant change
4. **Handle COM carefully**: COM interfaces require precise memory management
5. **Test on real hardware**: Multi-monitor and DPI features need real testing
6. **Check Windows versions**: Virtual Desktop APIs vary by Windows version
7. **Monitor performance**: Workspace switching should be fast (<200ms)
8. **Handle errors gracefully**: Workspaces are critical, don't crash
9. **Document extensively**: Workspace system is complex, needs good docs
10. **Reference phases 1-2**: Build on existing foundation

---

**End of Phase 3 Task Document**

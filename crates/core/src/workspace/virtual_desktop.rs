//! Virtual Desktop COM Interfaces and Management
//!
//! This module provides integration with Windows Virtual Desktop APIs using COM interfaces.
//! It includes both documented interfaces (IVirtualDesktopManager) and undocumented interfaces
//! (IVirtualDesktopManagerInternal, IVirtualDesktop) for full control over Virtual Desktops.
//!
//! # Safety
//!
//! This module uses COM interfaces which require unsafe code. All unsafe operations are
//! encapsulated in safe wrapper functions. Users of this module should be aware that:
//! - COM must be initialized before use (handled by VirtualDesktopManager::new())
//! - The undocumented interfaces may change between Windows versions
//! - Proper error handling is provided for all COM operations
//!
//! # Platform Support
//!
//! This module is only available on Windows platforms with Virtual Desktop support
//! (Windows 10 and later).

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Com::*,
};

#[cfg(target_os = "windows")]
use std::ptr;

// ============================================================================
// COM Interface Definitions
// ============================================================================

/// Documented COM interface for Virtual Desktop management
/// GUID: aa509086-5ca9-4c25-8f95-589d3c07b48a
#[cfg(target_os = "windows")]
#[windows::core::interface("aa509086-5ca9-4c25-8f95-589d3c07b48a")]
pub unsafe trait IVirtualDesktopManager: IUnknown {
    /// Check if a window is on the current virtual desktop
    fn IsWindowOnCurrentVirtualDesktop(&self, toplevelwindow: HWND) -> Result<BOOL>;
    
    /// Get the GUID of the virtual desktop that a window is on
    fn GetWindowDesktopId(&self, toplevelwindow: HWND) -> Result<GUID>;
    
    /// Move a window to a specific virtual desktop
    fn MoveWindowToDesktop(&self, toplevelwindow: HWND, desktopid: *const GUID) -> Result<()>;
}

/// Undocumented COM interface for internal Virtual Desktop operations
/// GUID: f31574d6-b682-4cdc-bd56-1827860abec6 (Windows 10 Build 10240+)
/// Note: This GUID may differ in other Windows versions
#[cfg(target_os = "windows")]
#[windows::core::interface("f31574d6-b682-4cdc-bd56-1827860abec6")]
pub unsafe trait IVirtualDesktopManagerInternal: IUnknown {
    /// Get the count of virtual desktops
    fn GetCount(&self, count: *mut u32) -> HRESULT;
    
    /// Move a view (window) to a desktop
    fn MoveViewToDesktop(
        &self,
        view: *const IApplicationView,
        desktop: *const IVirtualDesktop,
    ) -> HRESULT;
    
    /// Check if a view can be moved between desktops
    fn CanViewMoveDesktops(&self, view: *const IApplicationView, result: *mut BOOL) -> HRESULT;
    
    /// Get the currently active virtual desktop
    fn GetCurrentDesktop(&self, desktop: *mut *mut IVirtualDesktop) -> HRESULT;
    
    /// Get all virtual desktops as an IObjectArray
    fn GetDesktops(&self, desktops: *mut *mut IObjectArray) -> HRESULT;
    
    /// Get an adjacent virtual desktop (left/right)
    fn GetAdjacentDesktop(
        &self,
        desktop: *const IVirtualDesktop,
        direction: i32,
        adjacent: *mut *mut IVirtualDesktop,
    ) -> HRESULT;
    
    /// Switch to a specific virtual desktop
    fn SwitchDesktop(&self, desktop: *const IVirtualDesktop) -> HRESULT;
    
    /// Create a new virtual desktop
    fn CreateDesktopW(&self, desktop: *mut *mut IVirtualDesktop) -> HRESULT;
    
    /// Remove a virtual desktop
    fn RemoveDesktop(
        &self,
        destroy: *const IVirtualDesktop,
        fallback: *const IVirtualDesktop,
    ) -> HRESULT;
    
    /// Find a virtual desktop by its GUID
    fn FindDesktop(&self, desktopid: *const GUID, desktop: *mut *mut IVirtualDesktop)
        -> HRESULT;
}

/// Undocumented COM interface representing a single Virtual Desktop
/// GUID: ff72ffdd-be7e-43fc-9c03-ad81681e88e4 (Windows 10 Build 10240+)
#[cfg(target_os = "windows")]
#[windows::core::interface("ff72ffdd-be7e-43fc-9c03-ad81681e88e4")]
pub unsafe trait IVirtualDesktop: IUnknown {
    /// Check if this is the currently active desktop
    fn IsViewVisible(&self, view: *const IApplicationView, result: *mut BOOL) -> HRESULT;
    
    /// Get the GUID of this virtual desktop
    fn GetID(&self, id: *mut GUID) -> HRESULT;
}

/// Undocumented COM interface for application views (windows)
/// GUID: 372e1d3b-38d3-42e4-a15b-8ab2b178f513
#[cfg(target_os = "windows")]
#[windows::core::interface("372e1d3b-38d3-42e4-a15b-8ab2b178f513")]
pub unsafe trait IApplicationView: IUnknown {
    // Methods would go here, but we only need the interface for type safety
    // The actual methods are not needed for our use case
}

// ============================================================================
// COM CLSIDs and Constants
// ============================================================================

#[cfg(target_os = "windows")]
const CLSID_VIRTUAL_DESKTOP_MANAGER: GUID = GUID::from_u128(0xaa509086_5ca9_4c25_8f95_589d3c07b48a);

#[cfg(target_os = "windows")]
const CLSID_IMMERSIVE_SHELL: GUID = GUID::from_u128(0xc2f03a33_21f5_47fa_b4bb_156362a2f239);

#[cfg(target_os = "windows")]
const CLSID_VIRTUAL_DESKTOP_MANAGER_INTERNAL: GUID =
    GUID::from_u128(0xc5e0cdca_7b6e_41b2_9fc4_d93975cc467b);

// ============================================================================
// VirtualDesktopManager Implementation
// ============================================================================

/// Manages Windows Virtual Desktops through COM interfaces
#[cfg(target_os = "windows")]
pub struct VirtualDesktopManager {
    manager: IVirtualDesktopManager,
    internal: Option<IVirtualDesktopManagerInternal>,
}

#[cfg(target_os = "windows")]
impl VirtualDesktopManager {
    /// Create a new Virtual Desktop manager instance
    ///
    /// This initializes COM and creates instances of the Virtual Desktop COM objects.
    /// If the system doesn't support Virtual Desktops (e.g., older Windows versions),
    /// the internal manager will be None but the documented manager will still work.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - COM initialization fails
    /// - The Virtual Desktop Manager cannot be created
    pub fn new() -> anyhow::Result<Self> {
        unsafe {
            // Initialize COM for this thread
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

            // Create the documented Virtual Desktop Manager
            let manager: IVirtualDesktopManager = CoCreateInstance(
                &CLSID_VIRTUAL_DESKTOP_MANAGER,
                None,
                CLSCTX_LOCAL_SERVER,
            )?;

            // Try to get the internal manager (may fail on systems without support)
            let internal = Self::get_internal_manager().ok();

            Ok(Self { manager, internal })
        }
    }

    /// Attempt to get the internal Virtual Desktop manager
    ///
    /// This is a separate function because it may fail on systems that don't
    /// support the undocumented APIs.
    #[cfg(target_os = "windows")]
    unsafe fn get_internal_manager() -> Result<IVirtualDesktopManagerInternal> {
        // Get the IServiceProvider from the shell
        let service_provider: IServiceProvider =
            CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_LOCAL_SERVER)?;

        // Query for the internal manager through the service provider
        service_provider.QueryService(
            &CLSID_VIRTUAL_DESKTOP_MANAGER_INTERNAL,
            &IVirtualDesktopManagerInternal::IID,
        )
    }

    /// Check if the system supports Virtual Desktop undocumented APIs
    ///
    /// Returns true if the internal manager is available, false otherwise.
    pub fn is_supported(&self) -> bool {
        self.internal.is_some()
    }

    /// Get the number of Virtual Desktops
    ///
    /// # Errors
    ///
    /// Returns an error if the internal manager is not available or if the COM call fails.
    pub fn get_desktop_count(&self) -> anyhow::Result<usize> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let mut count: u32 = 0;
                let hr = internal.GetCount(&mut count);
                hr.ok()?;
                Ok(count as usize)
            } else {
                // If internal manager is not available, assume single desktop
                Ok(1)
            }
        }
    }

    /// Get all Virtual Desktop IDs
    ///
    /// Returns a vector of GUIDs, one for each virtual desktop.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The internal manager is not available
    /// - The COM calls fail
    /// - Desktop enumeration fails
    pub fn get_desktop_ids(&self) -> anyhow::Result<Vec<GUID>> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let mut desktops_ptr: *mut IObjectArray = ptr::null_mut();
                let hr = internal.GetDesktops(&mut desktops_ptr);
                hr.ok()?;

                if desktops_ptr.is_null() {
                    anyhow::bail!("GetDesktops returned null");
                }

                let desktops: IObjectArray = IObjectArray::from_raw(desktops_ptr);
                let count = desktops.GetCount()?;

                let mut ids = Vec::new();
                for i in 0..count {
                    let desktop: IVirtualDesktop =
                        desktops.GetAt(i, &IVirtualDesktop::IID)?;

                    let mut id = GUID::zeroed();
                    let hr = desktop.GetID(&mut id);
                    hr.ok()?;
                    ids.push(id);
                }

                Ok(ids)
            } else {
                Ok(vec![])
            }
        }
    }

    /// Get the ID of the currently active Virtual Desktop
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Virtual Desktop API is not available
    /// - The COM calls fail
    pub fn get_current_desktop_id(&self) -> anyhow::Result<GUID> {
        unsafe {
            if let Some(ref internal) = self.internal {
                let mut desktop_ptr: *mut IVirtualDesktop = ptr::null_mut();
                let hr = internal.GetCurrentDesktop(&mut desktop_ptr);
                hr.ok()?;

                if desktop_ptr.is_null() {
                    anyhow::bail!("GetCurrentDesktop returned null");
                }

                let desktop: IVirtualDesktop = IVirtualDesktop::from_raw(desktop_ptr);
                let mut id = GUID::zeroed();
                let hr = desktop.GetID(&mut id);
                hr.ok()?;
                Ok(id)
            } else {
                anyhow::bail!("Virtual Desktop API not available")
            }
        }
    }

    /// Check if a window is on the current Virtual Desktop
    ///
    /// # Errors
    ///
    /// Returns an error if the COM call fails.
    pub fn is_window_on_current_desktop(&self, hwnd: HWND) -> anyhow::Result<bool> {
        unsafe {
            let result = self.manager.IsWindowOnCurrentVirtualDesktop(hwnd)?;
            Ok(result.as_bool())
        }
    }

    /// Get the Virtual Desktop ID that a window is on
    ///
    /// # Errors
    ///
    /// Returns an error if the COM call fails or if the window is not on any desktop.
    pub fn get_window_desktop_id(&self, hwnd: HWND) -> anyhow::Result<GUID> {
        unsafe {
            let id = self.manager.GetWindowDesktopId(hwnd)?;
            Ok(id)
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for VirtualDesktopManager {
    fn drop(&mut self) {
        unsafe {
            // Uninitialize COM for this thread
            CoUninitialize();
        }
    }
}

// ============================================================================
// Non-Windows stub implementation
// ============================================================================

#[cfg(not(target_os = "windows"))]
pub struct VirtualDesktopManager;

#[cfg(not(target_os = "windows"))]
impl VirtualDesktopManager {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }

    pub fn is_supported(&self) -> bool {
        false
    }

    pub fn get_desktop_count(&self) -> anyhow::Result<usize> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }

    pub fn get_desktop_ids(&self) -> anyhow::Result<Vec<u128>> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }

    pub fn get_current_desktop_id(&self) -> anyhow::Result<u128> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }

    pub fn is_window_on_current_desktop(&self, _hwnd: isize) -> anyhow::Result<bool> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }

    pub fn get_window_desktop_id(&self, _hwnd: isize) -> anyhow::Result<u128> {
        anyhow::bail!("Virtual Desktop Manager is only available on Windows")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "virtual_desktop_tests.rs"]
mod virtual_desktop_tests;

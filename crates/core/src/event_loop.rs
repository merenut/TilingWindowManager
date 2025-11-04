//! Windows Event Loop for monitoring and dispatching window events.
//!
//! This module provides a thread-safe event loop that monitors Windows events using
//! the Win32 API's `SetWinEventHook` function. It detects window creation, destruction,
//! focus changes, and other window state changes.
//!
//! # Platform Support
//!
//! This module is only available on Windows platforms. On other platforms, stub
//! implementations are provided for compilation compatibility.
//!
//! # Safety
//!
//! The event loop uses unsafe Win32 API calls, but provides a safe Rust interface.
//! All hooks are properly registered and unregistered to prevent memory/handle leaks.
//!
//! # Examples
//!
//! ```no_run
//! use tiling_wm_core::event_loop::EventLoop;
//!
//! let mut event_loop = EventLoop::new();
//! event_loop.start().unwrap();
//!
//! // Poll for events
//! for event in event_loop.poll_events() {
//!     println!("Event: {:?}", event);
//! }
//!
//! event_loop.stop().unwrap();
//! ```

// Windows-specific implementation
#[cfg(target_os = "windows")]
mod windows_impl {
    use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
    use windows::{
        Win32::Foundation::HWND,
        Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, PeekMessageW, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW,
            EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
            EVENT_SYSTEM_MOVESIZEEND, MSG, PM_REMOVE, WINEVENT_OUTOFCONTEXT,
        },
    };

    /// Represents different types of window events
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

    /// Global sender for event communication from the Win32 callback.
    /// We store a raw pointer to the sender so it can be accessed from the callback.
    ///
    /// # Safety
    ///
    /// This is safe because:
    /// 1. The pointer is only set when the event loop starts
    /// 2. The pointer is cleared when the event loop stops
    /// 3. The EventLoop owns the sender and keeps it alive while hooks are active
    /// 4. Only one EventLoop instance should be active at a time (enforced by application design)
    ///
    /// # Note
    ///
    /// This design assumes single-threaded event loop usage. If multiple EventLoop instances
    /// need to run concurrently, this should be refactored to use thread-local storage or
    /// a more sophisticated synchronization mechanism.
    static mut EVENT_SENDER_PTR: *const Sender<WindowEvent> = std::ptr::null();

    /// Win32 event hook callback function.
    /// This function is called by Windows whenever a registered event occurs.
    unsafe extern "system" fn win_event_proc(
        _hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        _id_object: i32,
        _id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        // Skip events without a valid window handle
        if hwnd.0 == 0 {
            return;
        }

        // Get the event sender
        let sender_ptr = EVENT_SENDER_PTR;
        if sender_ptr.is_null() {
            return;
        }

        // Safety: The pointer is valid as long as the EventLoop is alive,
        // and we only dereference it to send a message (no mutation of the Sender itself)
        let sender = &*sender_ptr;

        // Convert Win32 event to our WindowEvent enum
        let window_event = match event {
            EVENT_OBJECT_CREATE => WindowEvent::WindowCreated(hwnd),
            EVENT_OBJECT_DESTROY => WindowEvent::WindowDestroyed(hwnd),
            EVENT_OBJECT_SHOW => WindowEvent::WindowShown(hwnd),
            EVENT_OBJECT_HIDE => WindowEvent::WindowHidden(hwnd),
            EVENT_SYSTEM_MOVESIZEEND | EVENT_OBJECT_LOCATIONCHANGE => {
                WindowEvent::WindowMoved(hwnd)
            }
            EVENT_SYSTEM_MINIMIZESTART => WindowEvent::WindowMinimized(hwnd),
            EVENT_SYSTEM_MINIMIZEEND => WindowEvent::WindowRestored(hwnd),
            EVENT_SYSTEM_FOREGROUND => WindowEvent::WindowFocused(hwnd),
            _ => return, // Ignore unknown events
        };

        // Send the event through the channel
        let _ = sender.send(window_event);
    }

    /// Event loop for monitoring Windows events
    pub struct EventLoop {
        event_tx: Sender<WindowEvent>,
        event_rx: Receiver<WindowEvent>,
        hooks: Vec<HWINEVENTHOOK>,
        running: bool,
    }

    impl EventLoop {
        /// Create a new event loop
        pub fn new() -> Self {
            let (tx, rx) = channel();
            EventLoop {
                event_tx: tx,
                event_rx: rx,
                hooks: Vec::new(),
                running: false,
            }
        }

        /// Start the event loop and register Windows event hooks.
        pub fn start(&mut self) -> anyhow::Result<()> {
            if self.running {
                return Ok(());
            }

            // Store the sender pointer in the global static for the callback to use
            unsafe {
                EVENT_SENDER_PTR = &self.event_tx as *const Sender<WindowEvent>;
            }

            // Register event hooks for various window events
            let events = vec![
                (EVENT_OBJECT_CREATE, EVENT_OBJECT_CREATE),
                (EVENT_OBJECT_DESTROY, EVENT_OBJECT_DESTROY),
                (EVENT_OBJECT_SHOW, EVENT_OBJECT_SHOW),
                (EVENT_OBJECT_HIDE, EVENT_OBJECT_HIDE),
                (EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND),
                (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE),
                (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZESTART),
                (EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZEEND),
                (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
            ];

            for (event_min, event_max) in events {
                unsafe {
                    let hook = SetWinEventHook(
                        event_min,
                        event_max,
                        None,
                        Some(win_event_proc),
                        0,
                        0,
                        WINEVENT_OUTOFCONTEXT,
                    );

                    if hook.is_invalid() {
                        // Clean up any hooks that were already registered
                        for h in self.hooks.drain(..) {
                            let _ = UnhookWinEvent(h);
                        }

                        // Clear the global sender pointer
                        EVENT_SENDER_PTR = std::ptr::null();

                        return Err(anyhow::anyhow!(
                            "Failed to set event hook for event {}",
                            event_min
                        ));
                    }

                    self.hooks.push(hook);
                }
            }

            self.running = true;
            Ok(())
        }

        /// Stop the event loop and unregister all event hooks.
        pub fn stop(&mut self) -> anyhow::Result<()> {
            if !self.running {
                return Ok(());
            }

            // Unhook all registered event hooks
            for hook in self.hooks.drain(..) {
                unsafe {
                    let _ = UnhookWinEvent(hook);
                }
            }

            // Clear the global sender pointer
            unsafe {
                EVENT_SENDER_PTR = std::ptr::null();
            }

            self.running = false;
            Ok(())
        }

        /// Poll for pending events from the event queue.
        pub fn poll_events(&self) -> impl Iterator<Item = WindowEvent> + '_ {
            std::iter::from_fn(move || match self.event_rx.try_recv() {
                Ok(event) => Some(event),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => None,
            })
        }

        /// Process Windows messages from the message queue.
        pub fn process_messages(&self) -> anyhow::Result<()> {
            unsafe {
                let mut msg = MSG::default();
                // Use PeekMessage pattern for non-blocking behavior
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    DispatchMessageW(&msg);
                }
            }
            Ok(())
        }

        /// Check if the event loop is currently running.
        pub fn is_running(&self) -> bool {
            self.running
        }
    }

    impl Drop for EventLoop {
        fn drop(&mut self) {
            let _ = self.stop();
        }
    }

    impl Default for EventLoop {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Stub implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
mod stub_impl {
    use std::sync::mpsc::{channel, Receiver, TryRecvError};

    /// Stub WindowEvent for non-Windows platforms
    #[derive(Debug, Clone)]
    pub enum WindowEvent {
        MonitorChanged,
    }

    /// Stub EventLoop for non-Windows platforms
    pub struct EventLoop {
        event_rx: Receiver<WindowEvent>,
    }

    impl EventLoop {
        pub fn new() -> Self {
            let (_tx, rx) = channel();
            EventLoop { event_rx: rx }
        }

        pub fn start(&mut self) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("EventLoop is only supported on Windows"))
        }

        pub fn stop(&mut self) -> anyhow::Result<()> {
            Ok(())
        }

        pub fn poll_events(&self) -> impl Iterator<Item = WindowEvent> + '_ {
            std::iter::from_fn(move || match self.event_rx.try_recv() {
                Ok(event) => Some(event),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => None,
            })
        }

        pub fn process_messages(&self) -> anyhow::Result<()> {
            Ok(())
        }

        pub fn is_running(&self) -> bool {
            false
        }
    }

    impl Default for EventLoop {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Re-export the appropriate implementation
#[cfg(target_os = "windows")]
pub use windows_impl::*;

#[cfg(not(target_os = "windows"))]
pub use stub_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_creation() {
        let event_loop = EventLoop::new();
        assert!(!event_loop.is_running());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_event_loop_start_stop() {
        let mut event_loop = EventLoop::new();
        assert!(!event_loop.is_running());

        let result = event_loop.start();
        assert!(
            result.is_ok(),
            "Failed to start event loop: {:?}",
            result.err()
        );
        assert!(event_loop.is_running());

        let result = event_loop.stop();
        assert!(
            result.is_ok(),
            "Failed to stop event loop: {:?}",
            result.err()
        );
        assert!(!event_loop.is_running());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_event_loop_double_start() {
        let mut event_loop = EventLoop::new();

        event_loop.start().unwrap();
        assert!(event_loop.is_running());

        // Starting again should be a no-op
        event_loop.start().unwrap();
        assert!(event_loop.is_running());

        event_loop.stop().unwrap();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_event_loop_double_stop() {
        let mut event_loop = EventLoop::new();
        event_loop.start().unwrap();

        event_loop.stop().unwrap();
        assert!(!event_loop.is_running());

        // Stopping again should be a no-op
        event_loop.stop().unwrap();
        assert!(!event_loop.is_running());
    }

    #[test]
    fn test_poll_events_empty() {
        let event_loop = EventLoop::new();
        let events: Vec<_> = event_loop.poll_events().collect();
        assert_eq!(events.len(), 0);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_event_loop_drop_cleanup() {
        let mut event_loop = EventLoop::new();
        event_loop.start().unwrap();
        assert!(event_loop.is_running());

        // Drop should automatically clean up hooks
        drop(event_loop);

        // If we reach here without crashing, cleanup worked
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_process_messages() {
        let event_loop = EventLoop::new();
        let result = event_loop.process_messages();
        assert!(result.is_ok());
    }

    // Manual test: This test requires interaction and should be run manually
    #[test]
    #[ignore]
    #[cfg(target_os = "windows")]
    fn test_window_events_detection() {
        let mut event_loop = EventLoop::new();
        event_loop.start().expect("Failed to start event loop");

        println!("Event loop started. Please open/close/focus windows for 10 seconds...");

        let start_time = std::time::Instant::now();
        let mut event_count = 0;

        while start_time.elapsed().as_secs() < 10 {
            event_loop.process_messages().unwrap();

            for event in event_loop.poll_events() {
                println!("Detected event: {:?}", event);
                event_count += 1;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        println!("Detected {} events total", event_count);
        assert!(event_count > 0, "Should have detected at least one event");

        event_loop.stop().unwrap();
    }
}

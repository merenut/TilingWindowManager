//! Layout algorithms for tiling window managers.
//!
//! This module provides different layout algorithms that determine how windows
//! are positioned and sized on the screen.

pub mod dwindle;

pub use dwindle::DwindleLayout;

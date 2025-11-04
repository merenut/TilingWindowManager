//! Layout algorithms for tiling window managers.
//!
//! This module provides different layout algorithms that determine how windows
//! are positioned and sized on the screen.

pub mod dwindle;
pub mod master;

pub use dwindle::DwindleLayout;
pub use master::MasterLayout;

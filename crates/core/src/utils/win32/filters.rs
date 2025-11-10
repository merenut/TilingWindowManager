//! Window filtering functions.
//!
//! This module provides utility functions for filtering collections of windows
//! based on various criteria.

use super::WindowHandle;

/// Filter windows by process ID.
///
/// # Arguments
///
/// * `windows` - The vector of windows to filter
/// * `process_id` - The process ID to match
///
/// # Returns
///
/// A vector of windows belonging to the specified process.
pub fn filter_by_process_id(windows: &[WindowHandle], process_id: u32) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| w.get_process_id() == process_id)
        .copied()
        .collect()
}

/// Filter windows by class name.
///
/// # Arguments
///
/// * `windows` - The vector of windows to filter
/// * `class_name` - The class name to match
///
/// # Returns
///
/// A vector of windows with the specified class name.
pub fn filter_by_class_name(windows: &[WindowHandle], class_name: &str) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| w.get_class_name().map(|c| c == class_name).unwrap_or(false))
        .copied()
        .collect()
}

/// Filter windows by title (exact match).
///
/// # Arguments
///
/// * `windows` - The vector of windows to filter
/// * `title` - The title to match
///
/// # Returns
///
/// A vector of windows with the specified title.
pub fn filter_by_title(windows: &[WindowHandle], title: &str) -> Vec<WindowHandle> {
    windows
        .iter()
        .filter(|w| w.get_title().map(|t| t == title).unwrap_or(false))
        .copied()
        .collect()
}

/// Filter windows by title pattern (case-insensitive substring match).
///
/// # Arguments
///
/// * `windows` - The vector of windows to filter
/// * `pattern` - The pattern to search for in titles
///
/// # Returns
///
/// A vector of windows whose titles contain the specified pattern.
pub fn filter_by_title_pattern(windows: &[WindowHandle], pattern: &str) -> Vec<WindowHandle> {
    let pattern_lower = pattern.to_lowercase();
    windows
        .iter()
        .filter(|w| {
            w.get_title()
                .map(|t| t.to_lowercase().contains(&pattern_lower))
                .unwrap_or(false)
        })
        .copied()
        .collect()
}

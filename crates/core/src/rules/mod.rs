//! Window rules engine module
//! 
//! This module provides a comprehensive rules engine for automatic window management:
//! - Pattern matching based on process name, window title, and class name
//! - Regex-based matching for flexible rules
//! - Multiple rule matches per window with action aggregation
//! - Efficient compiled regex patterns
//! - Integration with window manager for automatic rule application

pub mod matcher;
pub mod executor;

pub use matcher::{RuleMatcher, CompiledRule, RuleMatch};
pub use executor::RuleExecutor;

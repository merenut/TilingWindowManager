//! Command parsing and execution.
//!
//! This module handles converting string commands to Command enums and executing them.

use anyhow::Result;
use tracing::{debug, warn};

use crate::commands::{Command, CommandExecutor};
use crate::window_manager::WindowManager;

/// Execute a command from a string representation.
///
/// This function parses a command string and optional arguments, then executes
/// the corresponding command through the CommandExecutor.
///
/// Note: Currently most commands don't use arguments. The `exec` command would
/// use arguments to launch applications, but that functionality is not yet
/// implemented in the Command enum. This will be added in a future phase.
pub fn execute_command_from_string(
    executor: &CommandExecutor,
    wm: &mut WindowManager,
    command_str: &str,
    args: &[String],
) -> Result<()> {
    // Log if arguments are provided (for future exec command support)
    if !args.is_empty() {
        debug!("Command '{}' called with args: {:?}", command_str, args);
    }
    
    // Parse command string to Command enum
    // 
    // Design Note: This uses a simple match statement for command parsing rather than
    // a more complex dynamic approach (e.g., FromStr trait, command registry) for
    // several reasons:
    // 1. Simplicity and clarity - easy to see all supported commands
    // 2. Compile-time checking - typos in command strings caught by clippy
    // 3. Performance - match is optimized by compiler, no runtime parsing overhead
    // 4. Type safety - direct mapping to Command enum variants
    //
    // To add new commands:
    // 1. Add the variant to Command enum in commands.rs
    // 2. Add a case to this match statement
    // 3. Document in KEYBINDINGS_GUIDE.md
    let command = match command_str {
        // Window commands
        "close" => Command::CloseActiveWindow,
        "toggle-floating" => Command::ToggleFloating,
        "toggle-fullscreen" => Command::ToggleFullscreen,
        "minimize" => Command::MinimizeActive,
        "restore" => Command::RestoreActive,
        
        // Focus commands
        "focus-left" => Command::FocusLeft,
        "focus-right" => Command::FocusRight,
        "focus-up" => Command::FocusUp,
        "focus-down" => Command::FocusDown,
        "focus-previous" => Command::FocusPrevious,
        "focus-next" => Command::FocusNext,
        
        // Move commands
        "move-left" => Command::MoveWindowLeft,
        "move-right" => Command::MoveWindowRight,
        "move-up" => Command::MoveWindowUp,
        "move-down" => Command::MoveWindowDown,
        "swap-master" => Command::SwapWithMaster,
        
        // Layout commands
        "layout-dwindle" => Command::SetLayoutDwindle,
        "layout-master" => Command::SetLayoutMaster,
        "increase-master" => Command::IncreaseMasterCount,
        "decrease-master" => Command::DecreaseMasterCount,
        "increase-master-factor" => Command::IncreaseMasterFactor,
        "decrease-master-factor" => Command::DecreaseMasterFactor,
        
        // Workspace commands
        "workspace-1" => Command::SwitchWorkspace(1),
        "workspace-2" => Command::SwitchWorkspace(2),
        "workspace-3" => Command::SwitchWorkspace(3),
        "workspace-4" => Command::SwitchWorkspace(4),
        "workspace-5" => Command::SwitchWorkspace(5),
        "workspace-6" => Command::SwitchWorkspace(6),
        "workspace-7" => Command::SwitchWorkspace(7),
        "workspace-8" => Command::SwitchWorkspace(8),
        "workspace-9" => Command::SwitchWorkspace(9),
        "workspace-10" => Command::SwitchWorkspace(10),
        
        "move-to-workspace-1" => Command::MoveToWorkspace(1),
        "move-to-workspace-2" => Command::MoveToWorkspace(2),
        "move-to-workspace-3" => Command::MoveToWorkspace(3),
        "move-to-workspace-4" => Command::MoveToWorkspace(4),
        "move-to-workspace-5" => Command::MoveToWorkspace(5),
        
        // System commands
        "reload-config" => Command::Reload,
        "exit" | "quit" => Command::Quit,
        "show-command-palette" => Command::ShowCommandPalette,
        
        // Unknown command
        _ => {
            warn!("Unknown command: {}", command_str);
            return Ok(());
        }
    };
    
    debug!("Parsed command: {:?}", command);
    
    // Execute the command
    executor.execute(command, wm)?;
    
    Ok(())
}

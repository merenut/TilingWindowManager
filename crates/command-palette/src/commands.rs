use serde::{Deserialize, Serialize};
use sublime_fuzzy::best_match;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandEntry {
    pub display_name: String,
    pub command: String,
    pub args: Vec<String>,
}

pub struct CommandCatalog {
    commands: Vec<CommandEntry>,
}

impl CommandCatalog {
    pub fn new() -> Self {
        let mut commands = Vec::new();

        // Window commands
        commands.push(CommandEntry {
            display_name: "Close Active Window".to_string(),
            command: "close".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Toggle Floating".to_string(),
            command: "toggle_floating".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Toggle Fullscreen".to_string(),
            command: "toggle_fullscreen".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Minimize Active Window".to_string(),
            command: "minimize".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Restore Active Window".to_string(),
            command: "restore".to_string(),
            args: vec![],
        });

        // Focus commands
        commands.push(CommandEntry {
            display_name: "Focus Left".to_string(),
            command: "focus_left".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Focus Right".to_string(),
            command: "focus_right".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Focus Up".to_string(),
            command: "focus_up".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Focus Down".to_string(),
            command: "focus_down".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Focus Previous".to_string(),
            command: "focus_previous".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Focus Next".to_string(),
            command: "focus_next".to_string(),
            args: vec![],
        });

        // Move commands
        commands.push(CommandEntry {
            display_name: "Move Window Left".to_string(),
            command: "move_left".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Move Window Right".to_string(),
            command: "move_right".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Move Window Up".to_string(),
            command: "move_up".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Move Window Down".to_string(),
            command: "move_down".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Swap with Master".to_string(),
            command: "swap_master".to_string(),
            args: vec![],
        });

        // Layout commands
        commands.push(CommandEntry {
            display_name: "Set Layout: Dwindle".to_string(),
            command: "layout_dwindle".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Set Layout: Master".to_string(),
            command: "layout_master".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Increase Master Count".to_string(),
            command: "increase_master_count".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Decrease Master Count".to_string(),
            command: "decrease_master_count".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Increase Master Factor".to_string(),
            command: "increase_master_factor".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Decrease Master Factor".to_string(),
            command: "decrease_master_factor".to_string(),
            args: vec![],
        });

        // Workspace commands (1-9)
        for i in 1..=9 {
            commands.push(CommandEntry {
                display_name: format!("Switch to Workspace {}", i),
                command: "switch_workspace".to_string(),
                args: vec![i.to_string()],
            });
            commands.push(CommandEntry {
                display_name: format!("Move to Workspace {}", i),
                command: "move_to_workspace".to_string(),
                args: vec![i.to_string()],
            });
            commands.push(CommandEntry {
                display_name: format!("Move to Workspace {} and Follow", i),
                command: "move_to_workspace_follow".to_string(),
                args: vec![i.to_string()],
            });
        }

        // System commands
        commands.push(CommandEntry {
            display_name: "Reload Configuration".to_string(),
            command: "reload".to_string(),
            args: vec![],
        });
        commands.push(CommandEntry {
            display_name: "Quit Tenraku".to_string(),
            command: "quit".to_string(),
            args: vec![],
        });

        Self { commands }
    }

    pub fn search(&self, query: &str) -> Vec<(CommandEntry, i64)> {
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for command in &self.commands {
            if let Some(match_result) = best_match(query, &command.display_name) {
                results.push((command.clone(), match_result.score() as i64));
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    pub fn all_commands(&self) -> &[CommandEntry] {
        &self.commands
    }
}

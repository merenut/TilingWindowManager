use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use sublime_fuzzy::best_match;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableEntry {
    pub name: String,
    pub full_path: PathBuf,
    pub show_path: bool,
}

#[derive(Debug, Clone)]
pub struct Scanner {
    executables: Arc<RwLock<HashMap<String, Vec<PathBuf>>>>,
    is_scanning: Arc<RwLock<bool>>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            executables: Arc::new(RwLock::new(HashMap::new())),
            is_scanning: Arc::new(RwLock::new(false)),
        }
    }

    pub fn is_scanning(&self) -> bool {
        *self.is_scanning.read().unwrap()
    }

    pub fn start_scan(&self) {
        let executables = Arc::clone(&self.executables);
        let is_scanning = Arc::clone(&self.is_scanning);

        tokio::spawn(async move {
            *is_scanning.write().unwrap() = true;

            let mut exe_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

            // Get PATH environment variable
            if let Ok(path_var) = std::env::var("PATH") {
                for path_entry in path_var.split(';') {
                    if path_entry.is_empty() {
                        continue;
                    }

                    let path = PathBuf::from(path_entry);
                    if !path.exists() || !path.is_dir() {
                        continue;
                    }

                    // Read directory entries
                    if let Ok(entries) = std::fs::read_dir(&path) {
                        for entry in entries.flatten() {
                            if let Ok(file_type) = entry.file_type() {
                                if !file_type.is_file() {
                                    continue;
                                }
                            }

                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                if ext.eq_ignore_ascii_case("exe") {
                                    if let Some(name) = path.file_stem() {
                                        let name_str = name.to_string_lossy().to_lowercase();
                                        exe_map
                                            .entry(name_str)
                                            .or_insert_with(Vec::new)
                                            .push(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            *executables.write().unwrap() = exe_map;
            *is_scanning.write().unwrap() = false;
        });
    }

    pub fn search(&self, query: &str) -> Vec<(ExecutableEntry, i64)> {
        if query.is_empty() {
            return Vec::new();
        }

        let executables = self.executables.read().unwrap();
        let mut results = Vec::new();

        for (name, paths) in executables.iter() {
            if let Some(match_result) = best_match(query, name) {
                let show_path = paths.len() > 1;
                
                // Add all paths if there are duplicates, otherwise just the first
                for path in paths.iter() {
                    results.push((
                        ExecutableEntry {
                            name: name.clone(),
                            full_path: path.clone(),
                            show_path,
                        },
                        match_result.score() as i64,
                    ));
                }
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }

    pub fn get_executable_count(&self) -> usize {
        self.executables.read().unwrap().len()
    }
}

use std::{env, fs};
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::logger;

fn find_binary_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    if !dir.exists() || !dir.is_dir() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Check if this is the binary we're looking for
            if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                    return Some(path);
                }
            }
            
            // Recursively search subdirectories, but skip node_modules within node_modules
            if path.is_dir() && path.file_name().and_then(|n| n.to_str()) != Some("node_modules") {
                if let Some(found) = find_binary_recursive(&path, name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

pub fn run(task_name: &str) {
    let debug_mode = env::args().any(|arg| arg == "--debug");
    let package_file = Path::new("package.json");

    if !package_file.exists() {
        logger::error("No package.json file found in the current directory.");
        return;
    }

    let content = fs::read_to_string(package_file).expect("Failed to read package.json");
    let json: Value = serde_json::from_str(&content).expect("Failed to parse package.json");

    let package_name = json
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");
    let package_version = json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");

    let scripts = json.get("scripts").and_then(|s| s.as_object());

    match scripts {
        None => {
            logger::error("No 'scripts' field found in package.json");
            return;
        }
        Some(scripts_obj) => {
            if let Some(script) = scripts_obj.get(task_name) {
                if let Some(command) = script.as_str() {
                    println!("SuperNPM v{}\n", env!("CARGO_PKG_VERSION"));
                    println!("  > {}@{} {}", package_name, package_version, task_name);
                    println!("  > {}\n", command);

                    let current_dir = env::current_dir().expect("Failed to get current directory");
                    let node_modules = current_dir.join("node_modules");

                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if parts.is_empty() {
                        logger::error("Empty command");
                        return;
                    }

                    let binary_name = parts[0];

                    let mut possible_paths = vec![
                        node_modules.join(".bin").join(binary_name),
                        node_modules
                            .join(".bin")
                            .join(format!("{}.cmd", binary_name)),
                        node_modules.join(binary_name).join("bin").join(binary_name),
                        node_modules
                            .join(binary_name)
                            .join("dist")
                            .join(binary_name),
                    ];

                    if binary_name.starts_with('@') {
                        if let Some(idx) = binary_name.find('/') {
                            let (scope, pkg) = binary_name.split_at(idx);
                            let pkg = &pkg[1..];
                            possible_paths
                                .push(node_modules.join(scope).join(pkg).join("bin").join(pkg));
                            possible_paths
                                .push(node_modules.join(scope).join(pkg).join("dist").join(pkg));
                        }
                    }

                    if debug_mode {
                        logger::info(&format!("Looking for binary '{}' in:", binary_name));
                        for path in &possible_paths {
                            logger::info(&format!("  - {}", path.display()));
                        }
                    }

                    let binary_path = match possible_paths.iter().find(|p| p.exists()) {
                        Some(path) => path.to_path_buf(),
                        None => {
                            // Try recursive search as a fallback
                            if let Some(found_path) = find_binary_recursive(&node_modules, binary_name) {
                                if debug_mode {
                                    logger::info(&format!("Found binary through recursive search at: {}", found_path.display()));
                                }
                                found_path
                            } else {
                                logger::error(&format!(
                                    "Binary '{}' not found. Please make sure the package is installed.",
                                    binary_name
                                ));
                                return;
                            }
                        }
                    };

                    if debug_mode {
                        logger::info(&format!("Using binary at: {}", binary_path.display()));
                    }

                    match std::process::Command::new(&binary_path)
                        .args(&parts[1..])
                        .current_dir(current_dir)
                        .status()
                    {
                        Ok(status) => {
                            if !status.success() {
                                logger::error(&format!(
                                    "Script '{}' failed with exit code: {}",
                                    task_name,
                                    status.code().unwrap_or(-1)
                                ));
                            }
                        }
                        Err(e) => logger::error(&format!("Failed to execute script: {}", e)),
                    }
                } else {
                    logger::error(&format!("Script '{}' is not a string", task_name));
                }
            } else {
                logger::error(&format!("Script '{}' not found in package.json", task_name));
            }
        }
    }
}

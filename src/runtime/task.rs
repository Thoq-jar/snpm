use std::{env, fs};
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::logger;
use crate::utils::utils::ASCII_ART;
use crate::io::logger::colorize;
use crate::utils::utils::get_framework_info;

fn find_binary_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    if !dir.exists() || !dir.is_dir() {
        return None;
    }

    #[cfg(windows)]
    let possible_extensions = vec![".cmd", ".exe", ".bat", ""];
    #[cfg(not(windows))]
    let possible_extensions = vec!["", ".sh"];
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    for ext in &possible_extensions {
                        let target_name = if ext.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}{}", name, ext)
                        };
                        if file_name == target_name {
                            return Some(path);
                        }
                    }
                }
            }
            
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
                    println!("{}", colorize("red", ASCII_ART));
                    println!("SuperNPM v{}\n", env!("CARGO_PKG_VERSION"));

                    println!("  > Starting snpm tasks...");
                    println!("  > {}@{} {}", package_name, package_version, task_name);
                    println!("  > {}", command);

                    if let Some((framework_name, color)) = get_framework_info(command) {
                        println!("  > Booting {}...", colorize(color, framework_name));
                    }
                    println!();
                    
                    let current_dir = env::current_dir().expect("Failed to get current directory");
                    let node_modules = current_dir.join("node_modules");

                    let parts: Vec<&str> = command.split_whitespace().collect();
                    if parts.is_empty() {
                        logger::error("Empty command");
                        return;
                    }

                    let binary_name = parts[0];

                    let mut possible_paths = vec![];
                    
                    #[cfg(windows)]
                    {
                        possible_paths.extend(vec![
                            node_modules.join(".bin").join(format!("{}.cmd", binary_name)),
                            node_modules.join(".bin").join(format!("{}.exe", binary_name)),
                            node_modules.join(".bin").join(binary_name),
                            node_modules.join(binary_name).join("bin").join(format!("{}.cmd", binary_name)),
                            node_modules.join(binary_name).join("bin").join(format!("{}.exe", binary_name)),
                            node_modules.join(binary_name).join("bin").join(binary_name),
                            node_modules.join(binary_name).join("dist").join(format!("{}.cmd", binary_name)),
                            node_modules.join(binary_name).join("dist").join(format!("{}.exe", binary_name)),
                            node_modules.join(binary_name).join("dist").join(binary_name),
                        ]);
                    }

                    #[cfg(not(windows))]
                    {
                        possible_paths.extend(vec![
                            node_modules.join(".bin").join(binary_name),
                            node_modules.join(binary_name).join("bin").join(binary_name),
                            node_modules.join(binary_name).join("dist").join(binary_name),
                        ]);
                    }

                    if binary_name.starts_with('@') {
                        if let Some(idx) = binary_name.find('/') {
                            let (scope, pkg) = binary_name.split_at(idx);
                            let pkg = &pkg[1..];
                            #[cfg(windows)]
                            {
                                possible_paths.push(node_modules.join(scope).join(pkg).join("bin").join(format!("{}.cmd", pkg)));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("bin").join(format!("{}.exe", pkg)));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("bin").join(pkg));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("dist").join(format!("{}.cmd", pkg)));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("dist").join(format!("{}.exe", pkg)));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("dist").join(pkg));
                            }
                            #[cfg(not(windows))]
                            {
                                possible_paths.push(node_modules.join(scope).join(pkg).join("bin").join(pkg));
                                possible_paths.push(node_modules.join(scope).join(pkg).join("dist").join(pkg));
                            }
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

                    let result = std::process::Command::new(&binary_path)
                        .args(&parts[1..])
                        .current_dir(&current_dir)
                        .status();

                    match result {
                        Ok(status) => {
                            if !status.success() {
                                logger::error(&format!(
                                    "Script '{}' failed with exit code: {}",
                                    task_name,
                                    status.code().unwrap_or(-1)
                                ));
                            }
                        }
                        Err(e) => {
                            if e.raw_os_error() == Some(193) {
                                if debug_mode {
                                    logger::info("Direct execution failed, trying to run through node...");
                                }
                                
                                match std::process::Command::new("node")
                                    .arg(&binary_path)
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
                                    Err(e) => logger::error(&format!("Failed to execute script through node: {}", e)),
                                }
                            } else {
                                logger::error(&format!("Failed to execute script: {}", e));
                            }
                        }
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

pub fn run_npx(package_args: &str) {
    let debug_mode = env::args().any(|arg| arg == "--debug");
    println!("SuperNPM v{}\n", env!("CARGO_PKG_VERSION"));
    println!("  > npx {}\n", package_args);

    let current_dir = env::current_dir().expect("Failed to get current directory");
    
    #[cfg(windows)]
    let npx_command = {
        let node_path = std::process::Command::new("where")
            .arg("node")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.lines().next().map(|s| s.to_string()));

        match node_path {
            Some(path) => {
                let node_dir = Path::new(&path).parent().unwrap_or(Path::new(""));
                node_dir.join("npx.cmd")
            }
            None => PathBuf::from("npx.cmd")
        }
    };

    #[cfg(not(windows))]
    let npx_command = PathBuf::from("npx");

    if debug_mode {
        logger::info(&format!("Using npx at: {}", npx_command.display()));
    }
    
    match std::process::Command::new(npx_command)
        .args(package_args.split_whitespace())
        .current_dir(current_dir)
        .status()
    {
        Ok(status) => {
            if !status.success() {
                logger::error(&format!(
                    "npx execution failed with exit code: {}",
                    status.code().unwrap_or(-1)
                ));
            }
        }
        Err(e) => logger::error(&format!("Failed to execute npx: {}", e)),
    }
}

pub fn run_create(create_args: &str) {
    let debug_mode = env::args().any(|arg| arg == "--debug");
    println!("SuperNPM v{}\n", env!("CARGO_PKG_VERSION"));
    println!("  > npm create {}\n", create_args);

    let current_dir = env::current_dir().expect("Failed to get current directory");
    
    // On Windows, we need to use the full path to npm.cmd
    #[cfg(windows)]
    let npm_command = {
        let node_path = std::process::Command::new("where")
            .arg("node")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.lines().next().map(|s| s.to_string()));

        match node_path {
            Some(path) => {
                let node_dir = Path::new(&path).parent().unwrap_or(Path::new(""));
                node_dir.join("npm.cmd")
            }
            None => PathBuf::from("npm.cmd") // Fallback to just npm.cmd
        }
    };

    #[cfg(not(windows))]
    let npm_command = PathBuf::from("npm");

    if debug_mode {
        logger::info(&format!("Using npm at: {}", npm_command.display()));
    }
    
    match std::process::Command::new(npm_command)
        .arg("create")
        .args(create_args.split_whitespace())
        .current_dir(current_dir)
        .status()
    {
        Ok(status) => {
            if !status.success() {
                logger::error(&format!(
                    "npm create failed with exit code: {}",
                    status.code().unwrap_or(-1)
                ));
            }
        }
        Err(e) => logger::error(&format!("Failed to execute npm create: {}", e)),
    }
}

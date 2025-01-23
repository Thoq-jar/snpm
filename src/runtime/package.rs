use std::{fs, thread};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};
use serde_json::Value;
use crate::{logger, io::net, utils::utils};
use crate::utils::lockfile;
use crate::io::logger::colorize;
use crate::utils::utils::ASCII_ART;

pub fn install(debug_mode: bool, force_mode: bool) {
    let package_file = Path::new("package.json");

    if !package_file.exists() {
        logger::error("No package.json file found in the current directory. Please create one.");
        return;
    }

    let content = fs::read_to_string(package_file).expect("Failed to read package.json");
    let json: Value = serde_json::from_str(&content).expect("Failed to parse package.json");

    let dependencies = json.get("dependencies").and_then(|d| d.as_object());
    let dev_dependencies = json.get("devDependencies").and_then(|d| d.as_object());

    if dependencies.is_none() && dev_dependencies.is_none() {
        logger::error("No dependencies or devDependencies found in package.json");
        return;
    }

    println!("{}", colorize("red", ASCII_ART));
    println!("SuperNPM v{}\n", env!("CARGO_PKG_VERSION"));
    logger::info("Installing packages...\n");

    let lockfile = lockfile::LockFile::load().unwrap_or_else(|e| {
        logger::error(&format!(
            "Failed to load lockfile: {}. Creating new one.",
            e
        ));
        lockfile::LockFile::new()
    });

    let cache_dir = utils::get_cache_directory();
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
    }

    let (tx, rx): (
        mpsc::Sender<(String, String, PathBuf, bool)>,
        mpsc::Receiver<(String, String, PathBuf, bool)>,
    ) = mpsc::channel();
    let rx = Arc::new(std::sync::Mutex::new(rx));
    let lockfile = Arc::new(std::sync::Mutex::new(lockfile));

    let node_modules = PathBuf::from("node_modules");
    if !node_modules.exists() {
        fs::create_dir_all(&node_modules).expect("Failed to create node_modules directory");
    }

    let num_threads = num_cpus::get();
    let mut handles = vec![];

    for _ in 0..num_threads {
        let rx = Arc::clone(&rx);
        let lockfile = Arc::clone(&lockfile);
        let handle = thread::spawn(move || loop {
            let package = match rx.lock().unwrap().recv() {
                Ok((name, version, cache_path, is_dev)) => {
                    (name, version, cache_path.to_path_buf(), is_dev)
                }
                Err(_) => break,
            };

            let package_type = if package.3 {
                "devDependency"
            } else {
                "dependency"
            };
            logger::info(&format!("Installing {} {}", package_type, package.0));

            let mut should_use_npm = lockfile
                .lock()
                .unwrap()
                .should_use_npm(&package.0, &package.1);

            if !should_use_npm {
                match net::download_and_cache_package(&package.2, debug_mode, force_mode) {
                    Ok((resolved_version, tarball_url)) => {
                        lockfile.lock().unwrap().add_package(
                            package.0.clone(),
                            package.1.clone(),
                            Some(tarball_url),
                            false,
                            resolved_version,
                        );

                        let target_dir = PathBuf::from("node_modules").join(&package.0);
                        if target_dir.exists() {
                            let _ = fs::remove_dir_all(&target_dir);
                        }

                        if let Err(e) = fs::create_dir_all(&target_dir) {
                            logger::error(&format!("Failed to create package directory: {}", e));
                            continue;
                        }

                        if let Err(e) = utils::copy_dir_contents(&package.2, &target_dir) {
                            logger::error(&format!("Failed to copy package contents: {}", e));
                        }
                    }
                    Err(e) => {
                        should_use_npm = true;
                        logger::error(&format!(
                            "Failed to download package: {}. Falling back to npm...",
                            e
                        ));
                    }
                }
            }

            if should_use_npm {
                lockfile.lock().unwrap().add_package(
                    package.0.clone(),
                    package.1.clone(),
                    None,
                    true,
                    package.1.clone(),
                );

                let package_spec = format!("{}@{}", package.0, package.1);
                let (shell, shell_arg) = if cfg!(windows) {
                    ("cmd", "/C")
                } else {
                    ("sh", "-c")
                };

                match std::process::Command::new(shell)
                    .arg(shell_arg)
                    .arg(format!(
                        "npm install {}{}",
                        if package.3 { "--save-dev " } else { "" },
                        package_spec
                    ))
                    .status()
                {
                    Ok(status) => {
                        if status.success() {
                            logger::info(&format!(
                                "Successfully installed {} using npm",
                                package_spec
                            ));
                        } else {
                            logger::error(&format!(
                                "npm install failed for {} with exit code: {}",
                                package_spec,
                                status.code().unwrap_or(-1)
                            ));
                        }
                    }
                    Err(e) => logger::error(&format!("Failed to execute npm install: {}", e)),
                }
            }

            if let Err(e) = lockfile.lock().unwrap().save() {
                logger::error(&format!("Failed to save lockfile: {}", e));
            }
        });
        handles.push(handle);
    }

    if let Some(deps) = dependencies {
        for (package, version) in deps {
            let version_str = version.as_str().unwrap().trim_matches('"');
            let cache_path = cache_dir.join(format!("{}_{}", package, version_str));
            if let Err(e) = tx.send((package.clone(), version_str.to_string(), cache_path, false)) {
                logger::error(&format!("Failed to queue package for installation: {}", e));
            }
        }
    }

    if let Some(dev_deps) = dev_dependencies {
        for (package, version) in dev_deps {
            let version_str = version.as_str().unwrap().trim_matches('"');
            let cache_path = cache_dir.join(format!("{}_{}", package, version_str));
            if let Err(e) = tx.send((package.clone(), version_str.to_string(), cache_path, true)) {
                logger::error(&format!("Failed to queue package for installation: {}", e));
            }
        }
    }

    drop(tx);

    for handle in handles {
        if let Err(e) = handle.join() {
            logger::error(&format!("A worker thread panicked: {:?}", e));
        }
    }

    println!();
    logger::info("All packages have been installed successfully.");
}

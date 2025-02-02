use std::fs;
use std::path::{Path, PathBuf};

pub const ASCII_ART: &str = r"
     _______..__   __. .______   .___  ___. 
    /       ||  \ |  | |   _  \  |   \/   | 
   |   (----`|   \|  | |  |_)  | |  \  /  | 
    \   \    |  . `  | |   ___/  |  |\/|  | 
.----)   |   |  |\   | |  |      |  |  |  | 
|_______/    |__| \__| | _|      |__|  |__| 
";

pub fn get_framework_info(command: &str) -> Option<(&'static str, &'static str)> {
    match command {
        cmd if cmd.starts_with("next") => Some(("Next.js", "nextjs_pink")),
        cmd if cmd.starts_with("nuxt") => Some(("Nuxt", "green")),
        cmd if cmd.starts_with("vite") => Some(("Vite", "yellow")),
        cmd if cmd.starts_with("vue") => Some(("Vue", "green")),
        cmd if cmd.starts_with("react-scripts") => Some(("Create React App", "cyan")),
        cmd if cmd.starts_with("astro") => Some(("Astro", "orange")),
        cmd if cmd.starts_with("svelte") => Some(("Svelte", "red")),
        _ => None,
    }
}

pub fn get_cache_directory() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\snpm_cache")
    } else {
        dirs::home_dir()
            .expect("Failed to determine home directory")
            .join(".snpm_cache")
    }
}

pub fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if !src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory does not exist: {}", src.display()),
        ));
    }

    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(path.file_name().unwrap());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_contents(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

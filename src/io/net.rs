use std::fs;
use std::io::copy;
use std::path::Path;
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde_json::Value;
use tar::Archive;
use super::logger;

pub fn download_and_cache_package(
    path: &Path,
    debug_mode: bool,
    force_mode: bool,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    let temp_tarball = path.with_extension("tgz");
    if temp_tarball.exists() {
        fs::remove_file(&temp_tarball)?;
    }
    let temp_extract_dir = path.with_extension("tmp");
    if temp_extract_dir.exists() {
        fs::remove_dir_all(&temp_extract_dir)?;
    }

    let package_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid package name")?;

    let parts: Vec<&str> = package_name.split('_').collect();
    if parts.len() != 2 {
        return Err("Invalid package name format".into());
    }
    let (name, version_req_str) = (parts[0], parts[1].trim_matches('"'));

    let registry_url = if name.starts_with("@angular/") {
        let package_name = name.replace('@', "").replace('/', "%2F");
        format!("https://registry.npmjs.org/{}", package_name)
    } else if name.starts_with('@') {
        let encoded_name = name.replace('@', "").replace('/', "%2F");
        format!("https://registry.npmjs.org/{}", encoded_name)
    } else {
        format!("https://registry.npmjs.org/{}", name)
    };

    logger::info(&format!("Fetching metadata from: {}", registry_url));

    let client = Client::new();
    let response = client.get(&registry_url).send()?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch package metadata for '{}'. Status: {}",
            name,
            response.status()
        )
            .into());
    }

    let metadata: Value = response.json()?;

    let versions_obj = metadata
        .get("versions")
        .and_then(|v| v.as_object())
        .ok_or_else(|| format!("No versions found in metadata for package '{}'", name))?;

    let mut available_versions = Vec::new();
    for version in versions_obj.keys() {
        if let Ok(v) = Version::parse(version) {
            available_versions.push(v);
        }
    }

    if available_versions.is_empty() {
        return Err(format!("No valid versions found for package '{}'", name).into());
    }

    available_versions.sort_by(|a, b| b.cmp(a));

    if debug_mode {
        logger::info(&format!("Available versions for {}:", name));
        for version in &available_versions {
            logger::info(&format!("  - {}", version));
        }
    }

    let clean_version_req = version_req_str
        .trim_start_matches('^')
        .trim_start_matches('~');
    let version_req = VersionReq::parse(&format!("^{}", clean_version_req))
        .or_else(|_| VersionReq::parse(&format!("={}", clean_version_req)))?;

    logger::info(&format!("Looking for version matching: {}", version_req));

    let selected_version = match available_versions.iter().find(|v| version_req.matches(v)) {
        Some(v) => {
            logger::info(&format!(
                "Selected version {} (requested {})",
                v, version_req_str
            ));
            v
        }
        None => {
            if !force_mode {
                logger::error(&format!(
                    "No version found matching {} for package '{}'. Use --force to install closest available version.",
                    version_req, name
                ));
                return Err(format!("Version not found for package '{}'", name).into());
            }

            let req_version = Version::parse(clean_version_req)?;
            let closest_version = available_versions
                .iter()
                .min_by_key(|v| (v.major as i64 - req_version.major as i64).abs())
                .ok_or_else(|| format!("No versions available for package '{}'", name))?;

            logger::warn(&format!(
                "Using closest available version {} for package {} (requested {})",
                closest_version, name, version_req
            ));
            closest_version
        }
    };

    let version_data = &metadata["versions"][&selected_version.to_string()];
    let tarball_url = version_data["dist"]["tarball"]
        .as_str()
        .ok_or_else(|| format!("Failed to get tarball URL for package '{}'", name))?;

    logger::info(&format!(
        "Downloading {} from {}",
        package_name, tarball_url
    ));

    let tarball_response = client.get(tarball_url).send()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut temp_file = fs::File::create(&temp_tarball)?;

    copy(&mut tarball_response.bytes()?.as_ref(), &mut temp_file)?;

    fs::create_dir_all(&temp_extract_dir)?;

    let tar_gz = fs::File::open(&temp_tarball)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive.unpack(&temp_extract_dir)?;

    let package_dir = fs::read_dir(&temp_extract_dir)?
        .filter_map(Result::ok)
        .find(|entry| entry.path().is_dir())
        .ok_or_else(|| format!("No package directory found in archive for '{}'", name))?;

    fs::create_dir_all(path)?;

    for entry in fs::read_dir(package_dir.path())? {
        let entry = entry?;
        let target_path = path.join(entry.file_name());
        if entry.path().is_dir() {
            if target_path.exists() {
                fs::remove_dir_all(&target_path)?;
            }
            fs::rename(entry.path(), target_path)?;
        } else {
            if target_path.exists() {
                fs::remove_file(&target_path)?;
            }
            fs::copy(entry.path(), target_path)?;
        }
    }

    fs::remove_dir_all(&temp_extract_dir)?;
    fs::remove_file(&temp_tarball)?;

    logger::info(&format!(
        "Package cached at {}",
        path.to_str().unwrap_or("unknown location")
    ));

    Ok((selected_version.to_string(), tarball_url.to_string()))
}

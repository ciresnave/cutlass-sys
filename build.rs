use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CUTLASS_DIR");
    println!("cargo:rerun-if-env-changed=CUTLASS_DOWNLOAD_RETRIES");
    println!("cargo:rerun-if-env-changed=CUTLASS_DOWNLOAD_TIMEOUT");

    // Use the crate version to determine which CUTLASS version to download
    // Only use first 3 components (MAJOR.MINOR.PATCH) to map to CUTLASS versions
    // This allows us to use build metadata (e.g., 4.2.0.1) without conflicting with CUTLASS versions
    let pkg_version = env!("CARGO_PKG_VERSION");
    let cutlass_version = format!("v{}", get_cutlass_version(pkg_version));

    println!(
        "cargo:warning=cutlass-sys {} maps to CUTLASS {}",
        pkg_version, cutlass_version
    );

    // 1. Check for user-provided CUTLASS_DIR (highest priority)
    if let Ok(custom_dir) = env::var("CUTLASS_DIR") {
        let cutlass_root = PathBuf::from(&custom_dir);
        let include_dir = cutlass_root.join("include");

        if !include_dir.exists() {
            panic!(
                "CUTLASS_DIR is set to '{}' but '{}' does not exist. \
                Please ensure CUTLASS_DIR points to a valid CUTLASS installation.",
                custom_dir,
                include_dir.display()
            );
        }

        println!(
            "cargo:warning=Using CUTLASS from CUTLASS_DIR: {}",
            cutlass_root.display()
        );
        emit_cargo_keys(&cutlass_root, &include_dir);
        return;
    }

    // 2. Check persistent cache directory
    let cache_dir = get_cache_dir().join("cutlass").join(&cutlass_version);
    let cached_include = cache_dir.join("include");

    if cached_include.exists() {
        println!(
            "cargo:warning=Using cached CUTLASS {} at {}",
            cutlass_version,
            cache_dir.display()
        );
        emit_cargo_keys(&cache_dir, &cached_include);
        return;
    }

    // 3. Download CUTLASS (with retry logic)
    println!(
        "cargo:warning=Downloading CUTLASS {} from GitHub...",
        cutlass_version
    );

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let temp_dir = out_dir.join("cutlass_download_temp");
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    match download_cutlass_with_retry(&cutlass_version, &temp_dir) {
        Ok(extracted_root) => {
            // Move to persistent cache
            fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
            copy_dir_all(&extracted_root, &cache_dir).expect("Failed to copy to cache");

            let include_dir = cache_dir.join("include");
            println!(
                "cargo:warning=CUTLASS {} downloaded and cached successfully",
                cutlass_version
            );
            emit_cargo_keys(&cache_dir, &include_dir);

            // Clean up temp directory
            let _ = fs::remove_dir_all(&temp_dir);
        }
        Err(e) => {
            eprintln!("\n========================================");
            eprintln!("ERROR: Failed to download CUTLASS {}", cutlass_version);
            eprintln!("========================================");
            eprintln!("Reason: {}", e);
            eprintln!("\nTo fix this issue, you can:");
            eprintln!("  1. Set CUTLASS_DIR environment variable to a local CUTLASS installation");
            eprintln!("     Example: CUTLASS_DIR=/path/to/cutlass cargo build");
            eprintln!("  2. Increase download timeout (default 120s):");
            eprintln!("     CUTLASS_DOWNLOAD_TIMEOUT=300 cargo build");
            eprintln!("  3. Clone CUTLASS manually and point to it:");
            eprintln!(
                "     git clone --depth 1 --branch {} https://github.com/NVIDIA/cutlass.git",
                cutlass_version
            );
            eprintln!("     CUTLASS_DIR=./cutlass cargo build");
            eprintln!("========================================\n");
            panic!("Failed to obtain CUTLASS. See error message above for solutions.");
        }
    }
}

fn emit_cargo_keys(root: &PathBuf, include_dir: &PathBuf) {
    // Emit multiple keys for maximum compatibility with consumers
    println!("cargo:root={}", root.display());
    println!("cargo:include={}", include_dir.display());
    println!("cargo:include_dir={}", include_dir.display());
    println!("cargo:INCLUDE_DIR={}", include_dir.display());

    // Also set rustc-env so compiled Rust code can access it
    println!(
        "cargo:rustc-env=CUTLASS_INCLUDE_DIR={}",
        include_dir.display()
    );
    println!("cargo:rustc-env=CUTLASS_ROOT={}", root.display());

    println!(
        "cargo:warning=CUTLASS include directory: {}",
        include_dir.display()
    );
}

fn get_cache_dir() -> PathBuf {
    // Try CARGO_HOME first, then user cache directory, finally temp
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        PathBuf::from(cargo_home).join("cutlass-sys-cache")
    } else if let Some(cache) = dirs::cache_dir() {
        cache.join("cutlass-sys")
    } else {
        env::temp_dir().join("cutlass-sys-cache")
    }
}

fn download_cutlass_with_retry(
    version: &str,
    temp_dir: &PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let max_retries = env::var("CUTLASS_DOWNLOAD_RETRIES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    let timeout_secs = env::var("CUTLASS_DOWNLOAD_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(120);

    let timeout = Duration::from_secs(timeout_secs);

    let mut last_error = None;

    for attempt in 1..=max_retries {
        if attempt > 1 {
            let backoff = Duration::from_secs(2u64.pow(attempt as u32 - 1));
            println!(
                "cargo:warning=Retry attempt {} after {:?} backoff...",
                attempt, backoff
            );
            thread::sleep(backoff);
        }

        // Try HTTP download first
        match try_http_download(version, temp_dir, timeout) {
            Ok(path) => return Ok(path),
            Err(e) => {
                println!(
                    "cargo:warning=HTTP download attempt {} failed: {}",
                    attempt, e
                );
                last_error = Some(format!("HTTP download failed: {}", e));
            }
        }
    }

    // Try git clone as fallback
    println!("cargo:warning=Trying git clone fallback...");
    match try_git_clone(version, temp_dir) {
        Ok(path) => {
            println!("cargo:warning=Git clone succeeded");
            return Ok(path);
        }
        Err(e) => {
            println!("cargo:warning=Git clone also failed: {}", e);
        }
    }

    Err(last_error
        .unwrap_or_else(|| "All download attempts failed".to_string())
        .into())
}

fn try_http_download(
    version: &str,
    temp_dir: &PathBuf,
    timeout: Duration,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use reqwest::blocking::Client;
    use std::io::Cursor;

    let url = format!(
        "https://github.com/NVIDIA/cutlass/archive/refs/tags/{}.tar.gz",
        version
    );

    println!("cargo:warning=Fetching {} (timeout: {:?})", url, timeout);

    let client = Client::builder().timeout(timeout).build()?;

    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    println!(
        "cargo:warning=Downloaded {} bytes, extracting...",
        bytes.len()
    );

    // Extract the tarball
    let tar = flate2::read::GzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(tar);

    let extract_dir = temp_dir.join("extract");
    fs::create_dir_all(&extract_dir)?;
    archive.unpack(&extract_dir)?;

    // Find the extracted directory (usually cutlass-<version>)
    let extracted_dir = fs::read_dir(&extract_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("cutlass"))
        .ok_or("Could not find extracted CUTLASS directory")?;

    Ok(extracted_dir.path())
}

fn try_git_clone(version: &str, temp_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::process::Command;

    let clone_dir = temp_dir.join("cutlass-git");

    // Remove if exists from previous attempt
    let _ = fs::remove_dir_all(&clone_dir);

    let output = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "--branch",
            version,
            "https://github.com/NVIDIA/cutlass.git",
            clone_dir.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "git clone failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    Ok(clone_dir)
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

/// Extract MAJOR.MINOR.PATCH from version string for CUTLASS mapping
/// Strips pre-release (-rc.1, -alpha, etc.) and build metadata (+build)
/// Examples:
///   "4.2.0-rc.1" -> "4.2.0"
///   "4.2.0" -> "4.2.0"
///   "3.9.2-beta.2" -> "3.9.2"
fn get_cutlass_version(pkg_version: &str) -> String {
    // Split on '-' to remove pre-release suffix (e.g., -rc.1)
    // Split on '+' to remove build metadata (though we don't use it)
    let base_version = pkg_version
        .split('-')
        .next()
        .unwrap()
        .split('+')
        .next()
        .unwrap();

    // Take first 3 components (MAJOR.MINOR.PATCH)
    let parts: Vec<&str> = base_version.split('.').take(3).collect();
    parts.join(".")
}

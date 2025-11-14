use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Use the crate version to determine which CUTLASS version to download
    // This way we only need to update the version in Cargo.toml
    let cutlass_version = format!("v{}", env!("CARGO_PKG_VERSION"));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cutlass_dir = out_dir.join("cutlass");
    let include_dir = cutlass_dir.join("include");

    // Check if CUTLASS is already downloaded
    if !include_dir.exists() {
        println!(
            "cargo:warning=Downloading CUTLASS {} from GitHub...",
            cutlass_version
        );
        download_cutlass(&cutlass_version, &cutlass_dir).expect("Failed to download CUTLASS");
        println!("cargo:warning=CUTLASS downloaded successfully");
    }

    // Emit the include path for dependent crates
    println!("cargo:include={}", include_dir.display());

    // Also set it as a DEP_CUTLASS_INCLUDE for downstream crates
    println!("cargo:INCLUDE_DIR={}", include_dir.display());

    // Inform cargo about the CUTLASS version
    println!("cargo:VERSION={}", cutlass_version);
}

fn download_cutlass(version: &str, target_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use reqwest::blocking::get;
    use std::io::Cursor;

    // Construct GitHub release URL
    let url = format!(
        "https://github.com/NVIDIA/cutlass/archive/refs/tags/{}.tar.gz",
        version
    );

    println!("cargo:warning=Fetching {}", url);

    // Download the archive
    let response = get(&url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download CUTLASS: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;

    // Extract the tarball
    let tar = flate2::read::GzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(tar);

    // Create target directory
    fs::create_dir_all(target_dir)?;

    // Extract to temporary location
    let temp_extract = target_dir.parent().unwrap().join("cutlass_extract_temp");
    fs::create_dir_all(&temp_extract)?;

    archive.unpack(&temp_extract)?;

    // Find the extracted directory (usually cutlass-<version>)
    let extracted_dir = fs::read_dir(&temp_extract)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().is_dir() && e.file_name().to_string_lossy().starts_with("cutlass"))
        .ok_or("Could not find extracted CUTLASS directory")?;

    // Move the contents to the target directory
    let source = extracted_dir.path();
    copy_dir_all(&source, target_dir)?;

    // Clean up temp directory
    fs::remove_dir_all(&temp_extract)?;

    Ok(())
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

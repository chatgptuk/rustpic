fn main() {
    // Generate a timestamp to use as the asset version
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Set the ASSET_VERSION environment variable for the build
    println!("cargo:rustc-env=ASSET_VERSION={}", timestamp);
    
    // Re-run build script if these files change (optional but good practice)
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=assets/");
}

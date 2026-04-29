//! Embeds the Win32 application manifest (ComCtl32 v6) for `titan-center` on Windows hosts.
//! See `assets/windows/titan-desktop.manifest` and Microsoft Learn: Enabling Visual Styles.

use std::path::Path;

fn main() {
    let manifest =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/windows/titan-desktop.manifest");
    println!("cargo:rerun-if-changed={}", manifest.display());

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    if !cfg!(windows) {
        println!(
            "cargo:warning=Skipping embedded Win32 manifest: build host is not Windows (cross-build)."
        );
        return;
    }

    let mut res = winres::WindowsResource::new();
    let manifest_str = manifest.to_str().expect("manifest path must be UTF-8");
    res.set_manifest_file(manifest_str);
    if let Err(e) = res.compile() {
        panic!("winres manifest embed failed: {e}");
    }
}

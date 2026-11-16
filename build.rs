use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os != "windows" || target_env != "gnu" {
        return;
    }

    let arch_dir = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "x64",
        Ok("aarch64") => "arm64",
        Ok("x86") => "x86",
        _ => "x64",
    };

    let Some(dll_src) = find_webview2_loader(arch_dir) else {
        println!(
            "cargo:warning=WebView2Loader.dll ({arch_dir}) not found in Cargo registry; ensure it is on PATH"
        );
        return;
    };

    // OUT_DIR lives inside <target>/<profile>/build/.../out; locate <target>/<profile>
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let out_path = PathBuf::from(out_dir);
        if let Some(profile_dir) = out_path
            .ancestors()
            .find(|p| p.file_name().is_some_and(|name| name == "build"))
            .and_then(|build_dir| build_dir.parent())
        {
            let dll_dst = profile_dir.join("WebView2Loader.dll");
            let needs_copy = match (fs::metadata(&dll_src), fs::metadata(&dll_dst)) {
                (Ok(src_meta), Ok(dst_meta)) => src_meta.len() != dst_meta.len(),
                (Ok(_), Err(_)) => true,
                _ => false,
            };
            if needs_copy {
                if let Err(err) = fs::copy(&dll_src, &dll_dst) {
                    println!(
                        "cargo:warning=Failed to copy WebView2Loader.dll to {}: {err}",
                        dll_dst.display()
                    );
                }
            }
        }
    }
}

fn find_webview2_loader(arch_dir: &str) -> Option<PathBuf> {
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".cargo")))
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".cargo")))?;

    let registry_src = cargo_home.join("registry").join("src");
    let entries = fs::read_dir(&registry_src).ok()?;

    for index_entry in entries.flatten() {
        let index_path = index_entry.path();
        if !index_path.is_dir() {
            continue;
        }
        if let Some(found) = search_webview2_crate(&index_path, arch_dir) {
            return Some(found);
        }
    }
    None
}

fn search_webview2_crate(index_dir: &Path, arch_dir: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(index_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("webview2-com-sys-") {
            let candidate = entry.path().join(arch_dir).join("WebView2Loader.dll");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}


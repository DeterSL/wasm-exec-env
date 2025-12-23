use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    cxx_build::bridge("src/ffi.rs")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .include("include")
        .compile("detersl_ffi");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crate_name = env::var("CARGO_PKG_NAME").unwrap();

    copy_generated_header(&out_dir, &crate_name, "src/ffi.rs", "include/ffi.rs.h");
}

fn copy_generated_header(out_dir: &Path, crate_name: &str, ffi_src: &str, dest_path: &str) {
    let candidates = [
        out_dir
            .join("cxxbridge")
            .join("include")
            .join(crate_name)
            .join(ffi_src),
        out_dir
            .join("cxxbridge")
            .join("crate")
            .join(crate_name)
            .join(ffi_src),
        out_dir
            .join("cxxbridge")
            .join(crate_name)
            .join("src")
            .join(ffi_src.replace('/', "_") + ".h"),
    ];

    let header = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .or_else(|| find_generated_header(out_dir, dest_path_file_name(dest_path)))
        .unwrap_or_else(|| {
            println!(
                "cargo:warning=Could not locate generated header for {} under {}",
                ffi_src,
                out_dir.display()
            );
            dump_dir(&out_dir.join("cxxbridge"));
            std::process::exit(1);
        });

    let dest = PathBuf::from(dest_path);
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::copy(&header, &dest) {
        Ok(_) => println!(
            "cargo:warning=Copied generated header:\n  from: {}\n  to:   {}",
            header.display(),
            dest.display()
        ),
        Err(e) => {
            println!(
                "cargo:warning=Could not copy header: {}.\n  from: {}\n  to:   {}",
                e,
                header.display(),
                dest.display()
            );
            std::process::exit(1);
        }
    }
}

fn find_generated_header(out_dir: &Path, file_name: &str) -> Option<PathBuf> {
    let root = out_dir.join("cxxbridge");
    if !root.exists() {
        return None;
    }
    fn rec(dir: &Path, needle: &str) -> Option<PathBuf> {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = rec(&path, needle) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|s| s.to_str()) == Some(needle) {
                return Some(path);
            }
        }
        None
    }
    rec(&root, file_name)
}

fn dest_path_file_name(dest_path: &str) -> &str {
    Path::new(dest_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(dest_path)
}

fn dump_dir(dir: &Path) {
    if !dir.exists() {
        println!("cargo:warning=dir does not exist: {}", dir.display());
        return;
    }
    println!("cargo:warning=Listing {}", dir.display());
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            println!("cargo:warning= - {}", entry.path().display());
        }
    }
}

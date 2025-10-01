use std::{env, fs, path::{Path, PathBuf}};

fn main() {
    let mut build = cxx_build::bridge("src/ffi.rs");
    build
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17");

    build.compile("detersl_ffi");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crate_name = env::var("CARGO_PKG_NAME").unwrap();

    let expected = out_dir
        .join("cxxbridge")
        .join(&crate_name)
        .join("src")
        .join("ffi.rs.h");

    let header = if expected.exists() {
        expected
    } else {
        match find_generated_header(&out_dir) {
            Some(p) => p,
            None => {
                println!(
                    "cargo:warning=Could not locate generated header under {}",
                    out_dir.display()
                );
                dump_dir(&out_dir.join("cxxbridge"));
                return;
            }
        }
    };

    let dest = PathBuf::from("include").join("ffi.rs.h");
    if let Some(parent) = dest.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::copy(&header, &dest) {
        Ok(_) => println!(
            "cargo:warning=Copied generated header:\n  from: {}\n  to:   {}",
            header.display(),
            dest.display()
        ),
        Err(e) => println!(
            "cargo:warning=Could not copy header: {}.\n  from: {}\n  to:   {}",
            e,
            header.display(),
            dest.display()
        ),
    }
}

fn find_generated_header(out_dir: &Path) -> Option<PathBuf> {
    let root = out_dir.join("cxxbridge");
    if !root.exists() {
        return None;
    }
    fn rec(dir: &Path) -> Option<PathBuf> {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = rec(&path) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|s| s.to_str()) == Some("ffi.rs.h") {
                return Some(path);
            }
        }
        None
    }
    rec(&root)
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

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::copy;
use std::path::Path;

extern crate walkdir;
use walkdir::{DirEntry, WalkDir};

fn main() {

    // Assume libsdl2*-dev is installed on BSD, but the link search path may not include the directory
    // containing the libs.
    if cfg!(any(target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "dragonfly")) {
        println!("cargo:rustc-link-search=/usr/local/lib");
    }

    // Assume libsdl2*-dev etc. are installed with the package manager on unix family systems but on
    // a windows OS we look for the C built sdl2 libraries for the relevant platform/architecture to be
    // provided in a sub-directory and add them to the Link arguments.
    // We also ensure that the DLLs have been copied to the project root (or SDL_DLLS_RUN_DIR env var dir)
    // so that cargo run can find them.
    if cfg!(target_family = "windows") {

        // Assuming cargo always sets this environment variable!
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let win_sdl_dlls_dir = if let Ok(dir) = env::var("SDL_DLLS_RUN_DIR") {
            dir
        } else {
            manifest_dir.clone()
        };

        let win_sdl_libs_dir = format!("{}/sdl_libs", manifest_dir);

        let is_x64 = cfg!(target_arch = "x86_64");
        let is_mingw = cfg!(target_env = "gnu");

        let select_libs_dir = |base_dir| {
            let dir = format!("{}/{}/{}/{}",
                              win_sdl_libs_dir,
                              base_dir,
                              if is_mingw { "mingw" } else { "msvc" },
                              if is_x64 { "x64" } else { "x32" });
            dir
        };

        let machine_lib_dirs =
            [select_libs_dir("sdl2"), select_libs_dir("sdl2-image"), select_libs_dir("sdl2-ttf")];

        for dir in &machine_lib_dirs {

            println!("cargo:rustc-flags=-L {}", dir);

            // Copy sdl2 related DLLs to win_sdl_dlls_dir so `cargo run` succeeds.
            for entry in WalkDir::new(dir) {
                let entry = entry.unwrap();

                if is_dll_file(&entry) {

                    let src_file_path: &Path = entry.path();
                    let src_file_name: &OsStr = entry.file_name();
                    let target_dir: &String = &win_sdl_dlls_dir;
                    let target_dir_path: &Path = Path::new(target_dir);
                    let target_file_str: OsString = target_dir_path.join(src_file_name)
                        .into_os_string();
                    let target_file_path: &Path = Path::new(&target_file_str);

                    if !target_file_path.exists() ||
                       are_different_file_contents(&entry, target_file_path) {

                        copy(src_file_path, target_file_path)
                            .expect(format!("Failed to copy windows os dll from {} to {}",
                                            src_file_path.display(),
                                            target_file_path.display())
                                            .as_str());
                    }
                }
            }
        }
    }
}

fn is_dll_file(entry: &DirEntry) -> bool {

    if entry.file_type().is_file() {
        if let Some(osstr_ext) = entry.path().extension() {
            if let Some(ext) = osstr_ext.to_str() {
                return ext == "dll";
            }
        }
    }

    false
}

fn are_different_file_contents(entry: &DirEntry, file_path: &Path) -> bool {
    let entry_meta_r = entry.metadata();
    let file_meta_r = file_path.metadata();

    if let (Ok(entry_meta), Ok(file_meta)) = (entry_meta_r, file_meta_r) {

        if entry_meta.is_file() && file_meta.is_file() && entry_meta.len() == file_meta.len() {
            false
        } else {
            true
        }

    } else {
        // When we cannot read the metadata then default to assuming
        // different files.
        true
    }
}

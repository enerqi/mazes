use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::copy;
use std::path::Path;

extern crate walkdir;
use walkdir::{DirEntry, WalkDir};

fn main() {

    // Assume libsdl2*-dev etc. are installed with the package manager on unix family systems.
    //
    // On a windows OS we look for the C built sdl2 libraries for the relevant platform/architecture
    // and add them to the Link arguments.
    // We also ensure that the DLLs have been copied to ./sdl_libs (or SDL_LIBS_DIR).
    // The `cargo run` command will look for sdl2 dlls in ./sdl_libs...it's unknown why cargo cannot
    // be told to find the DLLs in subdirectories of sdl_libs.
    if cfg!(target_family = "windows") {

        let root_libs_dir = if let Ok(dir) = env::var("SDL_LIBS_DIR") {
            Some(dir)
        } else {
            if let Ok(cargo_root_dir) = env::var("CARGO_MANIFEST_DIR") {
                Some(format!("{}/sdl_libs", cargo_root_dir))
            } else {
                None
            }
        };

        if let Some(libs) = root_libs_dir {

            // Add link flag for the compiler
            println!("cargo:rustc-flags=-L {}", libs);

            // Copy sdl2 related DLLs to the root libs directory.
            let is_x64 = cfg!(target_arch = "x86_64");
            let is_mingw = cfg!(target_env = "gnu");

            let select_libs_dir = |base_dir| {
                let dir = format!("{}/{}/{}/{}",
                                  libs,
                                  base_dir,
                                  if is_mingw {
                                      "mingw"
                                  } else {
                                      "msvc"
                                  },
                                  if is_x64 {
                                      "x64"
                                  } else {
                                      "x32"
                                  });
                dir
            };

            let machine_lib_dirs = [select_libs_dir("sdl2"),
                                    select_libs_dir("sdl2-image"),
                                    select_libs_dir("sdl2-ttf")];

            for dir in &machine_lib_dirs {

                println!("cargo:rustc-flags=-L {}", dir);

                for entry in WalkDir::new(dir) {
                    let entry = entry.unwrap();

                    if is_dll_file(&entry) {

                        let src_file_path: &Path = entry.path();
                        let src_file_name: &OsStr = entry.file_name();
                        let target_dir: &String = &libs;
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

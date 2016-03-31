use std::env;

fn main() {

    // Assume libsdl2*-dev etc. are installed with the package manager on unix family systems.
    // On a windows OS we look for the C built sdl2 libraries in ./sdl_libs.
    // The `cargo run` command will also look for sdl2 dlls in the same directory.
    if cfg!(target_family = "windows") {

        let libs_dir =
            if let Ok(dir) = env::var("SDL_LIBS_DIR") {
                Some(dir)
            }
            else {
                if let Ok(cargo_root_dir) = env::var("CARGO_MANIFEST_DIR") {
                    Some(format!("{}/sdl_libs", cargo_root_dir))
                }
                else {
                    None
                }
            };

        if let Some(libs) = libs_dir {
            println!("cargo:rustc-flags=-L {}", libs);
        }
    }
}

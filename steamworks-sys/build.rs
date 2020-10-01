extern crate bindgen;

#[cfg(feature = "docs-only")]
fn main() {}

#[cfg(not(feature = "docs-only"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::fs::{self};

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_loc = env::var("STEAM_SDK_LOCATION")
        .expect("STEAM_SDK_LOCATION must be set");
    let sdk_loc = Path::new(&sdk_loc);

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut link_path = sdk_loc.join("redistributable_bin");
    if triple.contains("windows") {
        if !triple.contains("i686") {
            lib = "steam_api64";
            link_path.push("win64");
        }
    } else if triple.contains("linux") {
        if triple.contains("i686") {
            link_path.push("linux32");
        } else {
            link_path.push("linux64");
        }
    } else if triple.contains("darwin") {
        link_path.push("osx");
    } else {
        panic!("Unsupported OS");
    };
    println!("cargo:rustc-link-search={}", link_path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);

    let bindings = bindgen::Builder::default()
        .header("src/fixes.hpp")
        .header(sdk_loc.join("public/steam/steam_api_flat.h").to_string_lossy())
        .header(sdk_loc.join("public/steam/steam_gameserver.h").to_string_lossy())
        .clang_arg("-xc++")
        .opaque_type("SteamTVRegion_t")
        .clang_arg(format!("-I{}", sdk_loc.join("public").display()))
        .rustfmt_bindings(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true
        })
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(
            if env::var("STEAM_SDK_MAKE_BINDINGS").is_ok() {
                Path::new("src/bindings.rs").to_owned()
            } else {
                out_path.join("bindings.rs")
            }
        )
        .expect("Couldn't write bindings!");

    if triple.contains("windows") {
        let file_name = format!("{}.dll", lib);
        fs::copy(link_path.join(&file_name), out_path.join(file_name))?;
    } else if triple.contains("darwin") {
        fs::copy(link_path.join("libsteam_api.dylib"), out_path.join("libsteam_api.dylib"))?;
    } else if triple.contains("linux") {
        fs::copy(link_path.join("libsteam_api.so"), out_path.join("libsteam_api.so"))?;
    }

    Ok(())
}

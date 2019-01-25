/* raylib-sys
   build.rs - Cargo build script

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=gdi32");
    }
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=X11");
    }
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=OpenGL");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
    }

    if pkg_config::Config::new()
        .atleast_version("2.0.0")
        .probe("raylib")
        .is_ok()
    {
        // no need to build if we already have a system raylib
        return;
    }

    let source_url =
        url::Url::parse("https://github.com/raysan5/raylib/archive/2.0.0.tar.gz").unwrap();

    let download_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("download");

    if !download_dir.exists() {
        fs::create_dir(&download_dir).unwrap();
    }

    let source_tarball_filename = source_url.path_segments().unwrap().last().unwrap();
    let source_tarball_path = download_dir.join(source_tarball_filename);

    if !source_tarball_path.exists() {
        let f = File::create(&source_tarball_path).unwrap();
        let mut writer = BufWriter::new(f);
        let mut easy = curl::easy::Easy::new();
        easy.url(source_url.as_str()).unwrap();
        easy.follow_location(true).unwrap();
        easy.write_function(move |data| Ok(writer.write(data).unwrap()))
            .unwrap();
        easy.perform().unwrap();

        let response_code = easy.response_code().unwrap();
        if response_code != 200 {
            panic!(
                "Unexpected response code {} for {}",
                response_code, source_url
            );
        }
    }

    let extract_dir = download_dir.join("raylib-2.0.0");
    if !extract_dir.exists() {
        let file = File::open(source_tarball_path).unwrap();
        let unzipped = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(unzipped);
        archive.unpack(download_dir).unwrap();
    }

    let mut config = cmake::Config::new(extract_dir);
    config.define("BUILD_EXAMPLES", "OFF");
    config.define("BUILD_GAMES", "OFF");

    if cfg!(target_os = "macos") {
        config.define("MACOS_FATLIB", "OFF"); // rust can't handle universal binaries? https://github.com/rust-lang/rust/issues/50220
        config.generator("Ninja"); // default doesn't work?
    }

    let build_destination = config.build();

    println!(
        "cargo:rustc-link-search=native={}",
        build_destination.join("lib").display()
    );

    println!("cargo:rustc-link-lib=static=raylib");
}

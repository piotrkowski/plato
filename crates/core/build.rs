use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-check-cfg=cfg(mupdf_tofu)");
    println!("cargo:rustc-check-cfg=cfg(plato_no_djvu)");

    // Cross-compiling for Kobo.
    if target == "arm-unknown-linux-gnueabihf" {
        println!("cargo:rustc-env=PKG_CONFIG_ALLOW_CROSS=1");
        println!("cargo:rustc-link-search=target/mupdf_wrapper/Kobo");
        println!("cargo:rustc-link-search=libs");
        println!("cargo:rustc-link-lib=dylib=stdc++");
    // Fully static Kindle build (Aristotle, tools/build-plato.sh): every C
    // library is a static archive in libs/, built by Zig for musl. The same
    // archives built for x86_64 musl give a host build for previews.
    // PLATO_LIB_DIRS (a PATH-style list) points other workspaces, such as
    // Aristotle's aristotle-render, at the archives of a build-plato.sh run.
    } else if target == "armv7-unknown-linux-musleabihf" || target == "x86_64-unknown-linux-musl" {
        let root = format!("{}/../..", env::var("CARGO_MANIFEST_DIR").unwrap());
        println!("cargo:rerun-if-env-changed=PLATO_LIB_DIRS");
        if let Some(dirs) = env::var_os("PLATO_LIB_DIRS") {
            for dir in env::split_paths(&dirs) {
                println!("cargo:rustc-link-search=native={}", dir.display());
            }
        }
        println!("cargo:rustc-link-search=native={}/target/mupdf_wrapper/Kindle", root);
        println!("cargo:rustc-link-search=native={}/libs", root);
        // A MuPDF built without its Noto and CJK fonts (TOFU) says so in a
        // `mupdf-fonts` file next to the archives; the fallback fonts that
        // font/mod.rs takes from MuPDF are then left out.
        let mut dirs: Vec<_> = env::var_os("PLATO_LIB_DIRS")
            .map(|d| env::split_paths(&d).collect()).unwrap_or_default();
        dirs.push(format!("{}/libs", root).into());
        for dir in dirs.iter().filter(|d| d.is_dir()) {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
        let fonts = dirs.iter().find_map(|d| std::fs::read_to_string(d.join("mupdf-fonts")).ok());
        if fonts.as_deref().map(str::trim).is_some_and(|f| f != "full") {
            println!("cargo:rustc-cfg=mupdf_tofu");
        }
        // DjVu is optional: without libdjvulibre.a the djvu module is left
        // out and DjVu files are not opened.
        if !dirs.iter().any(|d| d.join("libdjvulibre.a").is_file()) {
            println!("cargo:rustc-cfg=plato_no_djvu");
        }
        println!("cargo:rustc-link-lib=static=mupdf-third");
        println!("cargo:rustc-link-lib=static=c++");
        println!("cargo:rustc-link-lib=static=c++abi");
    // Handle the Linux and macOS platforms.
    } else {
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
        match target_os.as_ref() {
            "linux" => {
                println!("cargo:rustc-link-search=target/mupdf_wrapper/Linux");
                println!("cargo:rustc-link-lib=dylib=stdc++");
            },
            "macos" => {
                println!("cargo:rustc-link-search=target/mupdf_wrapper/Darwin");
                println!("cargo:rustc-link-lib=dylib=c++");
            },
            _ => panic!("Unsupported platform: {}.", target_os),
        }

        println!("cargo:rustc-link-lib=mupdf-third");
    }

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=jpeg");
    println!("cargo:rustc-link-lib=png16");
    println!("cargo:rustc-link-lib=gumbo");
    println!("cargo:rustc-link-lib=openjp2");
    println!("cargo:rustc-link-lib=jbig2dec");
}

#[cfg(feature = "native")]
use cc;
#[cfg(feature = "native")]
use std::env;
#[cfg(feature = "native")]
use std::path;

// gcc -c -o vsprintf.o vsprintf.c && ar rcs libvprintf.a vsprintf.o

fn main() {
    #[cfg(feature = "native")]
    {
        println!("cargo:rustc-link-lib=raylib");
    }

    // Compile and link to get the vprintf library
    #[cfg(feature = "native")]
    {
        let _dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not found");
        let dir = path::Path::new(&_dir);

        // See https://docs.rs/cc/latest/cc/ for more info on build-time compilation of C code
        cc::Build::new()
            .file(dir.join("log").join("vsprintf.c"))
            .compile("vprintf");
    }
}

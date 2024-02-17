extern crate conan2;
extern crate home;
extern crate walkdir;

use conan2::ConanInstall;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // Don't link with libheif in case of building documentation for docs.rs.
        println!("cargo:rustc-cfg=docs_rs");
        return;
    }

    let mut include_dirs: Vec<String> = Vec::new();

    ConanInstall::new().build("missing").run().parse().emit();

    let conan_dir = match std::env::var("CONAN_HOME").ok() {
      None                    => home::home_dir().unwrap().join(".conan2"),
      Some(dir)       => PathBuf::from(dir)
    };
    let build_paths = WalkDir::new(conan_dir.to_str().unwrap()).max_depth(10).into_iter()
        .filter_map(|e| e.ok())
        .filter(|p| p.path().to_str().unwrap().ends_with("include/libheif"))
        .collect::<Vec<_>>();

    let include_path = build_paths.first().unwrap().path().parent().unwrap();

    include_dirs.push(include_path.to_str().unwrap().to_string());

    #[cfg(feature = "use-bindgen")]
    {
        use std::env;
        use std::path::PathBuf;
        // The bindgen::Builder is the main entry point
        // to bindgen, and lets you build up options for
        // the resulting bindings.
        let mut builder = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("wrapper.h")
            .generate_comments(true)
            .generate_cstr(true)
            .ctypes_prefix("libc")
            .allowlist_function("heif_.*")
            .allowlist_type("heif_.*")
            .size_t_is_usize(true)
            .clang_args([
                "-fparse-all-comments",
                "-fretain-comments-from-system-headers",
            ]);
        if !include_dirs.is_empty() {
            dbg!(&include_dirs);
            builder = builder.clang_args(include_dirs.iter().map(|dir| format!("--include-directory={}", dir)));
        }

        // Finish the builder and generate the bindings.
        let bindings = builder
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        // Write the bindings to the $OUT_DIR/bindings.rs file.
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}

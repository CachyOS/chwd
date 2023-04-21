use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=hd");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // Compile static helper library
    compile_helper_lib();

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .size_t_is_usize(true)
        .use_core()
        .allowlist_function("hd_.*")
        .allowlist_var("HD_.*")
        .allowlist_var("TAG_.*")
        .allowlist_var("ID_.*")
        .blocklist_type("FILE")
        .blocklist_type("timespec")
        //.blocklist_type("stat")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed);

    #[cfg(not(feature = "std"))]
    let bindings = bindings.ctypes_prefix("libc").raw_line("use libc::FILE;");

    // Finish the builder and generate the bindings.
    let bindings = bindings
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}

fn compile_helper_lib() {
    cc::Build::new()
        // Add file
        .file("wrapper.c")
        // Some extra parameters
        .flag_if_supported("-ffunction-sections")
        .flag_if_supported("-fdata-sections")
        .flag_if_supported("-fmerge-all-constants")
        // Compile!
        .compile("libhd_utils.a");
}

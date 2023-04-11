use std::fs;
//use std::path::PathBuf;

//const C_HEADER_OUTPUT: &'static str = "chwd-rust.h";

fn main() -> Result<(), &'static str> {
    for i in fs::read_dir("src").unwrap() {
        println!("cargo:rerun-if-changed={}", i.unwrap().path().display());
    }

    cxx_build::bridge("src/lib.rs").compile("chwd-rust");

//    execute_cbindgen()?;

    Ok(())
}

/*
/// Use cbindgen to generate C-header's for Rust static libraries.
fn execute_cbindgen() -> Result<(), &'static str> {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").or(Err("CARGO_MANIFEST_DIR not specified"))?;
    let build_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| ".".into()));
    let outfile_path = build_dir.join(C_HEADER_OUTPUT);

    // Useful for build diagnostics
    eprintln!("cbindgen outputting {:?}", &outfile_path);
    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(&outfile_path);

    Ok(())
}*/

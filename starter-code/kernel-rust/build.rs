use bindgen;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // --- Step 1: Bindgen for C Header Bindings ---
    let header = "../kernel/kernel.h";
    
    println!("cargo:rerun-if-changed={}", header);
    println!("cargo:rerun-if-changed=../shared/x86-64.h");
    println!("cargo:rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header(header)
        .clang_arg("-I../shared")          // Include path for `x86-64.h`
        .clang_arg("-include")
        .clang_arg("stdint.h")
        .clang_arg("-fpack-struct")        // Struct packing
        .clang_arg("-m64")                 // Target 64-bit
        .generate()
        .expect("Unable to generate bindings for kernel.h");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings_kernel.rs"))
        .expect("Couldn't write bindings for kernel.h");

    // --- Step 2: Compile Rust Code to Object File ---
    // Define source file for the Rust object
    let source_file = "src/kloader.rs";
    
    let output_dir = env::var("OUT_DIR").unwrap();
    let output_file = format!("{}/k-loader.o", output_dir);

    // Compile object files (.o)
    let status = Command::new("rustc")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .arg("-C")
        .arg("opt-level=3")
        .arg("--emit=obj")
        .arg("-o")
        .arg(output_file)
        .arg(source_file)
        .status()
        .expect("Failed to execute rustc");

    if !status.success() {
        panic!("rustc failed to compile object file");
    }

    // Rerun if `src/kloader.rs` changes
    println!("cargo:rerun-if-changed={}", source_file);
}

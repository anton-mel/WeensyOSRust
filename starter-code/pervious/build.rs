// use bindgen;
use std::path::PathBuf;
use std::process::Command;
use std::fs::{self, OpenOptions};
use std::io::Read;

fn main() {
    // --- Step 1: Bindgen for C Header Bindings ---
    let header = "../kernel/kernel.h";

    println!("cargo:rerun-if-changed={}", header);
    println!("cargo:rerun-if-changed=../shared/x86-64.h");
    println!("cargo:rerun-if-changed=build.rs");

    let src_path = PathBuf::from("src");
    if !src_path.exists() {
        panic!("kernel-rust folder must be present to generate bindings");
    }

    // Generate bindings and save them in src/bindings_kernel.rs
    let bindings = bindgen::Builder::default()
        .header(header)
        .clang_arg("-I../shared") // Include path for x86-64.h
        .clang_arg("-include")
        .clang_arg("stdint.h")
        .clang_arg("-fpack-struct") // Struct packing
        .clang_arg("-m64")          // Target 64-bit
        .use_core()
        .generate()
        .expect("Unable to generate bindings for kernel.h");

    let bindings_file = src_path.join("bindings_kernel.rs");
    bindings
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings for kernel.h");

    // --- Step 2: Ensure Cargo knows to generate object files (.o) ---

    let bindings_file = "src/bindings_kernel.rs";
    
    // Read the existing content of the file
    let mut original_content = String::new();
    if let Ok(mut file) = OpenOptions::new().read(true).open(bindings_file) {
        file.read_to_string(&mut original_content).expect("Failed to read bindings file");
    }

    // Prepend the attributes
    let new_content = r#"#![no_std]
#![no_main]

#![allow(unused)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

"#.to_string() + &original_content;

    // Write the modified content back to the file
    fs::write(bindings_file, new_content).expect("Failed to write to bindings file");

    let status = Command::new("rustc")
        .args(&[
            "--crate-type", "lib",
            "src/bindings_kernel.rs",
            "--emit=obj",
            "-o", "target/bindings_kernel.o"
        ])
        .status()
        .expect("Failed to compile bindings_kernel.rs");

    if !status.success() {
        panic!("rustc command failed for bindings_kernel.rs");
    }

    let status = Command::new("rustc")
        .args(&[
            "--crate-type", "lib",
            "src/kloader.rs",
            "--emit=obj",
            "-o", "target/kloader.o",
            "-L", "target", // Specify directory containing bindings_kernel.o
            "-l", "static=bindings_kernel" // Link against the generated object
        ])
        .status()
        .expect("Failed to compile kloader.rs");

    if !status.success() {
        panic!("rustc command failed for kloader.rs");
    }

}

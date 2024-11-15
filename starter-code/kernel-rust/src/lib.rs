// Avoid Rust-C warnings
#![no_std]
#![no_main]
#![allow(unused)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod kloader;

// Bindings to the C-headers
// Includes x86-64, lib, and elf headers
include!(concat!(env!("OUT_DIR"), "/bindings_kernel.rs"));

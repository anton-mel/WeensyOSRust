#![no_std]
#![no_main]
#![allow(unused)]

use bindings::{
    bindings_kernel,
    bindings_x86_64,
    bindings_elf,
    bindings_lib,
};

mod kernel;
extern crate alloc;

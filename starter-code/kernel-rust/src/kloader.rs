use crate::proc_;
use std::os::raw::c_void;
use std::os::raw::c_int;

// k-loader.c
//
//    Load a weensy application into memory from a RAM image.

extern "C" {
    pub static _binary_obj_p_allocator_start: u8;
    pub static _binary_obj_p_allocator_end: u8;
    pub static _binary_obj_p_allocator2_start: u8;
    pub static _binary_obj_p_allocator2_end: u8;
    pub static _binary_obj_p_allocator3_start: u8;
    pub static _binary_obj_p_allocator3_end: u8;
    pub static _binary_obj_p_allocator4_start: u8;
    pub static _binary_obj_p_allocator4_end: u8;
    pub static _binary_obj_p_fork_start: u8;
    pub static _binary_obj_p_fork_end: u8;
    pub static _binary_obj_p_forkexit_start: u8;
    pub static _binary_obj_p_forkexit_end: u8;
    pub static _binary_obj_p_test_start: u8;
    pub static _binary_obj_p_test_end: u8;
}

#[repr(C)]
pub struct RamImage {
    begin: &'static u8,
    end: &'static u8,
}

pub static RAMIMAGES: [RamImage; 7] = [
    RamImage { begin: unsafe { &_binary_obj_p_allocator_start }, end: unsafe { &_binary_obj_p_allocator_end } },
    RamImage { begin: unsafe { &_binary_obj_p_allocator2_start }, end: unsafe { &_binary_obj_p_allocator2_end } },
    RamImage { begin: unsafe { &_binary_obj_p_allocator3_start }, end: unsafe { &_binary_obj_p_allocator3_end } },
    RamImage { begin: unsafe { &_binary_obj_p_allocator4_start }, end: unsafe { &_binary_obj_p_allocator4_end } },
    RamImage { begin: unsafe { &_binary_obj_p_fork_start }, end: unsafe { &_binary_obj_p_fork_end } },
    RamImage { begin: unsafe { &_binary_obj_p_forkexit_start }, end: unsafe { &_binary_obj_p_forkexit_end } },
    RamImage { begin: unsafe { &_binary_obj_p_test_start }, end: unsafe { &_binary_obj_p_test_end } },
];


// program_load(p, programnumber)
//    Load the code corresponding to program `programnumber` into the process
//    `p` and set `p->p_registers.reg_rip` to its entry point. Calls
//    `assign_physical_page` to as required. Returns 0 on success and
//    -1 on failure (e.g. out-of-memory). `allocator` is passed to
//    `virtual_memory_map`.

pub unsafe extern "C" fn program_load(p: *mut proc_, programnumber: i32, allocator: extern fn() -> *mut std::ffi::c_void) -> i32 {
    0 // Default return value (indicating success)
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct elf_program {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_va: u64,
    pub p_pa: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

// program_load_segment(p, ph, src, allocator)
//    Load an ELF segment at virtual address `ph->p_va` in process `p`. Copies
//    `[src, src + ph->p_filesz)` to `dst`, then clears
//    `[ph->p_va + ph->p_filesz, ph->p_va + ph->p_memsz)` to 0.
//    Calls `assign_physical_page` to allocate pages and `virtual_memory_map`
//    to map them in `p->p_pagetable`. Returns 0 on success and -1 on failure.

pub unsafe extern "C" fn program_load_segment(p: *mut proc_, ph: *const elf_program, src: *const u8, allocator: extern fn() -> *mut c_void) -> c_int {
    0 // Default return value (indicating success)
}

// dummy main
fn main() {}

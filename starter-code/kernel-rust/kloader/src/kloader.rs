// Fix:: Add the panic for boundary checking
// and fix the pointer arithmetic

#![no_std]
#![no_main]

#![allow(dead_code, unused)]

use crate::bindings::{
    proc_, 
    elf_program, 
    elf_header,
    ELF_MAGIC,
    ELF_PTYPE_LOAD,
};

use core::ffi::c_void;
use core::ffi::c_int;

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

#[no_mangle]
pub unsafe extern "C" fn program_load(p: *mut proc_, programnumber: usize, allocator: extern fn() -> *mut c_void) -> i32 {
    // is this a valid program?
    let n_programs = RAMIMAGES.len();
    if programnumber >= n_programs {
        loop {} // panic here
    }

    // Get the elf_header from the RAM image at the specified program_number
    let ram_image = &RAMIMAGES[programnumber];
    let eh_ptr = ram_image.begin as *const u8; // Pointer to the start of the image
    let eh: &elf_header = unsafe { &*(eh_ptr as *const elf_header) }; // Cast to elf_header
    if (eh.e_magic != ELF_MAGIC) {
        loop {} // panic here
    }

    // Load each loadable program segment into memory
    let ph: &mut [elf_program] = unsafe {
        let program_header_ptr = (eh as *const elf_header as *const u8).offset(eh.e_phoff as isize);
        &mut *(program_header_ptr as *mut [elf_program; 10]) // Adjust array size as necessary
    };

    unsafe {
        // Use a raw pointer to manually access the array and avoid bounds checking
        let ph_ptr = ph.as_mut_ptr();
        
        for i in 0..eh.e_phnum as usize {
            // Ensure that the pointer is within bounds
            let ph_element = ph_ptr.add(i);
            
            // Manually check if we're within the bounds (no bounds checking done by Rust)
            if ph_element < ph_ptr.add(ph.len()) {
                // Safe to dereference without bounds checking
                if (*ph_element).p_type == ELF_PTYPE_LOAD {
                    let pdata = (eh as *const elf_header as *const u8)
                        .offset((*ph_element).p_offset as isize);
                    
                    if program_load_segment(p, ph_element, pdata, allocator) < 0 {
                        return -1;
                    }
                }
            } else {
                loop {} // Handle out-of-bounds access (you can customize this)
            }
        }
    }

    // Return to this solution later on
    // for i in 0..eh.e_phnum as usize {
    //     if ph[i].p_type == ELF_PTYPE_LOAD {
    //         let pdata = unsafe {
    //             (eh as *const elf_header as *const u8).offset(ph[i].p_offset as isize)
    //         };

    //         if program_load_segment(p, &ph[i], pdata, allocator) < 0 {
    //             return -1; // Return failure code if segment load fails
    //         }
    //     }
    // }

    // set the entry point from the ELF header
    (*p).p_registers.reg_rip = eh.e_entry;
    0 // Success (Required by C-kernel)
}

// program_load_segment(p, ph, src, allocator)
//    Load an ELF segment at virtual address `ph->p_va` in process `p`. Copies
//    `[src, src + ph->p_filesz)` to `dst`, then clears
//    `[ph->p_va + ph->p_filesz, ph->p_va + ph->p_memsz)` to 0.
//    Calls `assign_physical_page` to allocate pages and `virtual_memory_map`
//    to map them in `p->p_pagetable`. Returns 0 on success and -1 on failure.

#[no_mangle]
pub unsafe extern "C" fn program_load_segment(p: *mut proc_, ph: *const elf_program, src: *const u8, allocator: extern fn() -> *mut c_void) -> c_int {
    0 // Default return value (indicating success)
}


use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Handle this later
    // using bindings
    loop {};
}

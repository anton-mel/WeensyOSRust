use crate::*;
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
pub unsafe extern "C" fn program_load(
    p: *mut Proc, 
    programnumber: usize, 
    allocator: extern fn() -> *mut c_void
) -> i32 {
    // is this a valid program?
    let n_programs = RAMIMAGES.len();
    // assert!(programnumber < n_programs);

    // // Get the ElfHeader from the RAM image at the specified program_number
    // let ram_image = &RAMIMAGES[programnumber];
    // let eh_ptr = ram_image.begin as *const u8; // Pointer to the start of the image
    // let eh: &ElfHeader = unsafe { &*(eh_ptr as *const ElfHeader) }; // Cast to ElfHeader
    // assert!(eh.e_magic == ELF_MAGIC);

    // // Load each loadable program segment into memory
    // let ph: &mut [ElfProgram] = unsafe {
    //     let program_header_ptr = (eh as *const ElfHeader as *const u8).offset(eh.e_phoff as isize);
    //     &mut *(program_header_ptr as *mut [ElfProgram; 10]) // Adjust array size as necessary
    // };

    // // Return to this solution later on
    // for i in 0..eh.e_phnum as usize {
    //     if ph[i].p_type == ELF_PTYPE_LOAD {
    //         let pdata = unsafe {
    //             (eh as *const ElfHeader as *const u8).offset(ph[i].p_offset as isize)
    //         };

    //         if program_load_segment(p, &ph[i], pdata, allocator) < 0 {
    //             return -1; // Return failure code if segment load fails
    //         }
    //     }
    // }

    // // set the entry point from the ELF header
    // (*p).p_registers.reg_rip = eh.e_entry;
    0 // Success (Required by C-kernel)
}

// program_load_segment(p, ph, src, allocator)
//    Load an ELF segment at virtual address `ph->p_va` in process `p`. Copies
//    `[src, src + ph->p_filesz)` to `dst`, then clears
//    `[ph->p_va + ph->p_filesz, ph->p_va + ph->p_memsz)` to 0.
//    Calls `assign_physical_page` to allocate pages and `virtual_memory_map`
//    to map them in `p->p_pagetable`. Returns 0 on success and -1 on failure.

#[no_mangle]
pub unsafe extern "C" fn program_load_segment(
    p: *mut Proc,
    ph: *const ElfProgram,
    _src: *const u8,
    _allocator: extern "C" fn() -> *mut c_void,
) -> c_int {
    if p.is_null() || ph.is_null() {
        return -1; // Validate pointers
    }

    let mut va = (*ph).p_va;
    let _end_file = va + (*ph).p_filesz;
    let end_mem = va + (*ph).p_memsz;
    va &= !(PAGESIZE - 1);       // round to page boundary

    // allocate memory
    while va < end_mem {
        // Note: First, implement kernel crate
        // if assign_physical_page(va, (*p).p_pid) < 0
        //     || virtual_memory_map((*p).p_pagetable, va, va, PAGESIZE, PTE_P | PTE_W | PTE_U) < 0
        // {
        //     eprintln!(
        //         "program_load_segment(pid {}): can't assign address {:#x}",
        //         (*p).p_pid,
        //         va
        //     );
        //     return -1;
        // }
        va += PAGESIZE;
    }

    // Note: First, implement vm crate
    // Ensure new memory mappings are active
    // set_pagetable((*p).p_pagetable);

    // Copy data from the source to the destination in memory
    // let dst = (*ph).p_va as *mut c_void;
    // memcpy(dst, src as *const c_void, (*ph).p_filesz as size_t);

    // Zero out remaining memory
    // let clear_start = (dst as usize + (*ph).p_filesz) as *mut c_void;
    // memset(clear_start, 0, ((*ph).p_memsz - (*ph).p_filesz) as size_t);

    // Restore the kernel pagetable
    // set_pagetable(core::ptr::null_mut());
    0 // Success
}

use crate::*;
use core::ffi::c_void;
use core::ffi::c_int;
use stdlib::my_assert;

// k-loader.c
//
//    Load a weensy application into memory from a RAM image.

extern "C" {
    pub fn c_panic(format: *const core::ffi::c_char, ...) -> !;
    pub fn assign_physical_page(addr: usize, owner: usize) -> i32;
    pub fn set_pagetable(pagetable: *mut x86_64_pagetable);
    pub fn virtual_memory_map(pagetable: *mut x86_64_pagetable, vaddr: usize, paddr: usize, size: usize, flags: u32) -> core::ffi::c_int;
    pub fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut ::std::os::raw::c_void;
    pub fn memset(s: *mut core::ffi::c_void, c: core::ffi::c_int, n: core::ffi::c_ulong) -> *mut core::ffi::c_void;
    
    pub static mut kernel_pagetable: *mut x86_64_pagetable;

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
    my_assert!(programnumber < n_programs);
    let ram_image = &RAMIMAGES[programnumber];
    let eh_ptr = ram_image.begin as *const u8;
    let eh: &ElfHeader = unsafe { 
        &*(eh_ptr as *const ElfHeader) 
    };
    my_assert!(eh.e_magic == ELF_MAGIC);

    // load each loadable program segment into memory
    let ph: &[ElfProgram] = unsafe {
        let program_header_ptr = (eh as *const ElfHeader as *const u8).add(eh.e_phoff as usize);
        let program_array = program_header_ptr as *const ElfProgram;
        core::slice::from_raw_parts(program_array, eh.e_phnum as usize)
    };
    
    for i in 0..eh.e_phnum as usize {
        if ph[i].p_type == ELF_PTYPE_LOAD {
            let pdata = unsafe {
                (eh as *const ElfHeader as *const u8).offset(ph[i].p_offset as isize)
            };

            if program_load_segment(p, &ph[i], pdata, allocator) < 0 {
                return -1;
            }
        }
    }

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
pub unsafe extern "C" fn program_load_segment(
    p: *mut Proc,
    ph: *const ElfProgram,
    src: *const u8,
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
    unsafe {
        while va < end_mem {
            if assign_physical_page(va as usize, (*p).p_pid as usize) < 0
                || virtual_memory_map((*p).p_pagetable, va as usize, va as usize, PAGESIZE as usize, (PTE_P | PTE_W | PTE_U) as u32) < 0
            {
                c_panic(
                    "(program_load_segment) can't assign address!".as_ptr() as *const core::ffi::c_char
                );
            }
            va += PAGESIZE;
        }
    }

    // ensure new memory mappings are active
    set_pagetable((*p).p_pagetable);

    // copy data from the source to the destination in memory
    let dst = (*ph).p_va as *mut c_void;
    memcpy(dst, src as *const c_void, (*ph).p_filesz as usize);
    let clear_start = (dst as usize + (*ph).p_filesz as usize) as *mut c_void;
    memset(clear_start, 0, ((*ph).p_memsz - (*ph).p_filesz) as u64);

    // eestore the kernel pagetable
    set_pagetable(kernel_pagetable);
    0 // Success
}

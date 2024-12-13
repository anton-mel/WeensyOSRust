#![allow(unused)]

use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;

// NOTE
// Read x86-64.h for some useful functions and macros relevant here!

extern "C" {
    pub fn c_panic(format: *const core::ffi::c_char, ...) -> !;
    pub fn default_int_handler();
}

#[no_mangle]
#[allow(non_upper_case_globals)]
static mut kernel_pagetable: *mut x86_64_pagetable = core::ptr::null_mut();

pub struct KernelPageTables {
    pub kernel_pagetables: [x86_64_pagetable; 5],
}

impl KernelPageTables {
    pub fn new() -> Self {
        KernelPageTables {
            kernel_pagetables: [
                x86_64_pagetable::new(),
                x86_64_pagetable::new(),
                x86_64_pagetable::new(),
                x86_64_pagetable::new(),
                x86_64_pagetable::new(),
            ],
        }
    }
    
    // virtual_memory_init
    //    Initialize the virtual memory system, including an initial page table
    //    `kernel_pagetable`.

    pub unsafe fn virtual_memory_init(
        &mut self,
    ) {
        kernel_pagetable = &mut self.kernel_pagetables[0];

        // connect the pagetable pages
        self.kernel_pagetables[0].entry[0] =
            (&self.kernel_pagetables[1] as *const _ as u64) | PTE_P | PTE_W | PTE_U;
        self.kernel_pagetables[1].entry[0] =
            (&self.kernel_pagetables[2] as *const _ as u64) | PTE_P | PTE_W | PTE_U;
        self.kernel_pagetables[2].entry[0] =
            (&self.kernel_pagetables[3] as *const _ as u64) | PTE_P | PTE_W | PTE_U;
        self.kernel_pagetables[2].entry[1] =
            (&self.kernel_pagetables[4] as *const _ as u64) | PTE_P | PTE_W | PTE_U;

        // identity map the page table
        self.virtual_memory_map(
            kernel_pagetable,
            0,
            0,
            MEMSIZE_PHYSICAL as usize,
            (PTE_P | PTE_W | PTE_U) as i32,
        );

        // Verify the identity mapping
        for addr in (0..MEMSIZE_PHYSICAL).step_by(PAGESIZE as usize) {
            let vmap = self.virtual_memory_lookup(kernel_pagetable, addr as usize);
            // this assert will probably fail initially!
            // have you implemented virtual_memory_map and lookup_l1pagetable ?
            if !(vmap.pa == addr as usize) {
                c_panic("(virtual_memory_init) identity mapping failed".as_ptr() as *const i8);
            }
            if !((vmap.perm & (PTE_P | PTE_W) as i32) == (PTE_P | PTE_W) as i32) {
                c_panic("(virtual_memory_init) (vmap.perm & (PTE_P | PTE_W)) == (PTE_P | PTE_W) failed".as_ptr() as *const i8);
            }
        }

        // set pointer to this pagetable in the CR3 register
        // set_pagetable also does several checks for a valid pagetable
        self.set_pagetable(kernel_pagetable);
    }

    // set_pagetable
    //    Change page directory. lcr3() is the hardware instruction;
    //    set_pagetable() additionally checks that important kernel procedures are
    //    mappable in `pagetable`, and calls panic() if they aren't.

    pub unsafe fn set_pagetable(
        &self, 
        pagetable: *mut x86_64_pagetable,
    ) {
        if (page_offset(pagetable as *const u8) != 0) { // must be page aligned
            c_panic("Pagetable must be page-aligned".as_ptr() as *const i8);
        }

        // Check for kernel space being mapped in the pagetable
        if self.virtual_memory_lookup(pagetable, default_int_handler as usize).pa != default_int_handler as usize {
            c_panic("default_int_handler is not mapped in the pagetable".as_ptr() as *const i8);
        }

        if self.virtual_memory_lookup(kernel_pagetable, pagetable as usize).pa != pagetable as usize {
            c_panic("Pagetable is not mapped in kernel_pagetable".as_ptr() as *const i8);
        }

        if self.virtual_memory_lookup(pagetable, kernel_pagetable as usize).pa != kernel_pagetable as usize {
            c_panic("kernel_pagetable is not mapped in the pagetable".as_ptr() as *const i8);
        }

        // if self.virtual_memory_lookup(pagetable, self.virtual_memory_map as usize).pa != self.virtual_memory_map as usize {
        //     c_panic("virtual_memory_map is not mapped in the pagetable".as_ptr() as *const i8);
        // }

        // Set the page table in the CR3 register
        lcr3(pagetable as usize);
    }

    // virtual_memory_map(pagetable, va, pa, sz, perm)
    //    Map virtual address range `[va, va+sz)` in `pagetable`.
    //    When `X >= 0 && X < sz`, the new pagetable will map virtual address
    //    `va+X` to physical address `pa+X` with permissions `perm`.
    //
    //    Precondition: `va`, `pa`, and `sz` must be multiples of PAGESIZE
    //    (4096).
    //
    //    Typically `perm` is a combination of `PTE_P` (the memory is Present),
    //    `PTE_W` (the memory is Writable), and `PTE_U` (the memory may be
    //    accessed by User applications). If `!(perm & PTE_P)`, `pa` is ignored.
    //
    //    Returns 0 if the map succeeds, -1 if it fails (because a required
    //    page table was not allocated).

    pub unsafe fn virtual_memory_map(
        &self,
        pagetable: *mut x86_64_pagetable, // Pointer to the page table
        va: usize,                        // Virtual address
        pa: usize,                        // Physical address
        sz: usize,                        // Size
        perm: i32,                        // Permissions
    ) -> i32 {
        // Function body will go here
        0 // Placeholder return value
    }    

    // lookup_l1pagetable(pagetable, va, perm)
    //    Helper function to find the last level of `va` in `pagetable`
    //
    //    Returns an x86_64_pagetable pointer to the last level pagetable
    //    if it exists and can be accessed with the given permissions
    //    Returns NULL otherwise

    pub unsafe fn lookup_l1pagetable(
        &self,
        pagetable: *mut x86_64_pagetable, // Pointer to the page table
        va: usize,                        // Virtual address
        perm: i32,                        // Permissions
    ) -> *mut x86_64_pagetable {
        // Function body will go here
        core::ptr::null_mut()
    }
    
    // virtual_memory_lookup(pagetable, va)
    //    Returns information about the mapping of the virtual address `va` in
    //    `pagetable`. The information is returned as a `vamapping` object.

    pub unsafe fn virtual_memory_lookup(
        &self,
        pagetable: *mut x86_64_pagetable, // Pointer to the page table
        va: usize,                        // Virtual address
    ) -> VAMapping {
        // Function body will go here
        VAMapping {
            pn: 0,
            pa: 0,
            perm: 0,
        }
    }
}

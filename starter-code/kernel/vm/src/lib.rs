#![allow(static_mut_refs)]

use bindings::bindings_x86_64::*;

pub mod vm;

use crate::vm::KernelPageTables;
static mut VM: Option<KernelPageTables> = None;


#[no_mangle]
pub unsafe extern "C" fn virtual_memory_init() {
    if VM.is_none() {
        VM = Some(KernelPageTables::new());
    }
    if let Some(vm) = &mut VM {
        vm.virtual_memory_init();
    }
}

#[no_mangle]
pub unsafe extern "C" fn set_pagetable(
    pagetable: *mut x86_64_pagetable,
) {
    if VM.is_none() {
        VM = Some(KernelPageTables::new());
    }
    if let Some(vm) = &mut VM {
        vm.set_pagetable(pagetable);
    }
}

#[no_mangle]
pub unsafe extern "C" fn virtual_memory_map(
    pagetable: *mut x86_64_pagetable, // Pointer to the page table
    va: usize,                        // Virtual address
    pa: usize,                        // Physical address
    sz: usize,                        // Size
    perm: i32,                        // Permissions
) -> i32 {
    if VM.is_none() {
        VM = Some(KernelPageTables::new());
    }
    if let Some(vm) = &mut VM {
        return vm.virtual_memory_map(
            pagetable,
            va, 
            pa,
            sz,
            perm,
        );
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn lookup_l1pagetable(
    pagetable: *mut x86_64_pagetable, // Pointer to the page table
    va: usize,                        // Virtual address
    perm: i32,                        // Permissions
) -> *mut x86_64_pagetable {
    if VM.is_none() {
        VM = Some(KernelPageTables::new());
    }
    if let Some(vm) = &mut VM {
        return vm.lookup_l1pagetable(
            pagetable, 
            va,
            perm,
        );
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn virtual_memory_lookup(
    pagetable: *mut x86_64_pagetable, // Pointer to the page table
    va: usize,                        // Virtual address
) -> VAMapping {
    if VM.is_none() {
        VM = Some(KernelPageTables::new());
    }
    if let Some(vm) = &mut VM {
        return vm.virtual_memory_lookup(
            pagetable, 
            va,
        );
    }
    VAMapping {
        pn: 0,
        pa: 0,
        perm: 0,
    }
}

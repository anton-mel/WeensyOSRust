#![allow(static_mut_refs)]

mod kernel;
mod process;
mod memshow;
mod ph_page_info;

use bindings::bindings_x86_64::*;

use crate::kernel::Kernel;
static mut KERNEL: Option<Kernel> = None;


#[no_mangle]
pub unsafe extern "C" fn kernel(command: *const u8) {
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }
    if let Some(kernel) = &mut KERNEL {
        kernel.kernel(command);
    }
}

#[no_mangle]
pub unsafe extern "C" fn exception(reg: &mut x86_64_registers) {
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }
    if let Some(kernel) = &mut KERNEL {
        kernel.exception(reg);
    }
}

#[no_mangle]
pub unsafe extern "C" fn assign_physical_page(addr: usize, owner: usize) -> i32 {
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }
    if let Some(kernel) = &mut KERNEL {
        return kernel.assign_physical_page(addr, owner);
    }
    -1
}

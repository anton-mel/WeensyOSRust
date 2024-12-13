#![allow(static_mut_refs)]
#![allow(unused)]

mod kernel;
mod process;
mod memshow;
mod ph_page_info;

use crate::kernel::Kernel;
use bindings::bindings_x86_64::*;
use core::ffi::c_char;
use core::ptr;

unsafe extern "C" {
    #[link(name = "vm")] fn virtual_memory_map(pagetable: *mut x86_64_pagetable, vaddr: usize, paddr: usize, size: usize, flags: u32) -> core::ffi::c_int;
    #[link(name = "k-hardware")] fn hardware_init();
    #[link(name = "bindings_lib")] fn console_clear();
    unsafe fn timer_init(hz: u32);
}

use core::sync::atomic::{AtomicBool, Ordering};
use core::sync::atomic::AtomicUsize;
use core::cell::UnsafeCell;
use core::{slice, str};

// Define a global `Kernel` instance
static mut KERNEL: Option<Kernel> = None;

// Custom spinlock using AtomicBool
static LOCK: AtomicBool = AtomicBool::new(false);


// Custom spin lock implementation
fn lock() {
    while LOCK.swap(true, Ordering::Acquire) {
        // Spin while the lock is acquired
    }
}

fn unlock() {
    LOCK.store(false, Ordering::Release);
}

#[no_mangle]
pub unsafe extern "C" fn kernel(command: *const u8) {
    lock(); // Acquire the lock before accessing the kernel
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }

    if let Some(kernel) = &mut KERNEL {
        let mut test_str: &'static str = "else";
        kernel.kernel(test_str);
    }
    unlock(); // Release the lock after use
}

// assign_physical_page(addr, owner)
// Allocates the page with physical address `addr` to the given owner.
// Fails if physical page `addr` was already allocated. Returns 0 on success and -1 on failure.

#[no_mangle]
pub unsafe extern "C" fn process_setup(pid: usize, program_number: usize) -> i32 {
    // lock();
    // if KERNEL.is_none() {
    //     KERNEL = Some(Kernel::new());
    // }

    // if let Some(kernel) = &mut KERNEL {
    //     kernel.process_setup(pid, program_number);
    // }
    // unlock();
    0
}

#[no_mangle]
pub unsafe extern "C" fn exception(reg: &mut x86_64_registers) {
    lock(); // Acquire the lock before accessing the kernel
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }

    if let Some(kernel) = &mut KERNEL {
        kernel.exception(reg);
    }
    unlock(); // Release the lock after use
}

#[no_mangle]
pub unsafe extern "C" fn assign_physical_page(addr: usize, owner: usize) -> i32 {
    let mut output = -1; // Default value or error code
    lock(); // Acquire the lock before accessing the kernel
    if KERNEL.is_none() {
        KERNEL = Some(Kernel::new());
    }

    if let Some(kernel) = &mut KERNEL {
        output = kernel.assign_physical_page(addr, owner);
    }

    unlock(); // Release the lock after use
    return output;
}

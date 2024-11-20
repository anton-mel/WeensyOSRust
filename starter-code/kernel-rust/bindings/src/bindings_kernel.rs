#![allow(unused)]

use crate::bindings_x86_64::PAGESIZE;

// Maximum number of processes
pub const NPROC: usize = 16;

// Kernel start address
pub const KERNEL_START_ADDR: u64 = 0x40000;
// Top of the kernel stack
pub const KERNEL_STACK_TOP: u64 = 0x80000;

// First application-accessible address
pub const PROC_START_ADDR: u64 = 0x100000;

// Physical memory size
pub const MEMSIZE_PHYSICAL: u64 = 0x200000;
// Number of physical pages
pub const NPAGES: u64 = MEMSIZE_PHYSICAL / PAGESIZE;

// Virtual memory size
pub const MEMSIZE_VIRTUAL: u64 = 0x300000;

// Hardware interrupt numbers
pub const INT_HARDWARE: u32 = 32;
pub const INT_TIMER: u32 = INT_HARDWARE + 0;

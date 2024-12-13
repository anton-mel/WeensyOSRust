// Bindigns to x86_64.h: Rust-C code to interface with x86 hardware and CPU.
//
//   Contents:
//   - Memory and interrupt constants.
//   - x86_registers: Used in process descriptors to store x86 registers.
//   - x86 functions: C function wrappers for useful x86 instructions.
//   - Hardware structures: C structures and constants for initializing
//     x86 hardware, including the interrupt descriptor table.

use core::arch::asm;

use crate::bindings_kernel::P_FREE;
use crate::bindings_kernel::Procstate;

pub type X86_64PageentryT = u64;
pub type ProcstateT = ::core::ffi::c_uint;
pub type PidT = ::core::ffi::c_int;

pub const PAGESIZE: u64 = 4096;
const PAGEOFFBITS: usize = 12;                     // # bits in page offset

pub fn page_number(ptr: *const u8) -> usize {
    (ptr as usize) >> PAGEOFFBITS
}

pub fn pte_addr(pageentry: usize) -> usize {
    pageentry & !0xFFF
}

// Page table entry flags
pub const PTE_FLAGS: X86_64PageentryT = 0xFFF;
// - Permission flags: define whether page is accessible
pub const PTE_P: X86_64PageentryT = 1;      // entry is Present
pub const PTE_W: X86_64PageentryT = 2;      // entry is Writeable
pub const PTE_U: X86_64PageentryT = 4;      // entry is User-accessible
// - Accessed flags: automatically turned on by processor
pub const PTE_A: X86_64PageentryT = 32;     // entry was Accessed (read/written)
pub const PTE_D: X86_64PageentryT = 64;     // entry was Dirtied (written)
pub const PTE_PS: X86_64PageentryT = 128;   // entry has a large Page Size
// - There are other flags too!

// Page fault error flags
// These bits are stored in x86_registers::reg_err after a page fault trap.
pub const PFERR_PRESENT: u8 = 0x1;   // Fault happened due to a protection violation (rather than due to a missing page)
pub const PFERR_WRITE: u8 = 0x2;     // Fault happened on a write
pub const PFERR_USER: u8 = 0x4;      // Fault happened in an application (user mode) (rather than kernel)

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug, Copy, Clone)]
pub struct x86_64_pagetable {
    pub entry: [X86_64PageentryT; 512usize],
}

unsafe impl Send for x86_64_pagetable {}
unsafe impl Sync for x86_64_pagetable {}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct x86_64_registers {
    pub reg_rax: u64,
    pub reg_rcx: u64,
    pub reg_rdx: u64,
    pub reg_rbx: u64,
    pub reg_rbp: u64,
    pub reg_rsi: u64,
    pub reg_rdi: u64,
    pub reg_r8: u64,
    pub reg_r9: u64,
    pub reg_r10: u64,
    pub reg_r11: u64,
    pub reg_r12: u64,
    pub reg_r13: u64,
    pub reg_r14: u64,
    pub reg_r15: u64,
    pub reg_fs: u64,
    pub reg_gs: u64,
    pub reg_intno: u64,
    pub reg_err: u64,
    pub reg_rip: u64,
    pub reg_cs: u16,
    pub reg_padding2: [u16; 3usize],
    pub reg_rflags: u64,
    pub reg_rsp: u64,
    pub reg_ss: u16,
    pub reg_padding3: [u16; 3usize],
}

impl Default for x86_64_registers {
    fn default() -> Self {
        x86_64_registers {
            reg_rax: 0,
            reg_rcx: 0,
            reg_rdx: 0,
            reg_rbx: 0,
            reg_rbp: 0,
            reg_rsi: 0,
            reg_rdi: 0,
            reg_r8: 0,
            reg_r9: 0,
            reg_r10: 0,
            reg_r11: 0,
            reg_r12: 0,
            reg_r13: 0,
            reg_r14: 0,
            reg_r15: 0,
            reg_fs: 0,
            reg_gs: 0,
            reg_intno: 0,
            reg_err: 0,
            reg_rip: 0,
            reg_cs: 0,
            reg_padding2: [0; 3],
            reg_rflags: 0,
            reg_rsp: 0,
            reg_ss: 0,
            reg_padding3: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Proc {
    pub p_pid: PidT,
    pub p_registers: x86_64_registers,
    pub p_state: ProcstateT,
    pub p_pagetable: *mut x86_64_pagetable,
    pub display_status: u8,
}

unsafe impl Send for Proc {}
unsafe impl Sync for Proc {}

impl Default for Proc {
    fn default() -> Self {
        Proc {
            p_pid: 0,
            p_registers: x86_64_registers::default(),
            p_state: P_FREE,
            p_pagetable: core::ptr::null_mut(),
            display_status: 0,
        }
    }
}

impl Proc {
    pub fn new(pid: PidT, state: Procstate) -> Self {
        let mut proc = Proc::default();
        proc.p_pid = pid;
        proc.p_state = state;
        proc
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct VAMapping {
    pub pn: core::ffi::c_int,
    pub pa: usize,
    pub perm: core::ffi::c_int,
}

// Interrupt numbers
pub const INT_DIVIDE: u32 = 0x0;        // Divide error
pub const INT_DEBUG: u32 = 0x1;         // Debug exception
pub const INT_BREAKPOINT: u32 = 0x3;    // Breakpoint
pub const INT_OVERFLOW: u32 = 0x4;      // Overflow
pub const INT_BOUNDS: u32 = 0x5;        // Bounds check
pub const INT_INVALIDOP: u32 = 0x6;     // Invalid opcode
pub const INT_DOUBLEFAULT: u32 = 0x8;   // Double fault
pub const INT_INVALIDTSS: u32 = 0xa;    // Invalid TSS
pub const INT_SEGMENT: u32 = 0xb;       // Segment not present
pub const INT_STACK: u32 = 0xc;         // Stack exception
pub const INT_GPF: u32 = 0xd;           // General protection fault
pub const INT_PAGEFAULT: u32 = 0xe;     // Page fault

pub const INT_SYS: u32 = 48;
pub const INT_SYS_PANIC: u32 = 48;
pub const INT_SYS_GETPID: u32 = 49;
pub const INT_SYS_YIELD: u32 = 50;
pub const INT_SYS_PAGE_ALLOC: u32 = 51;
pub const INT_SYS_FORK: u32 = 52;
pub const INT_SYS_EXIT: u32 = 53;
pub const INT_SYS_MAPPING: u32 = 54;
pub const INT_SYS_MEM_TOG: u32 = 56;
pub const INT_SYS_BRK: u32 = 57;
pub const INT_SYS_SBRK: u32 = 58;

#[inline(always)]
pub unsafe fn rcr2() -> u64 {
    let mut val: u64;
    asm!("movq %%cr2, {0}", out(reg) val);
    val
}

// x86_64.rs: Rust-C code to interface with x86 hardware and CPU.
//
//   Contents:
//   - Memory and interrupt constants.
//   - x86_registers: Used in process descriptors to store x86 registers.
//   - x86 functions: C function wrappers for useful x86 instructions.
//   - Hardware structures: C structures and constants for initializing
//     x86 hardware, including the interrupt descriptor table.

pub type X86_64PageentryT = u64;
pub type ProcstateT = ::core::ffi::c_uint;
pub type PidT = ::core::ffi::c_int;

pub const PAGESIZE: u64 = 4096;
const PAGEOFFBITS: usize = 12;                     // # bits in page offset

pub fn page_number(ptr: *const u8) -> usize {
    (ptr as usize) >> PAGEOFFBITS
}

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

#[derive(Debug)]
pub struct Proc {
    pub p_pid: PidT,
    pub p_registers: x86_64_registers,
    pub p_state: ProcstateT,
    pub p_pagetable: Option<Box<x86_64_pagetable>>,
    pub display_status: u8,
}

impl Default for Proc {
    fn default() -> Self {
        Proc {
            p_pid: 0,
            p_registers: x86_64_registers::default(),
            p_state: 0,
            p_pagetable: Some(Box::new(x86_64_pagetable {
                entry: [0; 512],
            })),
            display_status: 0,
        }
    }
}

impl Proc {
    pub fn new(pid: PidT) -> Self {
        let mut proc = Proc::default();
        proc.p_pid = pid;
        proc
    }
}

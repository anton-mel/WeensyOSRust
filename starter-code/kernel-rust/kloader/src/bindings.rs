#![allow(non_camel_case_types)]

pub type procstate = ::core::ffi::c_uint;
pub use self::procstate as procstate_t;
pub const ELF_MAGIC: u32 = 1179403647;
pub type pid_t = ::core::ffi::c_int;
pub type x86_64_pageentry_t = u64;
pub const ELF_PTYPE_LOAD: u32 = 1;

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug, Copy, Clone)]
pub struct x86_64_pagetable {
    pub entry: [x86_64_pageentry_t; 512usize],
}

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

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct proc_ {
    pub p_pid: pid_t,
    pub p_registers: x86_64_registers,
    pub p_state: procstate_t,
    pub p_pagetable: *mut x86_64_pagetable,
    pub display_status: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct elf_program {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_va: u64,
    pub p_pa: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct elf_header {
    pub e_magic: u32,
    pub e_elf: [u8; 12usize],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}


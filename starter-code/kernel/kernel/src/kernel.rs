use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;
use bindings::bindings_lib::*;

use crate::memshow::memshow_physical;
use crate::memshow::memshow_virtual_animate;

use crate::process::ProcessTable;
use crate::ph_page_info::PhysicalPageInfoTable;
use crate::ph_page_info::PageOwner;

use stdlib::*;

use core::sync::atomic::{
    AtomicU32,
    AtomicU8,
    Ordering
};

// kernel.c
//
//    This is the kernel.

unsafe extern "C" {
    fn set_pagetable(pagetable: *mut x86_64_pagetable);
    fn virtual_memory_map(pagetable: *mut x86_64_pagetable, vaddr: usize, paddr: usize, size: usize, flags: u32) -> core::ffi::c_int;
    fn virtual_memory_lookup(pagetable: *mut x86_64_pagetable, va: usize) -> VAMapping;
    fn c_panic(format: *const core::ffi::c_char, ...) -> !;
    static kernel_pagetable: *mut x86_64_pagetable;
}

// INITIAL PHYSICAL MEMORY LAYOUT
//
//  +-------------- Base Memory --------------+
//  v                                         v
// +-----+--------------------+----------------+--------------------+---------/
// |     | Kernel      Kernel |       :    I/O | App 1        App 1 | App 2
// |     | Code + Data  Stack |  ...  : Memory | Code + Data  Stack | Code ...
// +-----+--------------------+----------------+--------------------+---------/
// 0  0x40000              0x80000 0xA0000 0x100000             0x140000
//                                             ^
//                                             | \___ PROC_SIZE ___/
//                                      PROC_START_ADDR

const PROC_SIZE: usize = 0x40000; // initial state only

const HZ: u32 = 100;                // timer interrupt frequency (interrupts/sec)
static TICKS: AtomicU32 =           // # timer interrupts so far
    AtomicU32::new(0);              // AtomicU32 for thread-safe mutable static

static DISP_GLOBAL: AtomicU8 =      // global flag to display memviewer
    AtomicU8::new(1);               // AtomicU8 for thread-safe mutable static

pub struct Kernel {
    proc_table: ProcessTable,
    pageinfo_table: PhysicalPageInfoTable,
}

impl Kernel {
    pub fn new() -> Self {
        Kernel {
            proc_table: ProcessTable::new(),
            pageinfo_table: PhysicalPageInfoTable::new(),
        }
    }

    // kernel(command)
    //    Initialize the hardware and processes and start running. The `command`
    //    string is an optional string passed from the boot loader.

    pub fn kernel(&mut self, command: *const u8) {
        unsafe extern "C" {
            fn hardware_init();
            fn console_clear();
            fn timer_init(hz: u32);
            fn strcmp(
                a: *const core::ffi::c_char,
                b: *const core::ffi::c_char,
            ) -> core::ffi::c_int;
        }

        unsafe{
            hardware_init();
            // Borrows PhysicalPageInfoTable
            self.pageinfo_table.pageinfo_init();
            console_clear();
            timer_init(HZ);

            if !command.is_null() && strcmp(command as *const i8, "fork".as_ptr() as *const i8) == 0 {
                self.process_setup(1, 4);
            } else if !command.is_null() && strcmp(command as *const i8, "forkexit".as_ptr() as *const i8) == 0 {
                self.process_setup(1, 5);
            } else if !command.is_null() && strcmp(command as *const i8, "test".as_ptr() as *const i8) == 0 {
                self.process_setup(1, 6);
            } else if !command.is_null() && strcmp(command as *const i8, "test2".as_ptr() as *const i8) == 0 {
                for i in 1..=2 {
                    self.process_setup(i, 6);
                }
            } else {
                for i in 1..=4 {
                    self.process_setup(i, i - 1);
                }
            }

            // Switch to the first process using run()
            self.proc_table.run(1);
        }
    }

    // process_setup(pid, program_number)
    //    Load application program `program_number` as process number `pid`.
    //    This loads the application's code and data into memory, sets its
    //    %rip and %rsp, gives it a stack page, and marks it as runnable.

    pub fn process_setup(&mut self, pid: usize, program_number: usize) {
        let mut p = self.proc_table.process_setup(pid, program_number);
        unsafe { // increase refcount since kernel_pagetable was used
            let pn = page_number(kernel_pagetable as *const u8);
            if let Some(page_info) = self.pageinfo_table.get_page_info_ref(pn) {
                page_info.refcount += 1;
            }
        }
        p.p_registers.reg_rsp = PROC_START_ADDR + (PROC_SIZE * pid) as u64;
        let stack_page = p.p_registers.reg_rsp - PAGESIZE;
        self.assign_physical_page(stack_page as usize, pid);
        unsafe {
            virtual_memory_map(
                p.p_pagetable, 
                stack_page as usize,
                stack_page as usize,
                PAGESIZE as usize,
                (PTE_P | PTE_W | PTE_U) as u32,
            );
        }
        p.p_state = P_RUNNABLE;
    }

    // assign_physical_page(addr, owner)
    //    Allocates the page with physical address `addr` to the given owner.
    //    Fails if physical page `addr` was already allocated. Returns 0 on
    //    success and -1 on failure. Used by the program loader.

    pub fn assign_physical_page(&mut self, addr: usize, owner: usize) -> i32 {
        let pn = page_number(addr as *const u8);
        if (addr & 0xFFF) != 0
            || pn >= self.pageinfo_table.pageinfo.len()
            || self.pageinfo_table.pageinfo[pn].refcount != 0 {
           return -1;
        }
    
        self.pageinfo_table.pageinfo[pn].owner = owner as i8;
        self.pageinfo_table.pageinfo[pn].refcount = 1;
        0
    }

    // check_page_table_mappings
    //    Check operating system invariants about kernel mappings for page
    //    table `pt`. Panic if any of the invariants are false.

    pub fn check_page_table_mappings(&self, pt: *mut x86_64_pagetable) {
        extern "C" {
            static mut start_data: u8;
            static mut end: u8;
            fn console_printf(
                cpos: i32,
                color: i32,
                format: *const u8,
                ...
            ) -> i32;
        }

        unsafe {
            let start_data_addr = &start_data as *const u8 as u64;
            let end_addr = &end as *const u8 as u64;

            for va in (KERNEL_START_ADDR..end_addr).step_by(PAGESIZE as usize) {
                let vam = virtual_memory_lookup(pt, va as usize);
                let vam_pa = vam.pa;
                let vam_perm = vam.perm;

                if vam_pa != va as usize {
                    let fmt = b"{:p} vs {:p}\0" as *const u8;
                    console_printf(22, 0, 0xC000 as *const u8,
                        fmt, va as *const u8, vam_pa as *const u8);
                }
                
                // FIX: my_assert! fails on multiple definitions
                if !(vam_pa == va as usize) {
                    c_panic("Assertion failed: vam_pa == va as usize".as_ptr() as *const i8);
                }
                if va >= start_data_addr {
                    if !(vam_perm & PTE_W as i32 != 0) {
                        c_panic("Assertion failed: vam_perm & PTE_W as i32 != 0".as_ptr() as *const i8);
                    }
                }
            }

            let kstack = KERNEL_STACK_TOP - PAGESIZE;
            let vam = virtual_memory_lookup(pt, kstack as usize);
            let vam_pa = vam.pa;
            let vam_perm = vam.perm;

            // FIX: my_assert! fails on multiple definitions
            if !(vam_pa == kstack as usize) {
                c_panic("Assertion failed: vam_pa == kstack as usize".as_ptr() as *const i8);
            }
            if !(vam_perm & PTE_W as i32 != 0) {
                c_panic("Assertion failed: vam_perm & PTE_W as i32 != 0".as_ptr() as *const i8);
            }
        }
    }

    // check_page_table_ownership
    //    Check operating system invariants about ownership and reference
    //    counts for page table `pt`. Panic if any of the invariants are false.

    #[allow(unused)]
    pub fn check_page_table_ownership(&self, pt: *mut x86_64_pagetable, pid: i32) {
        unsafe {
            let mut owner = pid;
            let mut expected_refcount = 1;

            if pt == kernel_pagetable {
                owner = PageOwner::PoKernel as i32;
                for proc in self.proc_table.processes.iter() {
                    if proc.p_state != P_FREE && proc.p_pagetable == kernel_pagetable {
                        expected_refcount += 1;
                    }
                }
            }

            self.check_page_table_ownership_level(pt, 0, owner, expected_refcount);
        }
    }

    #[allow(unused)]
    pub fn check_page_table_ownership_level(&self, pt: *mut x86_64_pagetable, level: usize, owner: i32, refcount: u32) {
        unsafe {
            let page_number = (pt as usize) / PAGESIZE as usize;
            my_assert!(page_number < NPAGES as usize);
            my_assert!(self.pageinfo_table.pageinfo[page_number].owner == owner as i8);
            my_assert!(self.pageinfo_table.pageinfo[page_number].refcount == refcount as i8);

            if level < 3 {
                for &entry in &(*pt).entry {
                    if entry != 0 {
                        let next_pt = (entry & !0xFFF) as *mut x86_64_pagetable;
                        self.check_page_table_ownership_level(next_pt, level + 1, owner, 1);
                    }
                }
            }
        }
    }

    // check_virtual_memory
    //    Check operating system invariants about virtual memory. Panic if any
    //    of the invariants are false.

    pub fn check_virtual_memory(&self) {
        unsafe {
            my_assert!(self.proc_table.processes[0].p_state == P_FREE);

            self.check_page_table_mappings(kernel_pagetable);
            // self.check_page_table_ownership(kernel_pagetable, -1);

            // for proc in self.proc_table.processes.iter() {
            //     if proc.p_state != P_FREE && proc.p_pagetable != kernel_pagetable {
            //         self.check_page_table_mappings(proc.p_pagetable);
            //         self.check_page_table_ownership(proc.p_pagetable, proc.p_pagetable as i32);
            //     }
            // }

            // for (pn, page) in self.pageinfo_table.pageinfo.iter().enumerate() {
            //     if page.refcount > 0 && page.owner >= 0 {
            //         my_assert!(self.proc_table.processes[page.owner as usize].p_state != P_FREE);
            //     }
            // }
        }
    }

    // exception(reg)
    //    Exception handler (for interrupts, traps, and faults).
    //
    //    The register values from exception time are stored in `reg`.
    //    The processor responds to an exception by saving application state on
    //    the kernel's stack, then jumping to kernel assembly code (in
    //    k-exception.S). That code saves more registers on the kernel's stack,
    //    then calls exception().
    //
    //    Note that hardware interrupts are disabled whenever the kernel is running.

    #[allow(non_snake_case)]
    pub fn exception(&mut self, reg: &mut x86_64_registers) {
        unsafe extern "C" {
            fn check_keyboard() -> core::ffi::c_int;
            fn console_show_cursor(cpos: core::ffi::c_int);
            fn default_exception(p: *mut Proc);
            fn memcpy(
                dst: *mut core::ffi::c_void,
                src: *const core::ffi::c_void,
                n: usize,
            ) -> *mut ::std::os::raw::c_void;
            fn console_printf(
                cpos: i32,
                color: i32,
                format: *const u8,
                ...
            ) -> i32;
        }
        
        // Copy the saved registers into the `current` process descriptor
        // and always use the kernel's page table.
        self.proc_table.exception(reg);
        unsafe { set_pagetable(kernel_pagetable); }

        // It can be useful to log events using `log_printf`.
        // Events logged this way are stored in the host's `log.txt` file.
        /*log_printf("proc %d: exception %d\n", current->p_pid, reg->reg_intno);*/

        // Show the current cursor location and memory state
        // (unless this is a kernel fault).
        unsafe { console_show_cursor(cursorpos); }
        if (reg.reg_intno != INT_PAGEFAULT as u64 && reg.reg_intno != INT_GPF as u64) // no error due to pagefault or general fault
            || (reg.reg_err & PFERR_USER as u64) != 0 // pagefault error in user mode
        {
            self.check_virtual_memory();
            if DISP_GLOBAL.load(Ordering::SeqCst) != 0 {
                unsafe{ memshow_physical(); }
                unsafe{ memshow_virtual_animate(); }
            }
        }

        // If Control-C was typed, exit the virtual machine.
        unsafe { check_keyboard(); }

        let curr_proc = self.proc_table.get_current_process();
        // Handle the exception based on the interrupt number.
        match reg.reg_intno as u32 {
            INT_SYS_PANIC => {
                // rdi stores pointer for msg string
                let addr = curr_proc.p_registers.reg_rdi;
                if addr == 0 {
                    unsafe {
                        c_panic("(exception) current process has not been set yet".as_ptr() as *const core::ffi::c_char);
                    }
                } else {
                    unsafe {
                        let map = virtual_memory_lookup(curr_proc.p_pagetable, addr as usize);
                        let mut msg = [0u8; 160];
                        memcpy(
                            &mut msg as *mut [u8; 160] as *mut core::ffi::c_void, 
                            map.pa as *const core::ffi::c_void, 
                            160
                        );
                        c_panic(msg.as_ptr() as *const i8);
                        /* will not be reached */
                    }
                }
            }
            INT_SYS_GETPID => {
                self.proc_table.set_register_rax(curr_proc.p_pid as u64);              
            }
            INT_SYS_YIELD => {
                self.proc_table.schedule();
                /* will not be reached */
            }
            INT_SYS_PAGE_ALLOC => {
                let addr = curr_proc.p_registers.reg_rdi;
                let r = self.assign_physical_page(
                    addr as usize, 
                    curr_proc.p_pid as usize,
                );
                if r >= 0 {
                    unsafe { 
                        virtual_memory_map(
                            curr_proc.p_pagetable, 
                            addr as usize,
                            addr as usize,
                            PAGESIZE as usize,
                            (PTE_P | PTE_W | PTE_U) as u32,
                        );
                    }
                }
                self.proc_table.set_register_rax(r as u64);
            }
            INT_SYS_MAPPING => {
                unsafe {
                    let current = self.proc_table.get_current_process_mut();
                    syscall_mapping(&mut *current);
                }
            }
            INT_SYS_MEM_TOG => {
                unsafe {
                    let current = self.proc_table.get_current_process_mut();
                    syscall_mem_tog(&mut *current);
                }
            }
            INT_TIMER => {
                TICKS.fetch_add(1, Ordering::SeqCst);
                self.proc_table.schedule();
                /* will not be reached */
            }
            INT_PAGEFAULT => {
                let current = self.proc_table.get_current_process_mut();
                // Analyze faulting address and access type.
                // let addr = unsafe { rcr2() };
                // let operation = if reg.reg_err & PFERR_WRITE as u64 != 0 { "write" } else { "read" };
                // let problem = if reg.reg_err & PFERR_PRESENT as u64 != 0 { "protection problem" } else { "missing page" };

                if reg.reg_err & PFERR_USER as u64 == 0 {
                    unsafe {
                        c_panic("Kernel page fault!".as_ptr() as *const core::ffi::c_char);
                    }
                }
                unsafe {
                    console_printf(
                        cpos!(24, 0), 
                        0x0C00, 
                        "Process page faule!".as_ptr() as *const u8,
                    );
                }
                current.p_state = P_BROKEN;
            }
            _ => {
                unsafe {
                    let current = self.proc_table.get_current_process_mut();
                    default_exception(&mut *current);
                    /* will not be reached */
                }
            }
        }

        // Return to the current process (or run something else).
        if curr_proc.p_state == P_RUNNABLE {
            self.proc_table.run(curr_proc.p_pid as usize);
        } else {
            self.proc_table.schedule();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_mapping(p: &mut Proc) {
    extern "C" {
        fn memcpy(
            dst: *mut core::ffi::c_void,
            src: *const core::ffi::c_void,
            n: usize,
        ) -> *mut ::std::os::raw::c_void;
    }
    
    let mapping_ptr = p.p_registers.reg_rdi;
    let ptr = p.p_registers.reg_rsi;

    // convert to physical address so the kernel can write to it
    let map = virtual_memory_lookup(p.p_pagetable, mapping_ptr as usize);

    // check for write access
    if (map.perm & (PTE_W | PTE_U) as i32) != (PTE_W | PTE_U) as i32 {
        return;
    }
    let endaddr = mapping_ptr + size_of::<VAMapping>() as u64 - 1;
    // check for write access for the end address
    let end_map = virtual_memory_lookup(p.p_pagetable, endaddr as usize);
    if (end_map.perm & (PTE_W | PTE_P) as i32) != (PTE_W | PTE_P) as i32 {
        return;
    }
    // find the actual mapping now
    let ptr_lookup = virtual_memory_lookup(p.p_pagetable, ptr as usize);
    unsafe {
        let dest = map.pa as *mut VAMapping;
        let src = &ptr_lookup as *const VAMapping as *const core::ffi::c_void;
        let size = core::mem::size_of::<VAMapping>();
        memcpy(dest as *mut core::ffi::c_void, src, size);
    }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_mem_tog(process: &mut Proc) {
    let p = process.p_registers.reg_rdi as PidT;

    if p == 0 {
        DISP_GLOBAL.fetch_xor(1, Ordering::SeqCst);
    } else {
        if p < 0 || p > NPROC as i32 || p != process.p_pid {
            return;
        }
        process.display_status = !process.display_status;
    }
}

use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;

use crate::process::ProcessTable;
use crate::ph_page_info::PhysicalPageInfoTable;

// kernel.c
//
//    This is the kernel.

unsafe extern "C" {
    #[link(name = "vm")] fn virtual_memory_map(pagetable: *mut x86_64_pagetable, vaddr: usize, paddr: usize, size: usize, flags: u32);
    #[link(name = "vm")] fn virtual_memory_lookup(pagetable: *mut x86_64_pagetable, va: usize) -> VAMapping;
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
static mut TICKS: u32 = 0;          // # timer interrupts so far

static mut DISP_GLOBAL: u8 = 1;     // global flag to display memviewer


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

    pub fn kernel(&mut self, command: &str) {
        unsafe extern "C" {
            #[link(name = "k-hardware")] fn hardware_init();
            #[link(name = "lib")] fn console_clear();
            fn timer_init(hz: u32);
        }

        unsafe{
            hardware_init();
            // Borrows PhysicalPageInfoTable
            self.pageinfo_table.pageinfo_init();
            console_clear();
            timer_init(HZ);

            match command {
                "fork" => self.process_setup(1, 4),
                "forkexit" => self.process_setup(1, 5),
                "test" => self.process_setup(1, 6),
                "test2" => {
                    for i in 1..=2 {
                        self.process_setup(i, 6);
                    }
                }
                _ => {
                    for i in 1..=4 {
                        self.process_setup(i, i - 1);
                    }
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
            // pageinfo[PAGENUMBER(kernel_pagetable);].refcount += 1;
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
        unsafe {
            DISP_GLOBAL = !DISP_GLOBAL;
        }
    } else {
        if p < 0 || p > NPROC as i32 || p != process.p_pid {
            return;
        }
        process.display_status = !process.display_status;
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

#[no_mangle]
pub unsafe extern "C" fn exception(reg: &mut x86_64_registers) {
    // unsafe {
    //     if let Some(current) = CURRENT.as_mut() {
    //         current.p_registers = *reg;
    //         set_pagetable(kernel_pagetable);
    //         console_show_cursor(cursorpos);

    //         if reg.reg_intno != INT_PAGEFAULT && reg.reg_intno != INT_GPF
    //             || (reg.reg_err & PFERR_USER) != 0
    //         {
    //             check_virtual_memory();
    //             if DISP_GLOBAL != 0 {
    //                 memshow_physical();
    //                 memshow_virtual_animate();
    //             }
    //         }

    //         check_keyboard();

    //         // Handling different exceptions
    //         match reg.reg_intno {
    //             INT_SYS_PANIC => {
    //                 let msg_addr = current.p_registers.reg_rdi;
    //                 if msg_addr == 0 {
    //                     panic!("Panic triggered: NULL message");
    //                 }
    //                 let map = virtual_memory_lookup(current.p_pagetable, msg_addr);
    //                 let mut msg = [0u8; 160];
    //                 memcpy(&mut msg, map.pa as *const u8, 160);
    //                 panic!("{}", String::from_utf8_lossy(&msg));
    //             }
    //             INT_SYS_GETPID => {
    //                 current.p_registers.reg_rax = current.p_pid;
    //             }
    //             INT_SYS_YIELD => schedule(),
    //             INT_SYS_PAGE_ALLOC => {
    //                 let addr = current.p_registers.reg_rdi;
    //                 if assign_physical_page(addr, current.p_pid) >= 0 {
    //                     virtual_memory_map(
    //                         current.p_pagetable,
    //                         addr,
    //                         addr,
    //                         PAGESIZE,
    //                         PTE_P | PTE_W | PTE_U,
    //                     );
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    // }
}


// schedule
//    Pick the next process to run and then run it.
//    If there are no runnable processes, spins forever.

#[no_mangle]
pub unsafe extern "C" fn schedule() {
    // let mut pid: usize;
    // unsafe {
    //     pid = (*CURRENT.load(Ordering::SeqCst)).p_pid;
    //     loop {
    //         pid = (pid + 1) % NPROC;
    //         if PROCESSES[pid].p_state == ProcessState::P_RUNNABLE {
    //             run(&mut PROCESSES[pid]);
    //         }
    //         check_keyboard();
    //     }
    // }
}


#[no_mangle]
pub unsafe extern "C" fn run(p: &mut Proc) {
    // assert_eq!(p.p_state, ProcessState::P_RUNNABLE);
    // unsafe {
    //     CURRENT.store(p, Ordering::SeqCst);
    // }
    // set_pagetable(p.p_pagetable);
    // exception_return(&p.p_registers);

    loop {
        // Spinloop
        // should never get here
    }
}


// pageinfo_init
//    Initialize the `pageinfo[]` array.

#[no_mangle]
pub unsafe extern "C" fn pageinfo_init() {
    
}


// check_page_table_mappings
//    Check operating system invariants about kernel mappings for page
//    table `pt`. Panic if any of the invariants are false.

#[no_mangle]
pub unsafe extern "C" fn check_page_table_mappings(pt: *mut x86_64_pagetable) {
    // extern "C" {
    //     static mut start_data: u8;
    //     static mut end: u8;
    // }

    // unsafe {
    //     let start_data = &start_data as *const u8 as usize;
    //     let end = &end as *const u8 as usize;

    //     for va in (KERNEL_START_ADDR..end).step_by(PAGESIZE) {
    //         let vam = virtual_memory_lookup(pt, va);
    //         if vam.pa != va {
    //             console_printf(22, 0, 0xC000, "{:p} vs {:p}", va as *const u8, vam.pa as *const u8);
    //         }
    //         assert_eq!(vam.pa, va);
    //         if va >= start_data {
    //             assert!(vam.perm & PTE_W != 0);
    //         }
    //     }

    //     let kstack = KERNEL_STACK_TOP - PAGESIZE;
    //     let vam = virtual_memory_lookup(pt, kstack);
    //     assert_eq!(vam.pa, kstack);
    //     assert!(vam.perm & PTE_W != 0);
    // }
}


// check_page_table_ownership
//    Check operating system invariants about ownership and reference
//    counts for page table `pt`. Panic if any of the invariants are false.

#[no_mangle]
pub unsafe extern "C" fn check_page_table_ownership(pt: *mut x86_64_pagetable, pid: i32) {
    // unsafe {
    //     let mut owner = pid;
    //     let mut expected_refcount = 1;

    //     if pt == KERNEL_PAGETABLE {
    //         owner = PO_KERNEL;
    //         for proc in PROCESSES.iter() {
    //             if proc.p_state != ProcessState::P_FREE && proc.p_pagetable == KERNEL_PAGETABLE {
    //                 expected_refcount += 1;
    //             }
    //         }
    //     }

    //     check_page_table_ownership_level(pt, 0, owner, expected_refcount);
    // }
}

#[no_mangle]
pub unsafe extern "C" fn check_page_table_ownership_level(pt: *mut x86_64_pagetable, level: usize, owner: i32, refcount: u32) {
    // unsafe {
    //     let page_number = (pt as usize) / PAGESIZE;
    //     assert!(page_number < NPAGES);
    //     assert_eq!(PAGEINFO[page_number].owner, owner);
    //     assert_eq!(PAGEINFO[page_number].refcount, refcount);

    //     if level < 3 {
    //         for &entry in &(*pt).entry {
    //             if entry != 0 {
    //                 let next_pt = (entry & !0xFFF) as *mut PageTable;
    //                 check_page_table_ownership_level(next_pt, level + 1, owner, 1);
    //             }
    //         }
    //     }
    // }
}


// check_virtual_memory
//    Check operating system invariants about virtual memory. Panic if any
//    of the invariants are false.

#[no_mangle]
pub unsafe extern "C" fn check_virtual_memory() {
    // unsafe {
    //     assert_eq!(PROCESSES[0].p_state, ProcessState::P_FREE);

    //     check_page_table_mappings(KERNEL_PAGETABLE);
    //     check_page_table_ownership(KERNEL_PAGETABLE, -1);

    //     for proc in PROCESSES.iter() {
    //         if proc.p_state != ProcessState::P_FREE && proc.p_pagetable != KERNEL_PAGETABLE {
    //             check_page_table_mappings(proc.p_pagetable);
    //             check_page_table_ownership(proc.p_pagetable, proc.p_pagetable as i32);
    //         }
    //     }

    //     for (pn, page) in PAGEINFO.iter().enumerate() {
    //         if page.refcount > 0 && page.owner >= 0 {
    //             assert_ne!(PROCESSES[page.owner as usize].p_state, ProcessState::P_FREE);
    //         }
    //     }
    // }
}

// memshow_physical
//    Draw a picture of physical memory on the CGA console.

const MEMSTATE_COLORS: [u16; 19] = [
    b'K' as u16 | 0x0D00, b'R' as u16 | 0x0700, b'.' as u16 | 0x0700, b'1' as u16 | 0x0C00,
    b'2' as u16 | 0x0A00, b'3' as u16 | 0x0900, b'4' as u16 | 0x0E00, b'5' as u16 | 0x0F00,
    b'6' as u16 | 0x0C00, b'7' as u16 | 0x0A00, b'8' as u16 | 0x0900, b'9' as u16 | 0x0E00,
    b'A' as u16 | 0x0F00, b'B' as u16 | 0x0C00, b'C' as u16 | 0x0A00, b'D' as u16 | 0x0900,
    b'E' as u16 | 0x0E00, b'F' as u16 | 0x0F00, b'S' as u16,
];
const SHARED_COLOR: u16 = MEMSTATE_COLORS[18];

#[no_mangle]
pub unsafe extern "C" fn memshow_physical() {
    // console_printf(CPOS(0, 32), 0x0F00, "PHYSICAL MEMORY");
    // for pn in 0..PAGENUMBER(MEMSIZE_PHYSICAL) {
    //     if pn % 64 == 0 {
    //         console_printf(CPOS(1 + pn / 64, 3), 0x0F00, "0x{:06X} ", pn << 12);
    //     }

    //     let mut owner = pageinfo[pn].owner;
    //     if pageinfo[pn].refcount == 0 {
    //         owner = PO_FREE;
    //     }
    //     let mut color = MEMSTATE_COLORS[(owner - PO_KERNEL) as usize];

    //     // Apply darker color for shared pages
    //     if pageinfo[pn].refcount > 1 && pn != PAGENUMBER(CONSOLE_ADDR) {
    //         #[cfg(feature = "shared")]
    //         {
    //             color = SHARED_COLOR | 0x0F00;
    //         }
    //         #[cfg(not(feature = "shared"))]
    //         {
    //             color &= 0x77FF;
    //         }
    //     }

    //     console[CPOS(1 + pn / 64, 12 + pn % 64)] = color;
    // }
}


// memshow_virtual(pagetable, name)
//    Draw a picture of the virtual memory map `pagetable` (named `name`) on
//    the CGA console.

#[no_mangle]
pub unsafe extern "C" fn memshow_virtual(pagetable: &x86_64_pagetable, name: *const u8) {
    // assert_eq!(pagetable as *const _ as usize, PTE_ADDR(pagetable as *const _ as usize));

    // console_printf(CPOS(10, 26), 0x0F00, "VIRTUAL ADDRESS SPACE FOR {}", name);
    // for va in (0..MEMSIZE_VIRTUAL).step_by(PAGESIZE) {
    //     let vam = virtual_memory_lookup(pagetable, va);
    //     let color = if vam.pn < 0 {
    //         b' ' as u16
    //     } else {
    //         assert!(vam.pa < MEMSIZE_PHYSICAL);
    //         let mut owner = pageinfo[vam.pn].owner;
    //         if pageinfo[vam.pn].refcount == 0 {
    //             owner = PO_FREE;
    //         }
    //         let mut color = MEMSTATE_COLORS[(owner - PO_KERNEL) as usize];

    //         // Apply reverse video for user-accessible pages
    //         if vam.perm & PTE_U != 0 {
    //             color = ((color & 0x0F00) << 4) | ((color & 0xF000) >> 4) | (color & 0x00FF);
    //         }

    //         // Apply darker color for shared pages
    //         if pageinfo[vam.pn].refcount > 1 && va != CONSOLE_ADDR {
    //             #[cfg(feature = "shared")]
    //             {
    //                 color = SHARED_COLOR | (color & 0xF000);
    //                 if vam.perm & PTE_U == 0 {
    //                     color |= 0x0F00;
    //                 }
    //             }
    //             #[cfg(not(feature = "shared"))]
    //             {
    //                 color &= 0x77FF;
    //             }
    //         }
    //         color
    //     };

    //     let pn = PAGENUMBER(va);
    //     if pn % 64 == 0 {
    //         console_printf(CPOS(11 + pn / 64, 3), 0x0F00, "0x{:06X} ", va);
    //     }
    //     console[CPOS(11 + pn / 64, 12 + pn % 64)] = color;
    // }
}


// memshow_virtual_animate
//    Draw a picture of process virtual memory maps on the CGA console.
//    Starts with process 1, then switches to a new process every 0.25 sec.

#[no_mangle]
pub unsafe extern "C" fn memshow_virtual_animate() {
    // static mut LAST_TICKS: u32 = 0;
    // static mut SHOWING: usize = 1;

    // unsafe {
    //     if LAST_TICKS == 0 || ticks - LAST_TICKS >= HZ / 2 {
    //         LAST_TICKS = ticks;
    //         SHOWING += 1;
    //     }

    //     while SHOWING <= 2 * NPROC
    //         && (processes[SHOWING % NPROC].p_state == P_FREE
    //             || processes[SHOWING % NPROC].display_status == 0)
    //     {
    //         SHOWING += 1;
    //     }
    //     SHOWING %= NPROC;

    //     if processes[SHOWING].p_state != P_FREE {
    //         let name = format!("{} ", SHOWING);
    //         memshow_virtual(&processes[SHOWING].p_pagetable, &name);
    //     }
    // }
}

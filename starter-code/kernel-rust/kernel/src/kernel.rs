use crate::bindings_x86_64::*;
use crate::bindings_kernel::*;

#[link(name = "kloader")]
extern "C" {
    fn program_load();
}

use once_cell::sync::OnceCell;
use lazy_static::lazy_static;
use alloc::vec::Vec;
use spin::Mutex;
use alloc::vec;


// kernel.c
//
//    This is the kernel.

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

const PROC_SIZE: usize = 0x40000;                                   // initial state only

static PROCESSES: OnceCell<Mutex<[Proc; 16]>> = OnceCell::new();    // array of process descriptors
                                                                    // Note that `processes[0]` is never used.  
static mut CURRENT: *mut Proc = core::ptr::null_mut();              // pointer to currently executing proc

const HZ: u32 = 100;                                                // timer interrupt frequency (interrupts/sec)
static mut TICKS: u32 = 0;                                          // # timer interrupts so far

static mut DISP_GLOBAL: u8 = 1;                                     // global flag to display memviewer

// PAGEINFO
//
//    The pageinfo[] array keeps track of information about each physical page.
//    There is one entry per physical page.
//    `pageinfo[pn]` holds the information for physical page number `pn`.
//    You can get a physical page number from a physical address `pa` using
//    `PAGENUMBER(pa)`. (This also works for page table entries.)
//    To change a physical page number `pn` into a physical address, use
//    `PAGEADDRESS(pn)`.
//
//    pageinfo[pn].refcount is the number of times physical page `pn` is
//      currently referenced. 0 means it's free.
//    pageinfo[pn].owner is a constant indicating who owns the page.
//      PO_KERNEL means the kernel, PO_RESERVED means reserved memory (such
//      as the console), and a number >=0 means that process ID.
//
//    pageinfo_init() sets up the initial pageinfo[] state.

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PhysicalPageInfo {
    pub owner: i8,
    pub refcount: i8,
}

lazy_static! {
    pub static ref PAGEINFO: Vec<PhysicalPageInfo> = {
        let num_pages = page_number(MEMSIZE_PHYSICAL as *const u8);
        vec![PhysicalPageInfo { owner: 0, refcount: 0 }; num_pages]
    };
}

#[repr(i8)]
#[allow(unused)]
pub enum PageOwner {
    PoFree = 0,         // this page is free
    PoRserved = -1,     // this page is reserved memory
    PoKernel = -2,      // this page is used by the kernel
}


// kernel(command)
//    Initialize the hardware and processes and start running. The `command`
//    string is an optional string passed from the boot loader.

#[no_mangle]
pub unsafe extern "C" fn kernel(command: *const u8) {
    // hardware_init();
    // pageinfo_init();
    // console_clear();
    // timer_init(HZ);

    // // Initialize process descriptors
    // let mut processes = Vec::with_capacity(NPROC);
    // for i in 0..NPROC {
    //     processes.push(Proc::new(i)); // Assume Proc::new() initializes default values
    // }

    // // Setting up processes based on command input
    // match command {
    //     "fork" => process_setup(1, 4),
    //     "forkexit" => process_setup(1, 5),
    //     "test" => process_setup(1, 6),
    //     "test2" => {
    //         for i in 1..=2 {
    //             process_setup(i, 6);
    //         }
    //     }
    //     _ => {
    //         for i in 1..=4 {
    //             process_setup(i, i - 1);
    //         }
    //     }
    // }

    // // Switch to the first process
    // run(&mut processes[1]);
}


// process_setup(pid, program_number)
//    Load application program `program_number` as process number `pid`.
//    This loads the application's code and data into memory, sets its
//    %rip and %rsp, gives it a stack page, and marks it as runnable.

#[no_mangle]
pub unsafe extern "C" fn process_setup(pid: usize, program_number: i32) {
    // let mut processes = PROCESSES.get_or_init(|| Mutex::new(Vec::new())).lock();
    // processes[pid].initialize_default();
    // processes[pid].set_pagetable(kernel_pagetable);

    // // Increment reference count for kernel pagetable
    // pageinfo[PAGENUMBER(kernel_pagetable)].refcount += 1;

    // let result = program_load(&mut processes[pid], program_number, None);
    // assert!(result >= 0);

    // // Configure stack and memory mapping
    // processes[pid].p_registers.reg_rsp = PROC_START_ADDR + PROC_SIZE * pid;
    // let stack_page = processes[pid].p_registers.reg_rsp - PAGESIZE;
    // assert_eq!(assign_physical_page(stack_page, pid), 0);
    // virtual_memory_map(
    //     processes[pid].p_pagetable,
    //     stack_page,
    //     stack_page,
    //     PAGESIZE,
    //     PTE_P | PTE_W | PTE_U,
    // );

    // processes[pid].p_state = ProcessState::Runnable;
}


// assign_physical_page(addr, owner)
//    Allocates the page with physical address `addr` to the given owner.
//    Fails if physical page `addr` was already allocated. Returns 0 on
//    success and -1 on failure. Used by the program loader.

#[no_mangle]
pub unsafe extern "C" fn assign_physical_page(addr: usize, owner: usize) -> i32 {
    // let pn = page_number(addr as *const u8);
    // if pn >= PAGEINFO.len() {
    //     return -1;
    // }

    // if PAGEINFO[pn].refcount != 0 {
    //     return -1;
    // }

    // PAGEINFO[pn].owner = owner as i8;
    // PAGEINFO[pn].refcount = 1;
    0
}

#[no_mangle]
pub unsafe extern "C" fn syscall_mapping(p: &mut Proc) {
    // let mapping_ptr = p.p_registers.reg_rdi;
    // let ptr = p.p_registers.reg_rsi;

    // // Convert to physical address so the kernel can write to it
    // let map = virtual_memory_lookup(p.p_pagetable, mapping_ptr);

    // // Check for write access
    // if (map.perm & (PTE_W | PTE_U)) != (PTE_W | PTE_U) {
    //     return;
    // }

    // let endaddr = mapping_ptr + size_of::<VAMapping>() as u64 - 1;
    // // Check for write access for the end address
    // let end_map = virtual_memory_lookup(p.p_pagetable, endaddr);
    // if (end_map.perm & (PTE_W | PTE_P)) != (PTE_W | PTE_P) {
    //     return;
    // }

    // // Find the actual mapping now
    // let ptr_lookup = virtual_memory_lookup(p.p_pagetable, ptr);

    // // Equivalent of memcpy in C: Copy data from ptr_lookup to the physical address `map.pa`
    // unsafe {
    //     let dest = map.pa as *mut VAMapping;
    //     ptr::write(dest, ptr_lookup);
    // }
}

#[no_mangle]
pub unsafe extern "C" fn syscall_mem_tog(process: &mut Proc) {
    // let p = process.p_registers.reg_rdi as pid_t;

    // if p == 0 {
    //     // Toggle global display status
    //     // Assuming disp_global is a static mutable variable
    //     unsafe {
    //         DISP_GLOBAL = !DISP_GLOBAL;
    //     }
    // } else {
    //     if p < 0 || p > NPROC || p != process.p_pid {
    //         return;
    //     }
    //     // Toggle the display status for the specific process
    //     process.display_status = !process.display_status;
    // }
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

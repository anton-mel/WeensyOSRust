use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;
use crate::virtual_memory_map;
use stdlib::my_assert;

unsafe extern "C" {
    #[link(name = "vm")] fn set_pagetable(pagetable: *mut x86_64_pagetable);
    #[link(name = "kloader")] fn program_load(process: *mut Proc, program_number: i32, arg: *const u8) -> i32;
    #[link(name = "k-exception")] fn exception_return(registers: *const x86_64_registers);
    #[link(name = "k-hardware")] fn process_init(process: *mut Proc);
    #[link(name = "k-hardware")] pub fn c_panic(format: *const core::ffi::c_char, ...) -> !;
    static kernel_pagetable: *mut x86_64_pagetable;
}

pub struct ProcessTable {
    pub processes: [Proc; NPROC], // array of processes
    current: Option<*mut Proc>,   // pointer to currently executing proc
}

impl ProcessTable {
    pub fn new() -> Self {
        let mut processes = [Proc::default(); NPROC];
        for pid in 0..NPROC {
            // Note that `processes[0]` is never used.
            processes[pid] = Proc::new(pid as i32, P_FREE);
        }
        ProcessTable {
            processes,
            current: None,
        }
    }

    // process_setup(pid, program_number)
    //    Load application program `program_number` as process number `pid`.
    //    This loads the application's code and data into memory, sets its
    //    %rip and %rsp, gives it a stack page, and marks it as runnable.

    pub fn process_setup(&mut self, pid: usize, pn: usize) -> Proc {
        let p = &mut self.processes[pid];
        unsafe { 
            process_init(p);
            p.p_pagetable = kernel_pagetable;

            let r = program_load(p, pn as i32, core::ptr::null());
            my_assert!(r >= 0);
        }
        return *p;
    }

    // run(p)
    //    Run process `p`. This means reloading all the registers from
    //    `p->p_registers` using the `popal`, `popl`, and `iret` instructions.
    //
    //    As a side effect, sets `current = p`.

    pub fn run(&mut self, pid: usize) {
        let p = &mut self.processes[pid];
        // assert(p.p_state == P_RUNNABLE);
        self.current = Some(p);
        unsafe{
            // Load the process's current pagetable.
            set_pagetable(p.p_pagetable);
            // This function is defined in k-exception.S. It restores the process's
            // registers then jumps back to user mode.
            exception_return(&p.p_registers);
        }
        loop {} // should never get here
    }

    // schedule
    //    Pick the next process to run and then run it.
    //    If there are no runnable processes, spins forever.

    pub fn schedule(&mut self) {
        unsafe extern "C" {
            #[link(name = "k-hardware")]
            fn check_keyboard() -> core::ffi::c_int;
        }
    
        let mut pid: usize;
    
        if let Some(current_proc_ptr) = self.current {
            let current_proc = unsafe { &*current_proc_ptr };
            pid = current_proc.p_pid as usize;
    
            loop {
                pid = (pid + 1) % NPROC;
                if self.processes[pid].p_state == P_RUNNABLE {
                    self.run(pid);
                }
                // If Control-C was typed, exit the virtual machine.
                unsafe{ check_keyboard(); }
            }
        } else {
            unsafe {
                c_panic("(schedule) No current process available.".as_ptr() as *const core::ffi::c_char);
            }
        }
    }       
}

use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;
use stdlib::my_assert;

unsafe extern "C" {
    fn set_pagetable(pagetable: *mut x86_64_pagetable);
    fn program_load(process: *mut Proc, program_number: i32, arg: *const u8) -> i32;
    fn exception_return(registers: *const x86_64_registers);
    fn process_init(process: *mut Proc);
    fn c_panic(format: *const core::ffi::c_char, ...) -> !;
    static kernel_pagetable: *mut x86_64_pagetable;
}

pub struct ProcessTable {
    pub processes: [Proc; NPROC], // array of processes
    pub current: Option<*mut Proc>,   // pointer to currently executing proc
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
        let p = self.get_process_by_pid_mut(pid);
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
        let p_ptr = self.get_process_by_pid_mut(pid) as *mut Proc;
        unsafe{
            my_assert!((*p_ptr).p_state == P_RUNNABLE);
            self.current = Some(p_ptr);
        }
        let p = self.get_process_by_pid_mut(pid);
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
    
    // exception
    //    Copy the saved registers into the `current` process descriptor
    //    and set up the kernel's page table.
    
    pub fn exception(&mut self, reg: &mut x86_64_registers) {
        if let Some(current_proc_ptr) = self.current {
            let current_proc = unsafe { &mut *current_proc_ptr };
            current_proc.p_registers = *reg;
            unsafe {
                current_proc.p_pagetable = kernel_pagetable;
            }
        } else {
            unsafe {
                c_panic("(exception) No current process available.".as_ptr() as *const core::ffi::c_char);
            }
        }
    }

    // get_current_process
    //    Returns a reference to the current process, if set. 
    //    If not, triggers a panic.

    pub fn get_current_process(&self) -> Proc {
        match self.current {
            Some(ptr) => unsafe {
                if ptr.is_null() {
                    c_panic("(get_current_process) current process pointer is null.".as_ptr() as *const core::ffi::c_char);
                } else {
                    *ptr
                }
            },
            None => unsafe {
                c_panic("(get_current_process) No current process available.".as_ptr() as *const core::ffi::c_char);
            },
        }
    }

    // get_current_process_mut
    //    Returns a mutable reference to the current process, if set. 
    //    If not, triggers a panic.

    pub fn get_current_process_mut(&mut self) -> &mut Proc {
        match self.current {
            Some(ptr) => unsafe {
                if ptr.is_null() {
                    c_panic("(get_current_process_mut) current process pointer is null.".as_ptr() as *const core::ffi::c_char);
                } else {
                    &mut *ptr
                }
            },
            None => unsafe {
                c_panic("(get_current_process_mut) No current process available.".as_ptr() as *const core::ffi::c_char);
            },
        }
    }

    // get_process_by_pid
    //    Returns a reference to the process with the given PID, if it exists.
    //    If the PID is invalid or the process is not set, triggers a panic.

    pub fn get_process_by_pid(&self, pid: usize) -> &Proc {
        if pid >= NPROC {
            unsafe {
                c_panic("(get_process_by_pid) Invalid PID.".as_ptr() as *const core::ffi::c_char);
            }
        }
        &self.processes[pid]
    }

    // get_process_by_pid_mut
    //    Returns a mutable reference to the process with the given PID, if it exists.
    //    If the PID is invalid or the process is not set, triggers a panic.

    pub fn get_process_by_pid_mut(&mut self, pid: usize) -> &mut Proc {
        if pid >= NPROC {
            unsafe {
                c_panic("(get_process_by_pid_mut) Invalid PID.".as_ptr() as *const core::ffi::c_char);
            }
        }
        &mut self.processes[pid]
    }

    // set_register_rax
    //    Helper function to safely set a register in the current process.

    pub fn set_register_rax(&mut self, value: u64) {
        if let Some(current_proc_ptr) = self.current {
            let current_proc = unsafe { &mut *current_proc_ptr };
            current_proc.p_registers.reg_rax = value;
        } else {
            unsafe {
                c_panic("(set_register_rax) No current process available.".as_ptr() as *const core::ffi::c_char);
            }
        }
    }
}

use bindings::bindings_x86_64::*;
use bindings::bindings_kernel::*;

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

#[repr(i8)]
#[allow(unused)]
#[derive(PartialEq, Clone)]
pub enum PageOwner {
    PoFree = 0,         // this page is free
    PoReserved = -1,     // this page is reserved memory
    PoKernel = -2,      // this page is used by the kernel
}

pub struct PhysicalPageInfoTable {
    pub pageinfo: [PhysicalPageInfo; NPAGES as usize],
}

impl PhysicalPageInfoTable {
    pub const fn new() -> Self {
        let mut pageinfo = [
            PhysicalPageInfo { 
                owner: PageOwner::PoFree as i8, 
                refcount: 0 
            }; NPAGES as usize];
        PhysicalPageInfoTable { pageinfo }
    }

    // pageinfo_init
    //    Initialize the `pageinfo[]` array.

    pub fn pageinfo_init(&mut self) {
        extern "C" {
            #[link(name = "k-hardware")] 
            fn physical_memory_isreserved(pa: usize) -> core::ffi::c_int;
            static mut start_data: u8;
            static mut end: u8;
        }

        unsafe {
            let end_addr = &end as *const u8 as usize;
            for addr in (0..MEMSIZE_PHYSICAL).step_by(PAGESIZE as usize) {
                let owner = if physical_memory_isreserved(addr as usize) != 0 {
                    PageOwner::PoReserved
                } else if (addr >= KERNEL_START_ADDR && addr < end_addr as u64)
                    || addr == KERNEL_STACK_TOP - PAGESIZE
                {
                    PageOwner::PoKernel
                } else {
                    PageOwner::PoFree
                };
                let page = &mut self.pageinfo[(addr / PAGESIZE) as usize];
                page.owner = owner.clone() as i8;
                page.refcount = if owner != PageOwner::PoFree { 1 } else { 0 };
            }
        }
    }

    // get_current_process_mut
    //    Returns a mutable reference to the pid process. 
    pub fn get_page_info_ref(&mut self, pn: usize) -> Option<&mut PhysicalPageInfo> {
        self.pageinfo.get_mut(pn)
    }
}

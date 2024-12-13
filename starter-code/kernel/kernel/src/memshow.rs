use bindings::bindings_x86_64::*;


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

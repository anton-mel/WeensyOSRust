// This file provides missing functionality for the Rust kernel
// required to complete the pset as a dynamicly linked library.
// It avoids th use of allocation and other OS features to keep
// the Rust kernel as minimal as possible. You do not need to 
// understand what this code does.

#[macro_export]
macro_rules! my_assert {
    ($condition:expr) => {
        if !$condition {
            unsafe {
                const FILE: &str = file!();
                const LINE: u32 = line!();
                const CONDITION: &str = stringify!($condition);

                fn itoa(mut n: u32) -> [u8; 10] {
                    let mut buf = [0u8; 10];
                    let mut i = 9;

                    if n == 0 {
                        buf[i] = b'0';
                    } else {
                        while n > 0 {
                            buf[i] = (n % 10) as u8 + b'0';
                            n /= 10;
                            if i == 0 {
                                break;
                            }
                            i -= 1;
                        }
                    }

                    // Move the digits to 
                    // the start of the buffer
                    let start = i;
                    buf.rotate_left(9 - start);

                    buf
                }

                // Use a statically allocated 
                // buffer for the message
                static mut MESSAGE: [u8; 256] = [0; 256];

                let len = {
                    let mut pos = 0;
                    // Build the assertion message
                    for &byte in b"Assertion failed at " {
                        if pos >= MESSAGE.len() { break; }
                        MESSAGE[pos] = byte;
                        pos += 1;
                    }
                    for &byte in FILE.as_bytes() {
                        if pos >= MESSAGE.len() { break; }
                        MESSAGE[pos] = byte;
                        pos += 1;
                    }
                    if pos < MESSAGE.len() { MESSAGE[pos] = b':'; pos += 1; }

                    // Use custom itoa function to convert LINE to string
                    let line_str = itoa(LINE);
                    for &byte in &line_str {
                        if pos >= MESSAGE.len() { break; }
                        MESSAGE[pos] = byte;
                        pos += 1;
                    }

                    if pos < MESSAGE.len() { MESSAGE[pos] = b':'; pos += 1; }
                    for &byte in b" condition '" {
                        if pos >= MESSAGE.len() { break; }
                        MESSAGE[pos] = byte;
                        pos += 1;
                    }
                    for &byte in CONDITION.as_bytes() {
                        if pos >= MESSAGE.len() { break; }
                        MESSAGE[pos] = byte;
                        pos += 1;
                    }
                    if pos < MESSAGE.len() { MESSAGE[pos] = b'\''; pos += 1; }
                    if pos < MESSAGE.len() { MESSAGE[pos] = b'\0'; pos += 1; }
                    pos
                };

                // WeensyOS: Require the use of c_panic for the display
                c_panic(MESSAGE.as_ptr() as *const core::ffi::c_char);
            }
        }
    };
}

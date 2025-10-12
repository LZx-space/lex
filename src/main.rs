// LZx Exploratory Operating System
#![no_std]
#![no_main]
#![feature(allocator_api, alloc_error_handler)]
mod interrupt;
pub mod uart;

use core::panic::PanicInfo;

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////

/// 打印
#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
			});
}
/// 打印并换行
#[macro_export]
macro_rules! println {
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////
#[unsafe(no_mangle)]
extern "C" fn eh_personality() {
    loop {
        unsafe {
            let ptr = 0x1000_0000 as *mut u8;
            ptr.add(0).write_volatile(b'Z'); // 发送 'X'
        }
    }
}

/// 这个函数会在 panic 时被调用
/// `info` 参数包含了 panic 发生的文件名、行号等可选信息
/// 在裸机环境中，我们通常无法“退出”或“打印”，所以最简单的实现就是让程序无限循环
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    } else {
        println!("no information available.");
    }
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            let ptr = 0x1000_0000 as *mut u8;
            ptr.add(0).write_volatile(b'Y'); // 发送 'X'
        }
    }
}

/// 声明一个外部符号，链接器会寻找它作为程序的入口
/// 函数在 `/asm/boot.S` 汇编文件中调用
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let mut uart = uart::Uart::new(0x1000_0000);
    uart.init();
    uart.put(b'H');
    println!("Hello, World!");
    loop {
        if let Some(c) = uart.get() {
            match c {
                8 => {
                    // This is a backspace, so we essentially have
                    // to write a space and backup again:
                    print!("{}{}{}", 8 as char, ' ', 8 as char);
                }
                10 | 13 => {
                    // Newline or carriage-return
                    println!();
                }
                0x1b => {
                    // Those familiar with ANSI escape sequences
                    // knows that this is one of them. The next
                    // thing we should get is the left bracket [
                    // These are multi-byte sequences, so we can take
                    // a chance and get from UART ourselves.
                    // Later, we'll button this up.
                    if let Some(next_byte) = uart.get() {
                        if next_byte == 91 {
                            // This is a right bracket! We're on our way!
                            if let Some(b) = uart.get() {
                                match b as char {
                                    'A' => {
                                        println!("That's the up arrow!");
                                    }
                                    'B' => {
                                        println!("That's the down arrow!");
                                    }
                                    'C' => {
                                        println!("That's the right arrow!");
                                    }
                                    'D' => {
                                        println!("That's the left arrow!");
                                    }
                                    _ => {
                                        println!("That's something else.....");
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    print!("{}", c as char);
                }
            }
        }
    }
}

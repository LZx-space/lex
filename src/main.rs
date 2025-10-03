// LZx Exploratory Operating System
#![no_std]
#![no_main]
#![feature(allocator_api, alloc_error_handler)]
pub mod uart;
mod interrupt;

use core::panic::PanicInfo;

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////

/// 打印
#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
        use core::fmt::Write;
        let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
    }};
}
/// 打印并换行
#[macro_export]
macro_rules! println
{
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

/// 声明一个外部符号，链接器会寻找它作为程序的入口
/// 函数在 `/asm/boot.S` 汇编文件中调用
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    uart::Uart::new(0x1000_0000).init();
    println!("Hello, World!");
    loop {}
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

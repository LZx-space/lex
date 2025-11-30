// LZx Exploratory Operating System
#![no_std]
#![no_main]
#![feature(allocator_api, alloc_error_handler)]
#![feature(nonzero_ops)]

///////////////////////////////////////////////////
// 汇编代码内联
///////////////////////////////////////////////////
global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/mem.S"));

mod memory;
mod uart;
mod util;

use crate::uart::Uart;
use core::arch::{asm, global_asm};
use core::fmt::{Error, Write};
use core::panic::PanicInfo;

/// 声明一个外部符号，链接器会寻找它作为程序的入口
/// * linker script在`/lds/virt.lds`
/// * 该函数在 `/asm/boot.S` 汇编文件中调用
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    Uart::init(0x1000_0000);
    match memory::init() {
        Ok(_) => {
            println!("内存管理初始化-成功");
        }
        Err(err) => {
            println!("内存管理初始化-失败-{}", err)
        }
    }
    println!("Hello World!");
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

///////////////////////////////////////////////////
// RUST宏
///////////////////////////////////////////////////

struct UartWrapper<'a> {
    uart: &'a Uart,
}

impl<'a> Write for UartWrapper<'a> {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        let uart = self.uart;
        for b in out.bytes() {
            uart.transmit(b);
        }
        Ok(())
    }
}

impl<'a> UartWrapper<'a> {
    pub fn new(uart: &'a Uart) -> Self {
        UartWrapper { uart }
    }
}

/// 打印
#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
        use core::fmt::Write;
        let uart = crate::uart::Uart::uart();
        let mut wrapper = crate::UartWrapper::new(&uart);
        let _ = write!(&mut wrapper, $($args)+);
    }};
}

/// 打印并换行
#[macro_export]
macro_rules! println {
	() => ({
		crate::print!("\r\n")
	});
	($fmt:expr) => ({
		crate::print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		crate::print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

///////////////////////////////////////////////////
// 语言功能
///////////////////////////////////////////////////
#[unsafe(no_mangle)]
extern "C" fn eh_personality() {
    loop {}
}

/// 这个函数会在 panic 时被调用
/// * `info` 参数包含了 panic 发生的文件名、行号等可选信息
/// * 在裸机环境中，我们通常无法“退出”或“打印”，所以最简单的实现就是让程序无限循环
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
    loop {}
}

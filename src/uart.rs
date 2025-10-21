use spin::once::Once;

static UART: Once<Uart> = Once::new();

/// 寄存器地址映射（物理布局）：
/// * 偏移 0:
///     * DLAB = 0时，`THR`/`RBR`(发送保持寄存器/接收缓冲寄存器)
///     * DLAB = 1时，`DLL`(除数锁存低字节)
/// * 偏移 1:
///     * DLAB = 0时，`IER`(中断使能寄存器)
///     * DLAB = 1时，`DLM`(除数锁存高字节)
/// * 偏移 2:
///     * `IIR`(中断标识寄存器)
///     * `FCR`(FIFO控制寄存器)
/// * 偏移 3: `LCR`(线控制寄存器)
///     * 8位分别功能
///       ```
///       bit7  bit6  bit5  bit4  bit3  bit2  bit1  bit0
///       ----------------------------------------------
///       DLAB  SB    EPS   PEN   STB   WLS2  WLS1  WLS0
///       ```
/// * 偏移 4: `MCR`(Modem控制寄存器)
/// * 偏移 5: `LSR`(线状态寄存器)
/// * 偏移 6: `MSR`(Modem状态寄存器)
/// * 偏移 7: `SCR`(Scratch寄存器)
pub struct Uart {
    base_address: usize,
}

#[allow(unused)]
impl Uart {
    // -----寄存器偏移量
    const DLL: usize = 0x00; // 波特率除数低位
    const RBR: usize = 0x00; // 接收数据寄存器（读）
    const THR: usize = 0x00; // 发送数据寄存器（写）- 修正为THR
    const DLM: usize = 0x01; // 波特率除数高位
    const LCR: usize = 0x03; // 线路控制寄存器
    const LSR: usize = 0x05; // 线路状态寄存器

    // -----LCR寄存器位定义（掩码）
    const LCR_DLAB: u8 = 1 << 7; // 波特率除数锁存位（1=允许修改波特率）
    const LCR_DATA_8BIT: u8 = 0b11; // 8 位数据位
    const LCR_STOP_1BIT: u8 = 0 << 2; // 1 个停止位
    const LCR_PARITY_NONE: u8 = 0 << 3; // 无校验位

    // -----LSR寄存器位定义
    const LSR_TX_EMPTY: u8 = 1 << 5; // 发送缓冲区空（可发送数据）

    /// 获取UART已初始化单例。必须先调用[`Self::init`]以初始化
    pub fn uart() -> &'static Uart {
        UART.get()
            .expect("Uart not initialized, must be called init(usize) first.")
    }

    /// 初始化UART单例
    pub fn init(base_address: usize) {
        UART.call_once(|| {
            let uart = Uart { base_address };
            uart.init_hardware(); // 在单例创建时初始化硬件
            uart
        });
    }

    /// 初始化UART硬件
    fn init_hardware(&self) {
        let base = self.base_address as *mut u8;
        unsafe {
            // 步骤 1：设置 DLAB 位（允许修改波特率除数）
            let mut lcr = base.add(Self::LCR).read_volatile();
            lcr |= Self::LCR_DLAB; // 置位 DLAB
            base.add(Self::LCR).write_volatile(lcr);

            // 步骤 2：配置波特率（115200）
            const DIVISOR: u16 = 10;
            base.add(Self::DLL).write_volatile((DIVISOR & 0xFF) as u8); // 低位
            base.add(Self::DLM).write_volatile((DIVISOR >> 8) as u8); // 高位

            // 步骤 3：配置数据格式（8 位数据位，1 位停止位，无校验），并清除 DLAB
            lcr = Self::LCR_DATA_8BIT | Self::LCR_STOP_1BIT | Self::LCR_PARITY_NONE;
            base.add(Self::LCR).write_volatile(lcr); // DLAB 被自动清除
        }
    }

    /// 发送字节
    pub fn transmit(&self, byte: u8) {
        let base = self.base_address as *mut u8;
        unsafe {
            // 等待发送缓冲区为空（LSR的第5位为1）
            while (base.add(Self::LSR).read_volatile() & Self::LSR_TX_EMPTY) == 0 {}
            // 写入到发送寄存器 - 使用THR
            base.add(Self::THR).write_volatile(byte);
        }
    }
}

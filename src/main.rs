#![no_std]
#![no_main]

use riscv_rt::entry;
use panic_halt as _;

/// QEMU ‘virt’ board UART0 base address
const UART0_BASE: usize = 0x1000_0000;
const UART_THR:  *mut u8 = UART0_BASE as *mut u8;           // Transmit Holding Reg  (offset 0x0)
const UART_LSR:  *const u8 = (UART0_BASE + 5) as *const u8; // Line Status Reg       (offset 0x5)
const LSR_TX_EMPTY: u8 = 1 << 5;                            // Bit 5 = THR & TSR empty

// putchar asm code 
#[inline(always)]
fn mmio_putchar(byte: u8) {
    unsafe {
        while core::ptr::read_volatile(UART_LSR) & LSR_TX_EMPTY == 0 {};
        core::ptr::write_volatile(UART_THR, byte);
    }
}

fn puts(s: &str) {
    for c in s.bytes() {
        mmio_putchar(c);
    }
    mmio_putchar(b'\n');
}

#[entry]
fn main() -> ! {
    
    puts("Hello, RISC-V from M-Mode!");

    loop {
        
    }
}
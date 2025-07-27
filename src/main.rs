#![no_std]
#![no_main]

use core::arch::asm;

use riscv_rt::entry;
use panic_halt as _;

// putchar asm code 
#[inline(always)]
fn sbi_putchar(ch: u8) {
    unsafe {
        asm!(
            "mv a0, {char}",
            "li a7, 0x01",
            "ecall",
            char = in(reg) ch as usize,
            lateout("a0") _,
            options(nostack, nomem),
        );
    }
}

fn puts(s: &str) {
    for c in s.bytes() {
        sbi_putchar(c);
    }
    sbi_putchar(b'\n');
}

#[entry]
fn main() -> ! {
    
    puts("Hello, RISC-V!");

    loop {
        
    }
}
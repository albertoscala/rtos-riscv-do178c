#![no_std]
#![no_main]

use core::panic::PanicInfo;

use riscv_rt::entry;
use riscv::register::{mcause, mepc, mie, mip, mstatus, mtvec::{self, Mtvec, TrapMode}};
use riscv::interrupt::{Exception, Interrupt};

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
}

// Trap handler

unsafe extern "C" {
    fn trap_entry(); // from trap.S
}

#[no_mangle]
pub extern "C" fn trap_handler(sp: *mut usize) {
    unsafe {
        let mcause_val = *sp.add(17);
        let mepc_val = *sp.add(16);
        let mtval_val = *sp.add(18);

        let cause = riscv::register::mcause::Mcause::from_bits(mcause_val);

        puts("Trap handler invoked!");
        puts("mcause: ");
        put_hex(mcause_val);
        puts("\n");

        puts("mepc: ");
        put_hex(mepc_val);
        puts("\n");

        puts("mtval: ");
        put_hex(mtval_val);
        puts("\n");

        match cause.cause() {
            riscv::register::mcause::Trap::Exception(e) => {
                puts("Exception detected\n");
                match e {
                    2 => handle_illegal(),
                    5 => handle_mem_fault(),
                    7 => handle_mem_fault(),
                    _ => panic!("Generic Exception"),
                }
            }
            riscv::register::mcause::Trap::Interrupt(i) => {
                puts("Interrupt detected\n");
                match i {
                    7 => timer_interrupt(),
                    11 => external_interrupt(),
                    _ => panic!("Generic Interrupt"),
                }
            }
        }
    }
}

// Timer

const MTIME: *mut u64 = 0x0200_BFF8 as *mut u64;
const MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;

fn init_timer(ticks: u64) {
    unsafe {
        let now = core::ptr::read_volatile(MTIME);
        core::ptr::write_volatile(MTIMECMP, now + ticks);

        mie::set_mtimer();      // Enable machine timer interrupt
        mstatus::set_mie();     // Global machine interrupt enable
    }
}

fn timer_interrupt() {
    unsafe {
        let now = core::ptr::read_volatile(MTIME);
        core::ptr::write_volatile(MTIMECMP, now + 10_000);
    }
}

// Handlers
fn put_hex(mut val: usize) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    let mut i = 0;

    if val == 0 {
        mmio_putchar(b'0');
        return;
    }

    while val > 0 {
        buf[i] = hex_chars[val & 0xF];
        val >>= 4;
        i += 1;
    }

    for ch in buf[..i].iter().rev() {
        mmio_putchar(*ch);
    }
}

fn put_dec(mut val: usize) {
    let mut buf = [0u8; 20];
    let mut i = 0;

    if val == 0 {
        mmio_putchar(b'0');
        return;
    }

    while val > 0 {
        buf[i] = b'0' + (val % 10) as u8;
        val /= 10;
        i += 1;
    }

    for ch in buf[..i].iter().rev() {
        mmio_putchar(*ch);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    puts("=== PANIC ===\n");

    if let Some(location) = info.location() {
        puts("File: ");
        puts(location.file());
        puts("\n");
        puts("Line: ");
        put_dec(location.line() as usize);
        puts("\n");
    }

    // Read trap cause and PC for diagnostics
    let cause = mcause::read();
    let pc = mepc::read();

    puts("Raw mcause bits: 0x");
    put_hex(cause.bits());
    puts("\n");

    puts("Cause: ");
    match cause.cause() {
        riscv::register::mcause::Trap::Exception(e) => {
            puts("Exception\n");
            match e {
                2 => puts("Illegal instruction\n"),   // Illegal instruction
                5 => puts("Load access fault\n"),     // Load access fault
                7 => puts("Store/AMO access fault\n"),// Store/AMO access fault
                _ => puts("Other Exception\n"),
            }
            // You can refine this with e.g. check e == IllegalInstruction
        }
        riscv::register::mcause::Trap::Interrupt(i) => {
            puts("Interrupt\n");
            match i {
                7 => puts("Machine Timer Interrupt\n"),
                11 => puts("Machine External Interrupt\n"),
                _ => puts("Other Interrupt\n"),
            }
            // Likewise: MachineTimer, MachineExternal, etc.
        }
    }

    puts("PC: 0x");
    put_hex(pc);
    puts("\n");

    loop {} // Halt deterministically
}

fn external_interrupt() {
    panic!("External interrupt occurred");
}

fn handle_illegal() {
    panic!("Illegal instruction trapped");
}

fn handle_mem_fault() {
    panic!("Memory access fault");
}

fn schedule_next_task() {
    // Stub scheduler
}

// Main

#[entry]
fn main() -> ! {
    
    unsafe {
        let mtvec_value = Mtvec::from_bits(trap_entry as usize);
        mtvec::write(mtvec_value);
        init_timer(10_000);

        let current = mtvec::read();
        if current.trap_mode() == TrapMode::Direct {
            puts("TrapMode is actually Direct!\n");
        }
        if current.address() == (trap_entry as usize) & !0x3  {
            puts("Trap Entry is correct!\n");
        }

        // === Test Illegal Instruction ===
        //core::arch::asm!("unimp");

        // === Test Memory Access Fault (Load) ===
        //let p: *mut u64 = 0xFFFF_FFFF_FFFF_FFFF as *mut u64;
        //core::ptr::read_volatile(p);
        let p: *mut u64 = 0x100 as *mut u64; // low unmapped address
        core::ptr::read_volatile(p);
        
    }

    loop {

    }
}
#![no_std]
#![no_main]

use riscv_rt::entry;
use panic_halt as _;
use riscv::register::{mcause, mepc, mie, mip, mstatus, mtvec::{self, Mtvec, TrapMode}};

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

// Trap handler

unsafe extern "C" {
    fn trap_entry(); // from trap.S
}

#[no_mangle]
pub extern "C" fn trap_handler() {
    let cause = mcause::read();

    if cause.is_interrupt() {
        match cause.code() {
            7 => timer_interrupt(),         // Machine timer interrupt
            11 => external_interrupt(),     // Machine external interrupt
            _ => panic!("Unhandled interrupt {:?}", cause),
        }
    } else {
        match cause.code() {
            2 => handle_illegal(),          // Illegal instruction
            5 => handle_mem_fault(),        // Load access fault
            7 => handle_mem_fault(),        // Store/AMO access fault
            _ => panic!("Unhandled exception {:?}", cause),
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

        // Verification step
        let current = mtvec::read();
        if current.trap_mode() == TrapMode::Direct {
            puts("TrapMode is actually Direct!");
        }
        if current.address() == (trap_entry as usize) & !0x3  {
            puts("Trap Entry is correct!");
        }

        // Testing handlers
    }



    loop {
        // Idle loop
    }
}
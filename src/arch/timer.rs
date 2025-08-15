use riscv::register::{mie, mstatus};
use crate::kernel::services::rtos_on_timer_tick;

const MTIME:    *mut u64 = 0x0200_BFF8 as *mut u64;
const MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;

pub fn init_timer(ticks: u64) {
    unsafe {
        let now = core::ptr::read_volatile(MTIME);
        core::ptr::write_volatile(MTIMECMP, now + ticks);
        mie::set_mtimer();      // Enable machine timer interrupt
        mstatus::set_mie();     // Global MIE
    }
}

// Re-arm and notify kernel tick
pub fn timer_interrupt() {
    unsafe {
        rtos_on_timer_tick();
        let now = core::ptr::read_volatile(MTIME);
        core::ptr::write_volatile(MTIMECMP, now + 10_000);
    }
}
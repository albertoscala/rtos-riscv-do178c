pub mod trap;
pub mod timer;

// === Trap frame layout saved by trap.S ===
// 20 * 8 = 160 bytes total on RV64
pub const FRAME_WORDS: usize = 20;

pub const OFF_RA:     isize = 0;
pub const OFF_T0:     isize = 1;
pub const OFF_T1:     isize = 2;
pub const OFF_T2:     isize = 3;
pub const OFF_A0:     isize = 4;
pub const OFF_A1:     isize = 5;
pub const OFF_A2:     isize = 6;
pub const OFF_A3:     isize = 7;
pub const OFF_A4:     isize = 8;
pub const OFF_A5:     isize = 9;
pub const OFF_A6:     isize = 10;
pub const OFF_A7:     isize = 11;
pub const OFF_T3:     isize = 12;
pub const OFF_T4:     isize = 13;
pub const OFF_T5:     isize = 14;
pub const OFF_T6:     isize = 15;
pub const OFF_MEPC:   isize = 16;
pub const OFF_MCAUSE: isize = 17;
pub const OFF_MTVAL:  isize = 18;

// Provided by trap.S
extern "C" {
    pub fn trap_entry();
    pub fn __rtos_boot_with_sp(sp: *mut usize) -> !;
}

// Linker-provided task stack region (.tasks)
extern "C" {
    pub static __task_stack_start: u8;
    pub static __task_stack_end:   u8;
}
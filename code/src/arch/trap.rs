use riscv::register::{mcause, mepc, mtval};
use crate::arch::{OFF_MCAUSE, OFF_MEPC, OFF_MTVAL};
use crate::drivers::uart::{puts, put_dec, put_hex};
use crate::arch::timer::timer_interrupt;
use crate::kernel::task::schedule;

// Exported symbol for mtvec
pub use super::trap_entry;

#[no_mangle]
pub extern "C" fn trap_handler(sp: *mut usize) -> *mut usize {
    unsafe {
        let mcause_val = *sp.offset(OFF_MCAUSE);
        let mepc_val   = *sp.offset(OFF_MEPC);

        // Manual decode (crate-version independent)
        let xlen_msb = (core::mem::size_of::<usize>() * 8 - 1) as u32;
        let is_interrupt = ((mcause_val >> xlen_msb) & 1) != 0;
        let code = mcause_val & ((1usize << xlen_msb) - 1);

        if is_interrupt {
            match code {
                7  => { // Machine Timer
                    timer_interrupt();
                    return schedule(sp);
                },
                _  => panic!("Generic Interrupt"),
            }
        } else {
            match code {
                2 => handle_illegal(),
                5 => handle_mem_fault(),
                7 => handle_mem_fault(),
                11 => { // ECALL from M-mode = task_yield()
                    *sp.offset(OFF_MEPC) = mepc_val.wrapping_add(4);
                    return schedule(sp);
                }
                _ => panic!("Generic Exception"),
            }
            sp
        }
    }
}

fn handle_illegal() {
    panic!("Illegal instruction trapped");
}

fn handle_mem_fault() {
    panic!("Memory access fault");
}
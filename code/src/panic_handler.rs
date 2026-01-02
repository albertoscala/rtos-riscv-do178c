use core::panic::PanicInfo;
use riscv::register::{mcause, mepc, mtval, mstatus, mie};
use crate::drivers::uart::{puts, put_dec, put_hex};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        // Lock down interrupts deterministically
        mstatus::clear_mie();
        mie::clear_mext();
        mie::clear_mtimer();
        mie::clear_msoft();
    }

    puts("=== PANIC ===\n");

    if let Some(location) = info.location() {
        puts("File: "); puts(location.file()); puts("\n");
        puts("Line: "); put_dec(location.line() as usize); puts("\n");
    }

    let cause = mcause::read();
    let pc    = mepc::read();
    let tval  = mtval::read();

    puts("Raw mcause bits: 0x"); put_hex(cause.bits()); puts("\n");

    match cause.cause() {
        mcause::Trap::Exception(code) => {
            puts("Cause: Exception (code "); put_dec(code as usize); puts(")\n");
            match code {
                0  => puts("Instruction address misaligned\n"),
                1  => puts("Instruction access fault\n"),
                2  => puts("Illegal instruction\n"),
                3  => puts("Breakpoint\n"),
                4  => puts("Load address misaligned\n"),
                5  => puts("Load access fault\n"),
                6  => puts("Store/AMO address misaligned\n"),
                7  => puts("Store/AMO access fault\n"),
                8  => puts("Environment call from U-mode\n"),
                9  => puts("Environment call from S-mode\n"),
                11 => puts("Environment call from M-mode (ECALL / task_yield)\n"),
                12 => puts("Instruction page fault\n"),
                13 => puts("Load page fault\n"),
                15 => puts("Store/AMO page fault\n"),
                _  => puts("Other Exception\n"),
            }
        }
        mcause::Trap::Interrupt(code) => {
            puts("Cause: Interrupt (code "); put_dec(code as usize); puts(")\n");
            match code {
                3  => puts("Machine Software Interrupt\n"),
                7  => puts("Machine Timer Interrupt\n"),
                11 => puts("Machine External Interrupt (PLIC)\n"),
                _  => puts("Other Interrupt\n"),
            }
        }
    }

    puts("mepc (PC): 0x"); put_hex(pc); puts("\n");
    puts("mtval    : 0x"); put_hex(tval); puts("\n");

    loop { core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst); }
}
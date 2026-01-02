#![no_std]
#![no_main]
#![allow(unused)]

use riscv_rt::entry;
use riscv::register::mtvec::{self, Mtvec, TrapMode};

mod arch;
mod drivers;
mod kernel;
mod panic_handler; // just defining the module pulls in #[panic_handler]

use arch::trap::trap_entry;
use arch::timer::init_timer;
use drivers::uart::{puts, put_dec, put_hex};
use kernel::services::{delay_ms, task_yield, ticks, Semaphore, SpinLock};
use kernel::task::{create_task, start_first_task};

//TODO: Capire da dove li va a prendere
extern "C" {
    static __task_stack_start: u8;
    static __task_stack_end:   u8;
}

static UART_LOCK: SpinLock<()> = SpinLock::new(());
static SEM: Semaphore = Semaphore::new(0);

#[entry]
fn main() -> ! {
    unsafe {
        // Show .tasks region for diagnostics
        let base = &__task_stack_start as *const u8 as usize;
        let end  = &__task_stack_end   as *const u8 as usize;
        puts(".tasks base: 0x"); put_hex(base); puts("\n");
        puts(".tasks end : 0x"); put_hex(end ); puts("\n");
        puts(".tasks size: 0x"); put_hex(end - base); puts("\n");

        // Trap vector + 1ms timer tick (+10_000 at =10 MHz on QEMU virt)
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

        // --- Demo tasks ---

        extern "C" fn periodic_500ms() {
            loop {
                let _g = UART_LOCK.lock();
                puts("[500ms] tick="); put_dec(ticks() as usize); puts("\n");
                drop(_g);
                delay_ms(500);
            }
        }

        extern "C" fn producer_200ms() {
            loop {
                delay_ms(200);
                SEM.post();
                let _g = UART_LOCK.lock();
                puts("[prod] +1\n");
                drop(_g);
            }
        }

        extern "C" fn consumer_blocking() {
            loop {
                SEM.wait();
                let _g = UART_LOCK.lock();
                puts("[cons] got token at tick "); put_dec(ticks() as usize); puts("\n");
                drop(_g);
            }
        }

        extern "C" fn yield_spammer() {
            loop {
                {
                    let _g = UART_LOCK.lock();
                    puts("[yield]\n");
                }
                task_yield();
            }
        }

        let _t0 = create_task(periodic_500ms,   1);
        let _t1 = create_task(producer_200ms,   1);
        let _t2 = create_task(consumer_blocking,1);
        let _t3 = create_task(yield_spammer,    1);

        start_first_task();
    }
}
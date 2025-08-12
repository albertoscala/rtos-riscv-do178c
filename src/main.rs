#![no_std]
#![no_main]

use core::panic::PanicInfo;

use riscv_rt::entry;
use riscv::register::{mcause, mepc, mie, mip, mstatus, mtval, mtvec::{self, Mtvec, TrapMode}};
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

// Tasks

// === Trap frame layout saved by trap.S ===
// 20 * 8 = 160 bytes total
const FRAME_WORDS: usize = 20;

const OFF_RA:     isize = 0;
const OFF_T0:     isize = 1;
const OFF_T1:     isize = 2;
const OFF_T2:     isize = 3;
const OFF_A0:     isize = 4;
const OFF_A1:     isize = 5;
const OFF_A2:     isize = 6;
const OFF_A3:     isize = 7;
const OFF_A4:     isize = 8;
const OFF_A5:     isize = 9;
const OFF_A6:     isize = 10;
const OFF_A7:     isize = 11;
const OFF_T3:     isize = 12;
const OFF_T4:     isize = 13;
const OFF_T5:     isize = 14;
const OFF_T6:     isize = 15;
const OFF_MEPC:   isize = 16;
const OFF_MCAUSE: isize = 17;
const OFF_MTVAL:  isize = 18;

unsafe extern "C" {
    static __task_stack_start: u8;
    static __task_stack_end:   u8;
}

const MAX_TASKS: usize = 4;
const TASK_STACK_BYTES: usize = 4096; // tune as you like

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState { Ready, Running, Blocked }

#[repr(C)]
pub struct Tcb {
    pub sp: *mut usize,              // task’s saved SP (to a trap frame)
    pub entry: extern "C" fn(),      // entry function
    pub state: TaskState,
    pub priority: u8,
    pub stack_lo: *mut u8,           // diagnostics/overflow checks
    pub stack_hi: *mut u8,
}

static mut TCBS: [Option<Tcb>; MAX_TASKS] = [const { None }; MAX_TASKS];
static mut NUM_TASKS: usize = 0;

unsafe fn carve_task_stack(id: usize) -> (*mut u8, *mut u8) {
    let base = &__task_stack_start as *const u8 as usize;
    let end  = &__task_stack_end   as *const u8 as usize;
    let total = end - base;

    assert!(MAX_TASKS * TASK_STACK_BYTES <= total, ".tasks too small for MAX_TASKS");

    let lo = base + id * TASK_STACK_BYTES;
    let hi = lo + TASK_STACK_BYTES;
    (lo as *mut u8, hi as *mut u8)
}

unsafe fn build_initial_frame(stack_hi: *mut u8, entry: extern "C" fn()) -> *mut usize {
    // 16-byte align, then reserve frame
    let mut sp = ((stack_hi as usize) & !0xF) as *mut usize;
    sp = sp.sub(FRAME_WORDS);

    // zero the frame (deterministic)
    for i in 0..FRAME_WORDS {
        core::ptr::write(sp.add(i), 0usize);
    }

    // set return PC for mret
    core::ptr::write(sp.offset(OFF_MEPC), entry as usize);

    sp
}

pub unsafe fn create_task(entry: extern "C" fn(), priority: u8) -> usize {
    assert!(NUM_TASKS < MAX_TASKS);
    let id = NUM_TASKS;
    NUM_TASKS += 1;

    let (lo, hi) = carve_task_stack(id);
    let sp = build_initial_frame(hi, entry);

    TCBS[id] = Some(Tcb {
        sp,
        entry,
        state: TaskState::Ready,
        priority,
        stack_lo: lo,
        stack_hi: hi,
    });

    id
}

// Scheduler round-robin

static mut CURRENT: usize = 0;

fn next_ready(from: usize) -> usize {
    for i in 1..=MAX_TASKS {
        let idx = (from + i) % MAX_TASKS;
        if let Some(t) = unsafe { &TCBS[idx] } {
            if t.state == TaskState::Ready {
                return idx;
            }
        }
    }
    from // no other READY task ⇒ keep current
}

/// called from the trap context to save current SP and pick the next task
pub unsafe fn schedule(current_sp: *mut usize) -> *mut usize {
    // save current task’s SP
    if let Some(t) = &mut TCBS[CURRENT] {
        t.sp = current_sp;
        if t.state == TaskState::Running {
            t.state = TaskState::Ready;
        }
    }

    // pick next
    let next = next_ready(CURRENT);
    CURRENT = next;

    // return the SP of the next task
    if let Some(t) = &mut TCBS[CURRENT] {
        t.state = TaskState::Running;
        return t.sp;
    }

    // should not happen; fall back
    current_sp
}

unsafe extern "C" { fn __rtos_boot_with_sp(sp: *mut usize) -> !; }

pub unsafe fn start_first_task() -> ! {
    // pick first READY task and jump into it
    for i in 0..MAX_TASKS {
        if let Some(t) = &TCBS[i] {
            if t.state == TaskState::Ready {
                CURRENT = i;
                // mark running and jump
                let t_mut = TCBS[i].as_mut().unwrap();
                t_mut.state = TaskState::Running;
                __rtos_boot_with_sp(t_mut.sp);
            }
        }
    }
    loop {}
}



// Trap handler

unsafe extern "C" {
    fn trap_entry(); // from trap.S
}

#[no_mangle]
pub extern "C" fn trap_handler(sp: *mut usize) -> *mut usize{
    unsafe {
        let mcause_val = *sp.offset(OFF_MCAUSE);
        let mepc_val   = *sp.offset(OFF_MEPC);
        let mtval_val  = *sp.offset(OFF_MTVAL);

        //puts("Trap handler invoked!");
        //puts("mcause: "); put_hex(mcause_val); puts("\n");
        //puts("mepc: ");   put_hex(mepc_val);   puts("\n");
        //puts("mtval: ");  put_hex(mtval_val);  puts("\n");

        // Manual decode (portable across crate versions)
        let xlen_msb = (core::mem::size_of::<usize>() * 8 - 1) as u32;
        let is_interrupt = ((mcause_val >> xlen_msb) & 1) != 0;
        let code = mcause_val & ((1usize << xlen_msb) - 1);

        if is_interrupt {
            //puts("Interrupt detected\n");
            match code {
                7  => { 
                    timer_interrupt();
                    return schedule(sp);
                },     // Machine Timer
                11 => {
                    let need = external_interrupt();
                    if need { return schedule(sp); }
                    return sp;
                },  // Machine External
                _  => panic!("Generic Interrupt"),
            }
        } else {
            //puts("Exception detected\n");
            match code {
                2 => handle_illegal(),       // Illegal Instruction
                5 => handle_mem_fault(),     // Load access fault
                7 => handle_mem_fault(),     // Store/AMO access fault
                11 => { // ECALL from M-mode = task_yield()
                    // skip the ecall instruction so we don't re-execute it after mret
                    *sp.offset(OFF_MEPC) = mepc_val.wrapping_add(4);
                    return schedule(sp);   // always reschedule on voluntary yield
                }
                _ => panic!("Generic Exception"),
            }
            return sp;
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

// re-arms the timer
fn timer_interrupt() {
    unsafe {
        // bump RTOS tick
        rtos_on_timer_tick();

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
    // --- lock down interrupts so we don't re-enter while printing ---
    unsafe {
        mstatus::clear_mie(); // global mask

        mie::clear_mext();    // MEIE
        mie::clear_mtimer();  // MTIE
        mie::clear_msoft();   // MSIE   <-- this is the one you want
    }

    puts("=== PANIC ===\n");

    if let Some(location) = info.location() {
        puts("File: "); puts(location.file()); puts("\n");
        puts("Line: "); put_dec(location.line() as usize); puts("\n");
    }

    let cause = mcause::read();
    let pc    = mepc::read();
    let tval  = mtval::read(); // extra fault info (addr or instruction)

    puts("Raw mcause bits: 0x"); put_hex(cause.bits()); puts("\n");

    match cause.cause() {
        mcause::Trap::Exception(code) => {
            puts("Cause: Exception (code "); put_dec(code as usize); puts(")\n");
            // Disambiguate common exception codes
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
            // Disambiguate common interrupt codes
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

    // Stay halted deterministically
    loop { core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst); }
}

fn external_interrupt() -> bool {
    // 1) PLIC claim -> get irq_id
    // 2) Minimal driver work (ack device, move a byte, set a semaphore, etc.)
    // 3) If that wakes a higher-priority task: return true
    // 4) PLIC complete(irq_id)
    false
}

fn handle_illegal() {
    panic!("Illegal instruction trapped");
}

fn handle_mem_fault() {
    panic!("Memory access fault");
}

// ===== kernel_services.rs (can live at bottom of main.rs) =====
use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering::{Acquire, Release, AcqRel, Relaxed}};

/// Your timer currently re-arms with +10_000 on QEMU virt (mtime ≈ 10 MHz) ⇒ 1 ms tick.
pub const TICK_HZ: u64 = 1000;

// ---------- timebase ----------
static TICKS: AtomicU64 = AtomicU64::new(0);

#[inline] pub fn ticks() -> u64 { TICKS.load(Relaxed) }

#[inline]
pub const fn ms_to_ticks(ms: u64) -> u64 { ((ms * TICK_HZ) + 999) / 1000 }

/// Called from the timer interrupt.
#[no_mangle]
pub extern "C" fn rtos_on_timer_tick() {
    TICKS.fetch_add(1, Relaxed);
}

// ---------- yield ----------
#[inline(always)]
pub fn task_yield() {
    unsafe { core::arch::asm!("ecall", options(nostack, nomem)) }
}

// ---------- delay ----------
pub fn delay_ms(ms: u64) {
    let deadline = ticks().wrapping_add(ms_to_ticks(ms));
    while (ticks().wrapping_sub(deadline) as i64) < 0 {
        task_yield();
    }
}

// ---------- SpinLock ----------
pub struct SpinLock<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}
unsafe impl<T: ?Sized + Send> Send for SpinLock<T> {}
unsafe impl<T: ?Sized + Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self { locked: AtomicBool::new(false), data: UnsafeCell::new(value) }
    }
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self.locked.compare_exchange(false, true, AcqRel, Acquire).is_err() {
            for _ in 0..32 { spin_loop(); }
            task_yield();
        }
        SpinLockGuard { lock: self }
    }
}
pub struct SpinLockGuard<'a, T: ?Sized> { lock: &'a SpinLock<T> }
impl<'a, T: ?Sized> core::ops::Deref for SpinLockGuard<'a, T> {
    type Target = T; fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
}
impl<'a, T: ?Sized> core::ops::DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T { unsafe { &mut *self.lock.data.get() } }
}
impl<'a, T: ?Sized> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) { self.lock.locked.store(false, Release); }
}

// ---------- CountingSemaphore (cooperative) ----------
pub struct Semaphore { count: AtomicIsize }
impl Semaphore {
    pub const fn new(initial: isize) -> Self { Self { count: AtomicIsize::new(initial) } }
    pub fn wait(&self) {
        loop {
            let c = self.count.load(Acquire);
            if c > 0 && self.count.compare_exchange(c, c - 1, AcqRel, Acquire).is_ok() {
                return;
            }
            task_yield();
        }
    }
    pub fn post(&self) { self.count.fetch_add(1, Release); }
    pub fn try_wait(&self) -> bool {
        let mut c = self.count.load(Acquire);
        while c > 0 {
            match self.count.compare_exchange(c, c - 1, AcqRel, Acquire) {
                Ok(_) => return true,
                Err(cur) => c = cur,
            }
        }
        false
    }
}

// Main

/* 
#[entry]
fn main() -> ! {
    
    unsafe {

        let base = &__task_stack_start as *const u8 as usize;
        let end  = &__task_stack_end   as *const u8 as usize;
        puts(".tasks base: 0x"); put_hex(base); puts("\n");
        puts(".tasks end : 0x"); put_hex(end ); puts("\n");
        puts(".tasks size: 0x"); put_hex(end - base); puts("\n");


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
        //let p: *mut u64 = 0x100 as *mut u64; // low unmapped address
        //core::ptr::read_volatile(p);
        
        extern "C" fn task1() {
            loop { puts("[task1]\n"); for _ in 0..20000 { core::hint::spin_loop(); } }
        }
        extern "C" fn task2() {
            loop { puts("[task2]\n"); for _ in 0..20000 { core::hint::spin_loop(); } }
        }

        let _t1 = create_task(task1, 1);
        let _t2 = create_task(task2, 1);
        start_first_task();
    }

}
*/

static UART_LOCK: SpinLock<()> = SpinLock::new(());
static SEM: Semaphore = Semaphore::new(0);

#[entry]
fn main() -> ! {
    unsafe {
        // Show .tasks region (as you already had)
        let base = &__task_stack_start as *const u8 as usize;
        let end  = &__task_stack_end   as *const u8 as usize;
        puts(".tasks base: 0x"); put_hex(base); puts("\n");
        puts(".tasks end : 0x"); put_hex(end ); puts("\n");
        puts(".tasks size: 0x"); put_hex(end - base); puts("\n");

        // Set trap entry and start timer (1ms tick with +10_000 @ 10MHz)
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
                // use the UART lock so lines don’t interleave mid-print
                let _g = UART_LOCK.lock();
                puts("[500ms] tick=");
                put_dec(ticks() as usize);
                puts("\n");
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
                SEM.wait(); // cooperative: yields while waiting
                let _g = UART_LOCK.lock();
                puts("[cons] got token at tick ");
                put_dec(ticks() as usize);
                puts("\n");
                drop(_g);
            }
        }

        extern "C" fn yield_spammer() {
            loop {
                {
                    let _g = UART_LOCK.lock();
                    puts("[yield]\n");
                }
                task_yield(); // voluntary context switch via ECALL
            }
        }

        // Create tasks (all same priority → round-robin + preemption by timer)
        let _t0 = create_task(periodic_500ms,   1);
        let _t1 = create_task(producer_200ms,   1);
        let _t2 = create_task(consumer_blocking,1);
        let _t3 = create_task(yield_spammer,    1);

        start_first_task();
    }
}
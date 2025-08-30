use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::sync::atomic::{
    AtomicBool, AtomicIsize, AtomicU64,
    Ordering::{Acquire, Release, AcqRel, Relaxed}
};

// Timer re-arms with +10_000 on QEMU virt (mtime =10 MHz) ⇒ 1 ms tick. Logic tick
pub const TICK_HZ: u64 = 1000;

// ---------- timebase ----------
static TICKS: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn ticks() -> u64 { TICKS.load(Relaxed) }

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
    type Target = T;
    fn deref(&self) -> &T { unsafe { &*self.lock.data.get() } }
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
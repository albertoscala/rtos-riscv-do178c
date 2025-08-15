# Software Design Description (SDD)
**Project:** Minimal RTOS in Rust for RISC‑V (M‑mode)  
**Date:** 2025-08-15

## 1. Overview
A small cooperative RTOS providing: system tick, task management, semaphore/lock primitives, UART console, and trap/panic handling. Assembly glue supplies trap entry and first task bootstrap.

## 2. Architecture
### 2.1 Module Structure
- **arch/**
  - `mod.rs` — trap frame constants and externs (`trap_entry`, `__rtos_boot_with_sp`, task stack symbols).  
  - `trap.rs` — `trap_handler` parses `mcause`/`mepc`/`mtval` and dispatches to timer ISR or fault handlers.  
  - `timer.rs` — `init_timer(ticks)`, `timer_interrupt()`.
- **kernel/**
  - `task.rs` — task control blocks, creation, ready/run state, first task start, scheduler helpers.  
  - `services.rs` — tick counter, `delay_ms`, `task_yield`, `Semaphore`, `SpinLock`.
- **drivers/**
  - `uart.rs` — polled TX (`puts`, `put_hex`, `put_dec`).
- **panic_handler.rs** — panic routine with interrupt lockdown and diagnostics.
- **main.rs** — demo workload, UART prints, timer init, task creation, start.

### 2.2 Control Flow
1. **Boot**: assembly startup → Rust `main` (or `entry`) → `init_timer(+ticks)` → task creation → `start_first_task()` → `__rtos_boot_with_sp`.  
2. **Tick**: timer interrupt → `timer_interrupt()` → `rtos_on_timer_tick()` (increments ticks, unblocks delays) → return to preempted context (cooperative scheduling otherwise).  
3. **Yield/Block**: tasks call `task_yield()` or `delay_ms()`/`Semaphore::wait()`, transitioning state to **Blocked/Ready**.  
4. **Trap/Fault**: `trap_handler` distinguishes timer vs. external vs. exceptions; faults call panic.

### 2.3 Data Design
- **TCB (`task.rs`)**  
  Fields: `sp: *mut usize`, `entry: extern "C" fn()`, `state: TaskState`, `priority: u8`, `stack_lo/hi`.  
  Limits: `MAX_TASKS = 4`, `TASK_STACK_BYTES = 4096`.
- **Global Timebase (`services.rs`)**  
  `TICKS: AtomicU64`; `TICK_HZ = 1000` (1 ms).  
- **Sync Primitives**  
  - `SpinLock<T>` with interior mutability for short CS.  
  - `Semaphore` with atomic count and blocking wait.

### 2.4 Interfaces
- **Public Functions**  
  - Time: `ticks()`, `delay_ms(ms: u64)`, `task_yield()`  
  - Tasks: `create_task(entry, prio) -> handle`, `start_first_task()`  
  - Sync: `Semaphore::new(n)`, `wait()`, `post()`, `try_wait()`; `SpinLock::new(v)`, `lock()`  
  - UART: `puts(&str)`, `put_hex(usize)`, `put_dec(usize)`  
- **ISR/Traps**  
  - `timer_interrupt()` called from `trap_handler` on timer cause.  
  - Panic path prints cause, `mepc`, `mtval` and halts.

### 2.5 Timing
- Tick period: 1 ms (QEMU `virt` MTIME ≈10 MHz, rearm +10_000).  
- ISR WCET: O(1) rearm plus wakeups; no dynamic allocation.

### 2.6 Memory
- Per‑task stack: 4096 B (configurable).  
- Task stacks placed in linker‑defined `.tasks` region (`__task_stack_start/end`).  
- Trap frame size: `FRAME_WORDS = 20` (saved registers in assembly).

### 2.7 Error Handling
- Illegal instruction or memory fault → panic.  
- Panic: interrupts disabled, message emitted, infinite loop.

### 2.8 Design Rationale
- Cooperative scheduling simplifies assurance arguments and determinism.  
- Polled UART avoids driver complexity.  
- Atomics used for time and semaphores to avoid disabling interrupts in long sections.

## 3. Detailed Design by Requirement (Traceability)
| Req | Design Element(s) | Source File(s) |
|-----|-------------------|----------------|
| HLR‑1 | Tick maintenance, rearm | `timer.rs`, `services.rs` |
| HLR‑2 | TCB, scheduler helpers | `task.rs` |
| HLR‑3 | Task API & bootstrap | `task.rs`, `mod.rs` |
| HLR‑4 | Delay service | `services.rs` |
| HLR‑5 | Semaphore | `services.rs` |
| HLR‑6 | SpinLock | `services.rs` |
| HLR‑7 | UART TX | `uart.rs` |
| HLR‑8 | Trap & panic | `trap.rs`, `panic_handler.rs` |
| HLR‑9 | Timer init & first task | `timer.rs`, `task.rs` |
| HLR‑10| Arch externs | `mod.rs` |

## 4. Assumptions and TBDs
- DAL level to be assigned.  
- External interrupts/PLIC integration not yet implemented (stub returns false).  
- Assembly context switch (`trap_entry`, save/restore) provided separately and must match `OFF_*` layout.

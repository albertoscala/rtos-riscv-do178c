# Software Requirements Data (SRD)
**Project:** Minimal RTOS in Rust for RISC‑V (M‑mode)  
**Standard:** DO‑178C (objectives addressed to the extent applicable for a minimal RTOS)  
**Date:** 2025-08-15

## 1. Purpose and Scope
This SRD captures software requirements for a minimal real‑time operating system (RTOS) targeting RISC‑V (RV64) in Machine mode on the QEMU `virt` platform. The RTOS provides a tiny kernel with cooperative/task‑yield scheduling, a 1 ms system tick, basic inter‑task synchronization, and UART console I/O for observability. The goal is to express **clear, verifiable, and traceable** requirements aligned with DO‑178C principles.

**Out‑of‑scope:** memory protection, MMU/MPU partitioning, file systems, networking stacks, and user mode. Assembly bootstrap and context switch stubs are provided separately and are not part of this SRD except as interfaces.

## 2. References
- Source files: main.rs, panic_handler.rs, task.rs, services.rs, uart.rs, trap.rs, timer.rs, mod.rs
- Hardware: QEMU `virt` machine; CLINT/MTIME and UART0 MMIO.
- DO‑178C, RTCA/EUROCAE (normative guidance).

## 3. Operational Environment and Constraints
- CPU: RISC‑V RV64, Machine mode.  
- Timer: CLINT MTIME/MTIMECMP at approximately 10 MHz on QEMU `virt`.  
- UART0 base: **0x1000_0000**; TX register at offset 0; LSR at offset 5. (See `uart.rs`.)
- Interrupts: Machine timer interrupts used for kernel tick; external/soft interrupts reserved. (See `timer.rs`, `trap.rs`.)
- Build: `no_std`, bare‑metal, Rust.

## 4. Definitions
- **Tick:** Kernel timebase unit (1 ms).  
- **Task:** Function started by the kernel with its own stack and TCB.  
- **TCB:** Task Control Block holding SP, entry, state, priority, and stack bounds.  
- **Blocking:** Task awaiting an event (e.g., semaphore wait or delay).

## 5. High‑Level Requirements (HLR)
**HLR‑1 Clock/Tick** — The RTOS shall maintain a monotonic tick counter advancing every **1 ms** using the machine timer interrupt.  
**HLR‑2 Scheduler** — The RTOS shall schedule up to **4** tasks (configurable constant) and run exactly one at a time.  
**HLR‑3 Task API** — The RTOS shall provide APIs to create a task, yield the processor, and start the first task.  
**HLR‑4 Delay** — The RTOS shall provide a millisecond delay service that blocks the calling task until the requested time elapses.  
**HLR‑5 Sync (Semaphore)** — The RTOS shall provide a counting semaphore with `post`, blocking `wait`, and non‑blocking `try_wait`.  
**HLR‑6 Locking (SpinLock)** — The RTOS shall provide a simple spinlock for short critical sections.  
**HLR‑7 Console I/O** — The RTOS shall provide UART functions to write strings and print hex/decimal values.  
**HLR‑8 Trap/Panic Handling** — The RTOS shall provide a trap handler that distinguishes exceptions/interrupts and a panic handler that disables interrupts, reports cause/PC, and halts.  
**HLR‑9 Deterministic Startup** — The RTOS shall initialize timer hardware and start the first task deterministically from a clean context.  
**HLR‑10 Portability Hooks** — The RTOS shall expose architecture interfaces for trap entry and context boot (`trap_entry`, `__rtos_boot_with_sp`) implemented in assembly.

## 6. Low‑Level Requirements (LLR)
**LLR‑1 Tick Frequency** — The kernel shall rearm `MTIMECMP` by **+10_000** counts on QEMU `virt`, generating a 1 ms tick (≈10 MHz timebase). (`timer.rs`)  
**LLR‑2 Tick Counter** — `ticks()` shall return the current 64‑bit tick count. (`services.rs`)  
**LLR‑3 Delay Conversion** — `delay_ms(ms)` shall block until `ticks() >= t0 + ms`. (`services.rs`)  
**LLR‑4 Task Limits** — `MAX_TASKS` = **4**; `TASK_STACK_BYTES` = **4096**. (`task.rs`)  
**LLR‑5 TCB Content** — Each task TCB shall store SP (to saved trap frame), entry fn pointer, state, priority, and stack bounds. (`task.rs`)  
**LLR‑6 First Task Start** — `start_first_task()` shall locate the first Ready task, mark it Running, and branch via `__rtos_boot_with_sp`. (`task.rs`, `mod.rs`)  
**LLR‑7 Yield** — `task_yield()` shall voluntarily relinquish the CPU to the next Ready task of equal priority. (`services.rs`, `task.rs`)  
**LLR‑8 Semaphore Correctness** — `wait()` shall block when count==0; `post()` increments; `try_wait()` decrements only if count>0. (`services.rs`)  
**LLR‑9 SpinLock** — `SpinLock::lock()` shall busy‑wait until acquired and release on drop. (`services.rs`)  
**LLR‑10 UART TX** — `puts`, `put_hex`, `put_dec` shall poll LSR and write bytes to THR. (`uart.rs`)  
**LLR‑11 Trap Decode** — `trap_handler` shall read saved `mcause`, `mepc`, `mtval` and dispatch timer vs. external vs. fault. (`trap.rs`)  
**LLR‑12 Timer ISR** — `timer_interrupt()` shall call `rtos_on_timer_tick()` and rearm `MTIMECMP` by **+10_000**. (`timer.rs`)  
**LLR‑13 Panic Actions** — On panic, the system shall disable interrupts, print diagnostic info including cause and PC, and halt. (`panic_handler.rs`)  
**LLR‑14 Frame Layout** — The trap frame layout and offsets (`OFF_*`) shall match the assembly prologue/epilogue. (`mod.rs`)

## 7. Interfaces
- **Public services:** `ticks`, `delay_ms`, `task_yield`, `Semaphore`, `SpinLock`, UART printing.  
- **Architecture:** `trap_entry` (vector), `__rtos_boot_with_sp` (first dispatch), timer init/ISR.  
- **Configuration:** constants `TICK_HZ`, `MAX_TASKS`, `TASK_STACK_BYTES`.

## 8. Safety/Design Assurance Considerations
- Intended assurance level: **placeholder** (update DAL per system safety assessment).  
- Defensive measures: panic path locks interrupts and reports state; illegal instructions/memory faults escalate to panic.  
- WCET/latency: spinlocks are short; timer ISR rearm is O(1). Delay and semaphores are blocking.

## 9. Verification
Each HLR/LLR is mapped to verification cases in the SVCP. Pass/fail criteria are objective and observable from UART output and/or instrumentation.

## 10. Traceability (Excerpt)
| Req ID | Design & Code Reference | Verification |
|-------|--------------------------|--------------|
| HLR‑1 | `timer.rs::init_timer`, `timer_interrupt`; `services.rs::rtos_on_timer_tick` | SV‑1, SV‑2 |
| HLR‑2 | `task.rs` scheduler | SV‑3, SV‑4 |
| HLR‑5 | `services.rs::Semaphore` | SV‑6 |
| LLR‑12| `timer.rs::timer_interrupt` | SV‑2 |
(See full matrices in SDD/SVCP.)


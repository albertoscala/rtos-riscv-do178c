# Software Verification Cases and Procedures (SVCP)
**Project:** Minimal RTOS in Rust for RISC‑V (M‑mode)  
**Date:** 2025-08-15

## 1. Strategy
Verification emphasizes **requirements‑based testing**, structural coverage where practical for Rust `no_std`, and conformance to the designed interfaces. Tests run under QEMU `virt` and observe UART output and timing.

## 2. Test Environment
- Platform: QEMU RISC‑V `virt`, RV64, machine mode.  
- Build: Rust stable `no_std`, linker script placing task stacks in `.tasks`.  
- Observation: UART0 TX captured by QEMU stdio.  
- Timebase: MTIME ≈10 MHz; tick = 1 ms via +10_000 rearm.

## 3. Test Cases
**SV‑1 Tick Rate Calibration (HLR‑1, LLR‑1/12)**  
**Objective:** Verify 1 ms tick.  
**Procedure:** Initialize timer; print `ticks()` at t0 and after 1000 ms using `delay_ms(1000)`. Expect Δticks ≈ 1000 ±2.  
**Pass:** 998 ≤ Δticks ≤ 1002.

**SV‑2 ISR Rearm (LLR‑12)**  
**Objective:** Verify `timer_interrupt()` rearms by +10_000.  
**Procedure:** Instrument ISR to print `MTIMECMP` deltas for 3 successive ticks.  
**Pass:** All deltas equal 10_000.

**SV‑3 Task Creation and First Start (HLR‑2/3, LLR‑6)**  
**Objective:** Verify first Ready task starts via `__rtos_boot_with_sp`.  
**Procedure:** Create two tasks; ensure first prints a banner; confirm state transitions Ready→Running once.  
**Pass:** Observed banner from task0; no double‑start.

**SV‑4 Cooperative Yield (HLR‑2/3, LLR‑7)**  
**Objective:** Verify `task_yield()` switches to next Ready task of equal priority.  
**Procedure:** Two equal‑priority tasks alternately print `[yield]` markers after calling `task_yield()`.  
**Pass:** Alternating sequence with no starvation.

**SV‑5 Delay Blocking (HLR‑4, LLR‑3)**  
**Objective:** Verify `delay_ms` blocks for requested time.  
**Procedure:** Task prints timestamps before/after `delay_ms(200)`.  
**Pass:** Δticks in [198, 205].

**SV‑6 Semaphore Correctness (HLR‑5, LLR‑8)**  
**Objective:** Verify `Semaphore::wait/post/try_wait`.  
**Procedure:** Producer posts every 200 ms; consumer blocks on wait; verify count never negative and consumer cadence ~200 ms.  
**Pass:** Consumer messages at ~200 ms intervals; `try_wait` true only when count>0.

**SV‑7 SpinLock Mutual Exclusion (HLR‑6, LLR‑9)**  
**Objective:** Verify critical section protection.  
**Procedure:** Two tasks contend for a shared counter under `SpinLock`; confirm no lost updates after N increments.  
**Pass:** Final counter equals expected N_total.

**SV‑8 UART Output (HLR‑7, LLR‑10)**  
**Objective:** Verify `puts`, `put_hex`, `put_dec`.  
**Procedure:** Print known values; compare to expected byte stream.  
**Pass:** Exact match.

**SV‑9 Trap Decode and Panic (HLR‑8, LLR‑11/13)**  
**Objective:** Verify trap path and panic diagnostics.  
**Procedure:** Intentionally execute illegal instruction and invalid load in a test build.  
**Pass:** UART shows cause text, `mepc`, `mtval`, and the system halts (no further task prints).

**SV‑10 Frame Layout Consistency (LLR‑14)**  
**Objective:** Verify `OFF_*` constants match assembly frame.  
**Procedure:** Unit test in assembly stores sentinels; Rust reads via offsets; values match.  
**Pass:** All sentinel reads correct.

## 4. Procedures (General)
1. Build firmware (release).  
2. Launch QEMU with UART on stdio.  
3. Reset between tests; run one test per image when destructive.  
4. Capture UART log; check automatic predicates (regex/thresholds).

## 5. Requirements Traceability Matrix (RTM)
| Requirement | Test Case(s) |
|-------------|---------------|
| HLR‑1 | SV‑1, SV‑2 |
| HLR‑2 | SV‑3, SV‑4 |
| HLR‑3 | SV‑3, SV‑4 |
| HLR‑4 | SV‑5 |
| HLR‑5 | SV‑6 |
| HLR‑6 | SV‑7 |
| HLR‑7 | SV‑8 |
| HLR‑8 | SV‑9 |
| HLR‑9 | SV‑3 |
| HLR‑10 | SV‑10 |
| LLR‑1/12 | SV‑1, SV‑2 |
| LLR‑2 | SV‑1 |
| LLR‑3 | SV‑5 |
| LLR‑4 | SV‑3 |
| LLR‑5 | SV‑3, SV‑4 |
| LLR‑6 | SV‑3 |
| LLR‑7 | SV‑4 |
| LLR‑8 | SV‑6 |
| LLR‑9 | SV‑7 |
| LLR‑10 | SV‑8 |
| LLR‑11 | SV‑9 |
| LLR‑13 | SV‑9 |
| LLR‑14 | SV‑10 |

## 6. Coverage and Robustness
- Aim for statement/branch coverage of kernel Rust code (assembly excluded).  
- Robustness: illegal/faulting ops, semaphore underflow prevention, lock release on drop.

## 7. Anomalies and Open Items
- External interrupt path (`external_interrupt`) is stubbed; tests marked **TBD**.  
- DAL assignment and independence levels to be finalized.

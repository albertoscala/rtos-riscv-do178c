# Minimal RTOS for RISC-V under DO-178C - Thesis Project

A small cooperative real-time operating system targeting **RISC-V RV64** in Machine mode, written in **Rust** with assembly context-switch glue. Developed as a Computer Science thesis following the **DO-178C** software assurance standard (objectives applied to the extent relevant for an academic project).

The system runs on the QEMU `virt` machine (no hardware required) and demonstrates a complete safety-critical software lifecycle: requirements → design → implementation → formal model → verification.

---

## What it does

- **Preemptive tick + cooperative scheduling** - 1 ms system tick via CLINT machine timer; tasks yield explicitly or block on delays/semaphores; the timer ISR forces a reschedule at each tick
- **Task management** - up to 4 tasks, each with its own 4 KiB stack carved from a linker-defined `.tasks` region; TCBs track state, priority, and stack bounds
- **Synchronisation primitives** - counting `Semaphore` (blocking `wait`, `post`, non-blocking `try_wait`) and `SpinLock<T>` with RAII guard
- **Trap & exception handling** - M-mode trap vector in assembly dispatches to a Rust handler; timer interrupt, `ecall`-based yield, illegal instruction and memory faults all handled
- **Panic handler** - disables interrupts, prints `mcause`, `mepc`, `mtval`, and halts deterministically
- **UART console** - polled TX on QEMU `virt` UART0 for observability

### Demo output (from `cargo run`)

```
.tasks base: 0x80020000
.tasks end : 0x80024000
.tasks size: 0x4000
TrapMode is actually Direct!
Trap Entry is correct!
[yield]
[500ms] tick=1
[yield]
[prod] +1
[cons] got token at tick 201
[yield]
[500ms] tick=501
...
```

---

## Architecture

```
┌──────────────────────────────────────┐
│              main.rs                 │  Demo tasks, timer init, task creation
├──────────────────────────────────────┤
│           kernel/                    │
│   task.rs      services.rs           │  TCB, scheduler | tick, delay, semaphore, spinlock
├──────────────────────────────────────┤
│           arch/                      │
│   trap.rs      timer.rs   mod.rs     │  Trap dispatch | CLINT | frame layout constants
├──────────────────────────────────────┤
│           drivers/                   │
│   uart.rs                            │  Polled MMIO UART
├──────────────────────────────────────┤
│      asm/trap.S    asm/boot.S        │  Context save/restore, first-task bootstrap
└──────────────────────────────────────┘
```

### Context switch

Context switch is handled entirely in `trap.S`. On every trap entry the full caller-saved register set + `mepc`/`mcause`/`mtval` is pushed onto the **current task's stack**, forming a 160-byte trap frame. `trap_handler` (Rust) receives a `*mut usize` to this frame and returns a (possibly different) task's frame pointer - the assembly epilogue then restores from whichever pointer it gets back, completing the switch.

`start_first_task` (`boot.S`) bootstraps the very first task by loading a pre-built synthetic frame directly, setting `MPIE=1` and `MPP=M`, then executing `mret`.

### Scheduling

Round-robin cooperative among `Ready` tasks of equal priority. `delay_ms` and `Semaphore::wait` set the calling task to `Blocked` and spin-yield until the condition is met. The timer ISR increments the global tick counter and calls `schedule()`, ensuring even a compute-bound task gets preempted at worst every 1 ms.

---

## DO-178C Artifacts

This project produces the documentation artefacts expected by DO-178C for a minimal DAL-? system:

| Artefact | Location | Description |
|---|---|---|
| Software Requirements Data (SRD) | [`documents/english/SRD_Minimal_RTOS_DO178C_EN.md`](documents/english/SRD_Minimal_RTOS_DO178C_EN.md) | High-level (HLR) and low-level (LLR) requirements with traceability |
| Software Design Description (SDD) | [`documents/english/SDD_Minimal_RTOS_DO178C_EN.md`](documents/english/SDD_Minimal_RTOS_DO178C_EN.md) | Architecture, module breakdown, data design, rationale |
| Software Verification Cases & Procedures (SVCP) | [`documents/english/SVCP_Minimal_RTOS_DO178C_EN.md`](documents/english/SVCP_Minimal_RTOS_DO178C_EN.md) | 10 test cases mapped to requirements (RTM) |
| TLA⁺ Formal Model | [`verification/model/scheduler.tla`](verification/model/scheduler.tla) | Safety and liveness properties of the scheduler state machine |

Italian versions of the SRD, SDD, and SVCP are also available under [`documents/italian/`](documents/italian/).

### TLA⁺ scheduler model

The file `verification/model/scheduler.tla` encodes the scheduler as a state machine and verifies:

- **Safety**: at most one task is `running` at any time
- **Liveness**: every `ready` task eventually gets to run (no starvation under weak fairness)

The model was checked with [TLC](https://github.com/tlaplus/tlaplus) for `MAX_TASKS ∈ {2, 3, 4}`.

---

## Building & running

### Prerequisites

```
rustup target add riscv64imac-unknown-none-elf
# QEMU for the runner:
# macOS:  brew install qemu
# Ubuntu: sudo apt install qemu-system-misc
```

### Run (QEMU)

```bash
cd code
cargo run --release
```

`cargo run` invokes QEMU automatically via the runner in `.cargo/config.toml`. Press `Ctrl-A X` to quit QEMU.

### Build only

```bash
cd code
cargo build --release
# ELF is at: target/riscv64imac-unknown-none-elf/release/rtos-riscv-do178c
```

### Inspect the binary

```bash
# Disassemble
rust-objdump -d target/riscv64imac-unknown-none-elf/release/rtos-riscv-do178c | less

# Section sizes
rust-size target/riscv64imac-unknown-none-elf/release/rtos-riscv-do178c
```

---

## Repository layout

```
.
├── code/                    # Rust firmware
│   ├── src/
│   │   ├── main.rs          # Entry point, demo tasks
│   │   ├── arch/            # Trap dispatch, timer, frame layout
│   │   ├── kernel/          # Scheduler (task.rs), services (services.rs)
│   │   ├── drivers/         # UART driver
│   │   └── panic_handler.rs # Deterministic panic path
│   ├── asm/
│   │   ├── trap.S           # Context save/restore (trap entry/exit)
│   │   └── boot.S           # First-task bootstrap (__rtos_boot_with_sp)
│   ├── memory.x             # Memory map (RAM origin, .tasks region)
│   ├── link.ld              # Top-level linker script
│   └── Cargo.toml
├── documents/
│   ├── english/             # SRD, SDD, SVCP (English)
│   └── italian/             # SRD, SDD, SVCP (Italian)
├── verification/
│   └── model/
│       └── scheduler.tla    # TLA⁺ formal model
└── thesis/
    ├── english/thesis.tex
    └── italian/tesi.tex
```

---

## Known limitations / future work

- `MAX_TASKS` and `TASK_STACK_BYTES` are compile-time constants; dynamic task creation is out of scope
- No stack overflow detection (guard pages / canaries would require MPU)
- `delay_ms` and semaphore `wait` are busy-yield loops - a proper sleep queue would reduce CPU usage
- The timer rearm constant `10_000` is hardcoded for QEMU's ≈10 MHz MTIME; a real target needs calibration
- Formal model covers the scheduler only; the semaphore and spinlock have not been model-checked

---

## Thesis

The full thesis document (LaTeX source) is in [`thesis/`](thesis/). It covers the motivation, DO-178C background, design decisions, and evaluation.

---

## License

MIT

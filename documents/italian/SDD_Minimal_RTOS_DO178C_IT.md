# Software Design Description (SDD)  
**Progetto:** Minimal RTOS in Rust per RISC-V (M-mode)  
**Data:** 2025-08-15  

---

## 1. Panoramica  
Un piccolo RTOS cooperativo che fornisce: tick di sistema, gestione task, primitive di sincronizzazione (semaforo/spinlock), console UART e gestione trap/panic. Il codice assembly fornisce l’entry point delle trap e il bootstrap del primo task.  

---

## 2. Architettura  

### 2.1 Struttura dei Moduli  
- **arch/**  
  - `mod.rs` — costanti del frame di trap e dichiarazioni `extern` (`trap_entry`, `__rtos_boot_with_sp`, simboli per gli stack dei task).  
  - `trap.rs` — `trap_handler` legge `mcause`/`mepc`/`mtval` e smista verso ISR del timer o gestori di fault.  
  - `timer.rs` — `init_timer(ticks)`, `timer_interrupt()`.  

- **kernel/**  
  - `task.rs` — Task Control Block, creazione task, stati ready/run, avvio primo task, funzioni di scheduling.  
  - `services.rs` — contatore tick, `delay_ms`, `task_yield`, `Semaphore`, `SpinLock`.  

- **drivers/**  
  - `uart.rs` — trasmissione UART in polling (`puts`, `put_hex`, `put_dec`).  

- **panic_handler.rs** — routine di panic con disabilitazione interrupt e diagnostica.  
- **main.rs** — workload dimostrativo, stampe UART, inizializzazione timer, creazione task e avvio.  

---

### 2.2 Flusso di Controllo  
1. **Boot**: startup in assembly → Rust `main` (o `entry`) → `init_timer(+ticks)` → creazione task → `start_first_task()` → `__rtos_boot_with_sp`.  
2. **Tick**: interrupt del timer → `timer_interrupt()` → `rtos_on_timer_tick()` (incrementa tick, sblocca delay) → ritorno al contesto preempted (scheduling cooperativo).  
3. **Yield/Block**: i task chiamano `task_yield()` o `delay_ms()`/`Semaphore::wait()`, passando allo stato **Blocked/Ready**.  
4. **Trap/Fault**: `trap_handler` distingue timer vs eccezioni; i fault portano a panic.  

---

### 2.3 Struttura dei Dati  
- **TCB (`task.rs`)**  
  Campi: `sp: *mut usize`, `entry: extern "C" fn()`, `state: TaskState`, `priority: u8`, `stack_lo/hi`.  
  Limiti: `MAX_TASKS = 4`, `TASK_STACK_BYTES = 4096`.  

- **Timebase Globale (`services.rs`)**  
  `TICKS: AtomicU64`; `TICK_HZ = 1000` (1 ms).  

- **Primitive di Sincronizzazione**  
  - `SpinLock<T>` con mutabilità interna per sezioni critiche brevi.  
  - `Semaphore` con contatore atomico e attesa bloccante.  

---

### 2.4 Interfacce  
- **Funzioni Pubbliche**  
  - Tempo: `ticks()`, `delay_ms(ms: u64)`, `task_yield()`  
  - Task: `create_task(entry, prio) -> handle`, `start_first_task()`  
  - Sync: `Semaphore::new(n)`, `wait()`, `post()`, `try_wait()`; `SpinLock::new(v)`, `lock()`  
  - UART: `puts(&str)`, `put_hex(usize)`, `put_dec(usize)`  

- **ISR/Trap**  
  - `timer_interrupt()` invocata da `trap_handler` su causa timer.  
  - Il percorso di panic stampa causa, `mepc`, `mtval` e ferma il sistema.  

---

### 2.5 Temporizzazione  
- Periodo tick: 1 ms (QEMU `virt` MTIME ≈10 MHz, riarmo +10_000).  
- WCET ISR: O(1) (riarmo + sblocco), nessuna allocazione dinamica.  

---

### 2.6 Memoria  
- Stack per task: 4096 B (configurabile).  
- Stack dei task allocati nella sezione `.tasks` definita dal linker (`__task_stack_start/end`).  
- Dimensione trap frame: `FRAME_WORDS = 20` (registri salvati in assembly) (19 usati, 1 di padding).  

---

### 2.7 Gestione Errori  
- Istruzione illegale o fault memoria → panic.  
- Panic: interrupt disabilitati, messaggio emesso, loop infinito.  

---

### 2.8 Rationale di Progetto  
- Scheduling cooperativo semplifica l’analisi di determinismo e safety.  
- UART in polling riduce la complessità del driver.  
- Uso di atomiche per tick e semafori, evitando di disabilitare interrupt per sezioni lunghe.  

---

## 3. Design Dettagliato per Requisito (Tracciabilità)  
| Req | Elemento di Design | File Sorgente |
|-----|--------------------|---------------|
| HLR-1 | Tick e riarmo | `timer.rs`, `services.rs` |
| HLR-2 | TCB, scheduler helpers | `task.rs` |
| HLR-3 | API Task & bootstrap | `task.rs`, `mod.rs` |
| HLR-4 | Servizio Delay | `services.rs` |
| HLR-5 | Semaforo | `services.rs` |
| HLR-6 | SpinLock | `services.rs` |
| HLR-7 | UART TX | `uart.rs` |
| HLR-8 | Trap & panic | `trap.rs`, `panic_handler.rs` |
| HLR-9 | Timer init & primo task | `timer.rs`, `task.rs` |
| HLR-10| Externs architetturali | `mod.rs` |  

---

## 4. Assunzioni  
- **DAL**: livello da assegnare (per questo progetto non viene attribuito, contesto dimostrativo/accademico).  
- **Context switch assembly** (`trap_entry`, salvataggio/ripristino registri) fornito separatamente e coerente con i valori `OFF_*`. Verificato: 19 registri salvati, 1 slot extra riservato (padding/allineamento).  

---
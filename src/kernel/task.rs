use crate::arch::{FRAME_WORDS, OFF_MEPC, __rtos_boot_with_sp, __task_stack_end, __task_stack_start};

pub const MAX_TASKS: usize = 4;
pub const TASK_STACK_BYTES: usize = 4096; // tune per analysis

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState { Ready, Running, Blocked }

#[repr(C)]
pub struct Tcb {
    pub sp: *mut usize,              // saved SP → trap frame
    pub entry: extern "C" fn(),      // entry function
    pub state: TaskState,
    pub priority: u8,
    pub stack_lo: *mut u8,           // diagnostics/overflow checks
    pub stack_hi: *mut u8,
}

static mut TCBS: [Option<Tcb>; MAX_TASKS] = [const { None }; MAX_TASKS];
static mut NUM_TASKS: usize = 0;
static mut CURRENT: usize = 0;

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

fn next_ready(from: usize) -> usize {
    for i in 1..=MAX_TASKS {
        let idx = (from + i) % MAX_TASKS;
        if let Some(t) = unsafe { &TCBS[idx] } {
            if t.state == TaskState::Ready {
                return idx;
            }
        }
    }
    from // none → keep current
}

/// Called from trap context: save current SP and pick next task.
pub unsafe fn schedule(current_sp: *mut usize) -> *mut usize {
    if let Some(t) = &mut TCBS[CURRENT] {
        t.sp = current_sp;
        if t.state == TaskState::Running {
            t.state = TaskState::Ready;
        }
    }

    let next = next_ready(CURRENT);
    CURRENT = next;

    if let Some(t) = &mut TCBS[CURRENT] {
        t.state = TaskState::Running;
        return t.sp;
    }

    current_sp // should not happen
}

pub unsafe fn start_first_task() -> ! {
    for i in 0..MAX_TASKS {
        if let Some(t) = &TCBS[i] {
            if t.state == TaskState::Ready {
                CURRENT = i;
                let t_mut = TCBS[i].as_mut().unwrap();
                t_mut.state = TaskState::Running;
                __rtos_boot_with_sp(t_mut.sp);
            }
        }
    }
    loop {}
}
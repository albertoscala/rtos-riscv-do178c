    .section .text
    .globl __rtos_boot_with_sp
__rtos_boot_with_sp:
    mv   sp, a0
    ld   t0, 16*8(sp)
    csrw mepc, t0
    ld ra, 0*8(sp)
    ld t0, 1*8(sp)
    ld t1, 2*8(sp)
    ld t2, 3*8(sp)
    ld a0, 4*8(sp)
    ld a1, 5*8(sp)
    ld a2, 6*8(sp)
    ld a3, 7*8(sp)
    ld a4, 8*8(sp)
    ld a5, 9*8(sp)
    ld a6, 10*8(sp)
    ld a7, 11*8(sp)
    ld t3, 12*8(sp)
    ld t4, 13*8(sp)
    ld t5, 14*8(sp)
    ld t6, 15*8(sp)
    addi sp, sp, 160
    # ensure we return to Machine mode with interrupts enabled-after-mret
    li   t1, (3 << 11)     # MPP=11 (Machine)
    csrs mstatus, t1
    li   t1, (1 << 7)      # MPIE=1
    csrs mstatus, t1
    mret
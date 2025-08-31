/* memory.x — keeps it dead simple for QEMU virt */

MEMORY
{
  RAM : ORIGIN = 0x80000000, LENGTH = 16M
}

/* Link-time aliases that riscv-rt expects ------------- */
REGION_ALIAS("REGION_TEXT",   RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA",   RAM);
REGION_ALIAS("REGION_BSS",    RAM);
REGION_ALIAS("REGION_HEAP",   RAM);
REGION_ALIAS("REGION_STACK",  RAM);

/* Make the stack grow *down* from the top of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* extra-sections */
SECTIONS
{
  .tasks (NOLOAD) : ALIGN(16)
  {
    __task_stack_start = .;
    . += 16K;   /* 16 KiB total budget for all task stacks */
    __task_stack_end = .;
  } > RAM
} INSERT AFTER .bss;
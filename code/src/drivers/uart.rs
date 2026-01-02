/// QEMU ‘virt’ board UART0 base address
const UART0_BASE: usize = 0x1000_0000;
const UART_THR:  *mut u8   = UART0_BASE as *mut u8;           // Transmit Holding Reg (0x0)
const UART_LSR:  *const u8 = (UART0_BASE + 5) as *const u8;   // Line Status Reg      (0x5)
const LSR_TX_EMPTY: u8 = 1 << 5; // Bit 5 = THR & TSR empty

#[inline(always)]
pub fn mmio_putchar(byte: u8) {
    unsafe {
        while core::ptr::read_volatile(UART_LSR) & LSR_TX_EMPTY == 0 {}
        core::ptr::write_volatile(UART_THR, byte);
    }
}

pub fn puts(s: &str) {
    for c in s.bytes() {
        mmio_putchar(c);
    }
}

pub fn put_hex(mut val: usize) {
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
    for ch in buf[..i].iter().rev() { mmio_putchar(*ch); }
}

pub fn put_dec(mut val: usize) {
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
    for ch in buf[..i].iter().rev() { mmio_putchar(*ch); }
}
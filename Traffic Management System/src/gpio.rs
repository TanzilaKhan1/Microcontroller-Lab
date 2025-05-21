// gpio.rs
use core::ptr::{read_volatile, write_volatile};

pub const MODER: u32 = 0x00;
pub const OTYPER: u32 = 0x04;
pub const OSPEEDR: u32 = 0x08;
pub const PUPDR: u32 = 0x0C;
pub const IDR: u32 = 0x10;
pub const BSRR: u32 = 0x18;

/// Enable GPIO clock in RCC register (bits 0-7 control GPIOA-H, e.g.)
pub unsafe fn enable_gpio_clocks(rcc_ahb1enr_addr: u32, gpio_mask: u32) {
    let enr = rcc_ahb1enr_addr as *mut u32;
    write_volatile(enr, read_volatile(enr) | gpio_mask);
}

/// Configure pin as output (push-pull, high speed)
pub unsafe fn pin_to_output(port_base: u32, pin: u8) {
    let moder = (port_base + MODER) as *mut u32;
    let otyper = (port_base + OTYPER) as *mut u32;
    let ospeedr = (port_base + OSPEEDR) as *mut u32;

    let mut v = read_volatile(moder);
    v &= !(0b11 << (pin * 2));
    v |= 0b01 << (pin * 2); // output mode
    write_volatile(moder, v);

    let mut v = read_volatile(otyper);
    v &= !(1 << pin); // push-pull output
    write_volatile(otyper, v);

    let mut v = read_volatile(ospeedr);
    v &= !(0b11 << (pin * 2));
    v |= 0b10 << (pin * 2); // high speed
    write_volatile(ospeedr, v);
}

/// Configure pin as input with pull-up resistor
pub unsafe fn pin_to_input_pullup(port_base: u32, pin: u8) {
    let moder = (port_base + MODER) as *mut u32;
    let pupdr = (port_base + PUPDR) as *mut u32;

    let mut v = read_volatile(moder);
    v &= !(0b11 << (pin * 2)); // input mode (00)
    write_volatile(moder, v);

    let mut v = read_volatile(pupdr);
    v &= !(0b11 << (pin * 2));
    v |= 0b01 << (pin * 2); // pull-up (01)
    write_volatile(pupdr, v);
}

/// Drive pin high or low using BSRR register (atomic)
pub unsafe fn drive_pin(port_base: u32, pin: u8, on: bool) {
    let bsrr = (port_base + BSRR) as *mut u32;
    if on {
        write_volatile(bsrr, 1 << pin);
    } else {
        write_volatile(bsrr, 1 << (pin + 16));
    }
}

/// Read input data register bit (true if pin is HIGH)
pub unsafe fn read_pin(port_base: u32, pin: u8) -> bool {
    let idr = (port_base + IDR) as *const u32;
    (read_volatile(idr) & (1 << pin)) != 0
}

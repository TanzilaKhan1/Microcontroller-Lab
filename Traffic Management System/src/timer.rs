// timer.rs
use core::ptr::{read_volatile, write_volatile};
use cortex_m::asm;

const SYST_CSR: u32 = 0xE000_E010;
const SYST_RVR: u32 = 0xE000_E014;
const SYST_CVR: u32 = 0xE000_E018;

static mut SYSTEM_MS: u32 = 0;

pub unsafe fn systick_init(sysclk_hz: u32) {
    write_volatile(SYST_CSR as *mut u32, 0); // disable SysTick
    write_volatile(SYST_RVR as *mut u32, sysclk_hz / 1000 - 1); // reload for 1 ms ticks
    write_volatile(SYST_CVR as *mut u32, 0); // clear current value
    write_volatile(SYST_CSR as *mut u32, 1 | (1 << 2)); // enable SysTick with processor clock and interrupts disabled
}

/// Wait for `ms` milliseconds, polling the timer flag and calling `poll_fn` each iteration
pub fn delay_ms_poll(ms: u32, mut poll_fn: impl FnMut()) {
    for _ in 0..ms {
        while unsafe { read_volatile(SYST_CSR as *const u32) & (1 << 16) } == 0 {
            asm::nop();
        }
        unsafe { SYSTEM_MS = SYSTEM_MS.wrapping_add(1) };
        poll_fn();
    }
}

pub fn get_system_ms() -> u32 {
    unsafe { SYSTEM_MS }
}

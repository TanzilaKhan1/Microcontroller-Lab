#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use cortex_m::asm;
use cortex_m_rt::entry;

/* ----------------------  Register blocks  ------------------------- */
const RCC_BASE: u32 = 0x4002_3800;
const RCC_AHB1ENR: u32 = RCC_BASE + 0x30;

/* GPIO bases */
const GPIOA_BASE: u32 = 0x4002_0000;
const GPIOB_BASE: u32 = 0x4002_0400;
const GPIOC_BASE: u32 = 0x4002_0800;

/* GPIO offsets */
const MODER: u32 = 0x00;
const OTYPER: u32 = 0x04;
const OSPEEDR: u32 = 0x08;
const PUPDR: u32 = 0x0C;
const IDR: u32 = 0x10;
const BSRR: u32 = 0x18;

/* SysTick */
const SYST_CSR: u32 = 0xE000_E010;
const SYST_RVR: u32 = 0xE000_E014;
const SYST_CVR: u32 = 0xE000_E018;

/* Traffic lights – road A (GPIO-A) */
const A_GREEN: u8 = 5;
const A_YELLOW: u8 = 6;
const A_RED: u8 = 7;

/* Traffic lights – road B (GPIO-B) */
const B_GREEN: u8 = 0;
const B_YELLOW: u8 = 1;
const B_RED: u8 = 2;

/* Bar-graph LEDs – road A (GPIO-A) */
const LVL1_A: u8 = 8;
const LVL2_A: u8 = 9;
const LVL3_A: u8 = 10;

/* Bar-graph LEDs – road B (GPIO-B) */
const LVL1_B: u8 = 6;
const LVL2_B: u8 = 7;
const LVL3_B: u8 = 8;

/* Push-buttons (active-LOW) */
const BTN_A_PIN: u8 = 13; // PC13 – road A
const BTN_B_PIN: u8 = 0; // PC0  – road B (real header pin)

/* --------------------------- Timing ------------------------------- */
const SYSCLK_HZ: u32 = 16_000_000; // CPU clock
const YELLOW_TIME_MS: u32 = 2_000;
const BASE_GREEN_TIME_MS: u32 = 10_000; // level 0 → 10 s
const DEBOUNCE_MS: u32 = 50;

/* ----------------------  Global state  ---------------------------- */
#[derive(Copy, Clone)]
struct ButtonState {
    is_pressed: bool,
    last_change_ms: u32,
}

struct TrafficState {
    level_a: u8, // 0-3
    level_b: u8, // 0-3
    btn_a: ButtonState,
    btn_b: ButtonState,
}

/* SAFETY: only accessed in `interrupt::free`-style poll loop */
static mut TRAFFIC: TrafficState = TrafficState {
    level_a: 3,
    level_b: 3,
    btn_a: ButtonState {
        is_pressed: false,
        last_change_ms: 0,
    },
    btn_b: ButtonState {
        is_pressed: false,
        last_change_ms: 0,
    },
};

/* 1 ms monotonic counter (wraps ≈ 49 days) */
static mut SYSTEM_MS: u32 = 0;

/* ================================================================== */
#[entry]
fn main() -> ! {
    init_gpio();
    systick_init(SYSCLK_HZ);

    unsafe { update_bargraphs() };

    loop {
        /* ------- A green / B red ------------------------------------ */
        set_pair_a(true, false, false);
        set_pair_b(false, false, true);
        delay_ms_poll(green_time_ms(unsafe { TRAFFIC.level_a }));

        /* ------- A yellow / B red ----------------------------------- */
        set_pair_a(false, true, false);
        set_pair_b(false, false, true);
        delay_ms_poll(YELLOW_TIME_MS);

        /* ------- A red / B green ------------------------------------ */
        set_pair_a(false, false, true);
        set_pair_b(true, false, false);
        delay_ms_poll(green_time_ms(unsafe { TRAFFIC.level_b }));

        /* ------- A red / B yellow ----------------------------------- */
        set_pair_a(false, false, true);
        set_pair_b(false, true, false);
        delay_ms_poll(YELLOW_TIME_MS);
    }
}

/* ---------------------- Hardware init ----------------------------- */
fn init_gpio() {
    /* enable GPIOA/B/C */
    unsafe {
        let enr = RCC_AHB1ENR as *mut u32;
        write_volatile(enr, read_volatile(enr) | (1 << 0) | (1 << 1) | (1 << 2));
    }

    /* configure all LED pins as outputs */
    for &(port, pin) in &[
        (GPIOA_BASE, A_GREEN),
        (GPIOA_BASE, A_YELLOW),
        (GPIOA_BASE, A_RED),
        (GPIOB_BASE, B_GREEN),
        (GPIOB_BASE, B_YELLOW),
        (GPIOB_BASE, B_RED),
        (GPIOA_BASE, LVL1_A),
        (GPIOA_BASE, LVL2_A),
        (GPIOA_BASE, LVL3_A),
        (GPIOB_BASE, LVL1_B),
        (GPIOB_BASE, LVL2_B),
        (GPIOB_BASE, LVL3_B),
    ] {
        unsafe { gpio_to_output(port, pin) }
    }

    /* configure PC13 & PC0 as inputs with pull-up */
    for &pin in &[BTN_A_PIN, BTN_B_PIN] {
        unsafe {
            let moder = (GPIOC_BASE + MODER) as *mut u32;
            let pupdr = (GPIOC_BASE + PUPDR) as *mut u32;

            let mut v = read_volatile(moder);
            v &= !(0b11 << (pin * 2)); // 00 = input
            write_volatile(moder, v);

            let mut v = read_volatile(pupdr);
            v &= !(0b11 << (pin * 2));
            v |= 0b01 << (pin * 2); // pull-up
            write_volatile(pupdr, v);
        }
    }
}

/* ------------------------- GPIO helpers --------------------------- */
unsafe fn gpio_to_output(port: u32, pin: u8) {
    let moder = (port + MODER) as *mut u32;
    let otyper = (port + OTYPER) as *mut u32;
    let ospeedr = (port + OSPEEDR) as *mut u32;

    let mut v = read_volatile(moder);
    v &= !(0b11 << (pin * 2));
    v |= 0b01 << (pin * 2); // output
    write_volatile(moder, v);

    let mut v = read_volatile(otyper);
    v &= !(1 << pin); // push-pull
    write_volatile(otyper, v);

    let mut v = read_volatile(ospeedr);
    v &= !(0b11 << (pin * 2));
    v |= 0b10 << (pin * 2); // high-speed
    write_volatile(ospeedr, v);
}

unsafe fn drive_led(port: u32, pin: u8, on: bool) {
    let bsrr = (port + BSRR) as *mut u32;
    if on {
        write_volatile(bsrr, 1 << pin);
    } else {
        write_volatile(bsrr, 1 << (pin + 16));
    }
}

/* ---------------- Traffic-light helpers --------------------------- */
fn set_pair_a(g: bool, y: bool, r: bool) {
    unsafe {
        drive_led(GPIOA_BASE, A_GREEN, g);
        drive_led(GPIOA_BASE, A_YELLOW, y);
        drive_led(GPIOA_BASE, A_RED, r);
    }
}
fn set_pair_b(g: bool, y: bool, r: bool) {
    unsafe {
        drive_led(GPIOB_BASE, B_GREEN, g);
        drive_led(GPIOB_BASE, B_YELLOW, y);
        drive_led(GPIOB_BASE, B_RED, r);
    }
}

/* ---------------- Bar-graph update -------------------------------- */
unsafe fn update_bargraphs() {
    drive_led(GPIOA_BASE, LVL1_A, TRAFFIC.level_a >= 1);
    drive_led(GPIOA_BASE, LVL2_A, TRAFFIC.level_a >= 2);
    drive_led(GPIOA_BASE, LVL3_A, TRAFFIC.level_a >= 3);

    drive_led(GPIOB_BASE, LVL1_B, TRAFFIC.level_b >= 1);
    drive_led(GPIOB_BASE, LVL2_B, TRAFFIC.level_b >= 2);
    drive_led(GPIOB_BASE, LVL3_B, TRAFFIC.level_b >= 3);
}

/* ---------------- Timing & wait-loop ------------------------------ */
fn systick_init(sysclk: u32) {
    unsafe {
        write_volatile(SYST_CSR as *mut u32, 0); // disable
        write_volatile(SYST_RVR as *mut u32, sysclk / 1000 - 1); // 1 ms
        write_volatile(SYST_CVR as *mut u32, 0);
        write_volatile(SYST_CSR as *mut u32, 1 | (1 << 2)); // enable
    }
}

fn delay_ms_poll(ms: u32) {
    for _ in 0..ms {
        while unsafe { read_volatile(SYST_CSR as *const u32) & (1 << 16) } == 0 {
            asm::nop();
        }
        unsafe { SYSTEM_MS = SYSTEM_MS.wrapping_add(1) };
        poll_buttons();
    }
}

/* ------------------ Button handling & debouncing ------------------ */
#[derive(PartialEq)]
enum ButtonEvent {
    NoChange,
    Pressed,
    Released,
}

/* debounce using a raw pointer to avoid `static_mut_refs` lint */
fn debounce(raw: bool, st: *mut ButtonState, now: u32) -> ButtonEvent {
    // SAFETY: caller guarantees exclusive access for this short call
    let s = unsafe { &mut *st };

    if now.wrapping_sub(s.last_change_ms) >= DEBOUNCE_MS {
        if raw != s.is_pressed {
            s.is_pressed = raw;
            s.last_change_ms = now;
            return if raw {
                ButtonEvent::Pressed
            } else {
                ButtonEvent::Released
            };
        }
    }
    ButtonEvent::NoChange
}

fn poll_buttons() {
    let idr = unsafe { read_volatile((GPIOC_BASE + IDR) as *const u32) };
    let raw_a = (idr & (1 << BTN_A_PIN)) == 0;
    let raw_b = (idr & (1 << BTN_B_PIN)) == 0;
    let now = unsafe { SYSTEM_MS };

    unsafe {
        let a_ptr = core::ptr::addr_of_mut!(TRAFFIC.btn_a);
        match debounce(raw_a, a_ptr, now) {
            ButtonEvent::Released => {
                TRAFFIC.level_a = (TRAFFIC.level_a + 3) % 4;
                update_bargraphs();
            }
            _ => {}
        }

        let b_ptr = core::ptr::addr_of_mut!(TRAFFIC.btn_b);
        match debounce(raw_b, b_ptr, now) {
            ButtonEvent::Released => {
                TRAFFIC.level_b = (TRAFFIC.level_b + 3) % 4;
                update_bargraphs();
            }
            _ => {}
        }
    }
}

/* ----------------------- Utility ---------------------------------- */
fn green_time_ms(level: u8) -> u32 {
    BASE_GREEN_TIME_MS * (level as u32 + 1) // 0→10 s … 3→40 s
}

/* ---------------------------- panic ------------------------------- */
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        asm::bkpt();
    }
}

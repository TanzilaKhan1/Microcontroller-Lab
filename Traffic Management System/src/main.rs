#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;

mod button;
mod gpio;
mod timer;
mod traffic;

use button::*;
use gpio::*;
use timer::*;
use traffic::*;

const RCC_AHB1ENR: u32 = 0x4002_3800 + 0x30;

const GPIOA_BASE: u32 = 0x4002_0000;
const GPIOB_BASE: u32 = 0x4002_0400;
const GPIOC_BASE: u32 = 0x4002_0800;

/* Pins for traffic lights & buttons (same as before) */
const A_GREEN: u8 = 0;
const A_YELLOW: u8 = 1;
const A_RED: u8 = 4;

const B_GREEN: u8 = 12;
const B_YELLOW: u8 = 2;
const B_RED: u8 = 1;

const LVL1_A: u8 = 5;
const LVL2_A: u8 = 6;
const LVL3_A: u8 = 7;

const LVL1_B: u8 = 13;
const LVL2_B: u8 = 14;
const LVL3_B: u8 = 15;

const BTN_A_PIN: u8 = 13; // PC13
const BTN_B_PIN: u8 = 0; // PC0

const SYSCLK_HZ: u32 = 16_000_000;
const YELLOW_TIME_MS: u32 = 2_000;
const BASE_GREEN_TIME_MS: u32 = 10_000;

struct TrafficState {
    level_a: u8,
    level_b: u8,
}

static mut TRAFFIC: TrafficState = TrafficState {
    level_a: 3,
    level_b: 3,
};

#[entry]
fn main() -> ! {
    unsafe {
        // Enable GPIO clocks for A, B, C
        enable_gpio_clocks(RCC_AHB1ENR, (1 << 0) | (1 << 1) | (1 << 2));
    }

    // Configure LEDs as outputs
    unsafe {
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
            pin_to_output(port, pin);
        }

        // Configure buttons as inputs with pull-up
        pin_to_input_pullup(GPIOC_BASE, BTN_A_PIN);
        pin_to_input_pullup(GPIOC_BASE, BTN_B_PIN);
    }

    // Initialize timer
    unsafe {
        systick_init(SYSCLK_HZ);
    }

    let mut buttons = Buttons::new(BTN_A_PIN, BTN_B_PIN, GPIOC_BASE);
    let traffic_lights = TrafficLights {
        green_a: A_GREEN,
        yellow_a: A_YELLOW,
        red_a: A_RED,
        green_b: B_GREEN,
        yellow_b: B_YELLOW,
        red_b: B_RED,
        lvl1_a: LVL1_A,
        lvl2_a: LVL2_A,
        lvl3_a: LVL3_A,
        lvl1_b: LVL1_B,
        lvl2_b: LVL2_B,
        lvl3_b: LVL3_B,
        port_a: GPIOA_BASE,
        port_b: GPIOB_BASE,
    };

    unsafe {
        traffic_lights.update_bargraphs(TRAFFIC.level_a, TRAFFIC.level_b);
    }

    loop {
        unsafe {
            traffic_lights.set_pair_a(true, false, false); // A green
            traffic_lights.set_pair_b(false, false, true); // B red
        }
        delay_ms_poll(green_time_ms(unsafe { TRAFFIC.level_a }), || {
            let now = get_system_ms();
            let (evt_a, evt_b) = buttons.poll(now);
            if evt_a == ButtonEvent::Released {
                unsafe {
                    TRAFFIC.level_a = (TRAFFIC.level_a + 3) % 4;
                    traffic_lights.update_bargraphs(TRAFFIC.level_a, TRAFFIC.level_b);
                }
            }
            if evt_b == ButtonEvent::Released {
                unsafe {
                    TRAFFIC.level_b = (TRAFFIC.level_b + 3) % 4;
                    traffic_lights.update_bargraphs(TRAFFIC.level_a, TRAFFIC.level_b);
                }
            }
        });

        unsafe {
            traffic_lights.set_pair_a(false, true, false); // A yellow
            traffic_lights.set_pair_b(false, false, true); // B red
        }
        delay_ms_poll(YELLOW_TIME_MS, || {});

        unsafe {
            traffic_lights.set_pair_a(false, false, true); // A red
            traffic_lights.set_pair_b(true, false, false); // B green
        }
        delay_ms_poll(green_time_ms(unsafe { TRAFFIC.level_b }), || {
            let now = get_system_ms();
            let (evt_a, evt_b) = buttons.poll(now);
            if evt_a == ButtonEvent::Released {
                unsafe {
                    TRAFFIC.level_a = (TRAFFIC.level_a + 3) % 4;
                    traffic_lights.update_bargraphs(TRAFFIC.level_a, TRAFFIC.level_b);
                }
            }
            if evt_b == ButtonEvent::Released {
                unsafe {
                    TRAFFIC.level_b = (TRAFFIC.level_b + 3) % 4;
                    traffic_lights.update_bargraphs(TRAFFIC.level_a, TRAFFIC.level_b);
                }
            }
        });

        unsafe {
            traffic_lights.set_pair_a(false, false, true); // A red
            traffic_lights.set_pair_b(false, true, false); // B yellow
        }
        delay_ms_poll(YELLOW_TIME_MS, || {});
    }
}

fn green_time_ms(level: u8) -> u32 {
    BASE_GREEN_TIME_MS * (level as u32 + 1)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

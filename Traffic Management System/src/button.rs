// button.rs
use core::ptr;

use crate::gpio;

const DEBOUNCE_MS: u32 = 50;

#[derive(Copy, Clone)]
pub struct ButtonState {
    pub is_pressed: bool,
    pub last_change_ms: u32,
}

#[derive(PartialEq, Debug)]
pub enum ButtonEvent {
    NoChange,
    Pressed,
    Released,
}

pub struct Buttons {
    pub btn_a: ButtonState,
    pub btn_b: ButtonState,
    pub pin_a: u8,
    pub pin_b: u8,
    pub port_base: u32,
}

impl Buttons {
    pub fn new(pin_a: u8, pin_b: u8, port_base: u32) -> Self {
        Buttons {
            btn_a: ButtonState {
                is_pressed: false,
                last_change_ms: 0,
            },
            btn_b: ButtonState {
                is_pressed: false,
                last_change_ms: 0,
            },
            pin_a,
            pin_b,
            port_base,
        }
    }

    fn debounce(&mut self, raw: bool, st: &mut ButtonState, now: u32) -> ButtonEvent {
        if now.wrapping_sub(st.last_change_ms) >= DEBOUNCE_MS {
            if raw != st.is_pressed {
                st.is_pressed = raw;
                st.last_change_ms = now;
                if raw {
                    return ButtonEvent::Pressed;
                } else {
                    return ButtonEvent::Released;
                }
            }
        }
        ButtonEvent::NoChange
    }

    pub fn poll(&mut self, now: u32) -> (ButtonEvent, ButtonEvent) {
        let idr = unsafe { ptr::read_volatile((self.port_base + gpio::IDR) as *const u32) };
        let raw_a = (idr & (1 << self.pin_a)) == 0;
        let raw_b = (idr & (1 << self.pin_b)) == 0;

        let mut event_a = ButtonEvent::NoChange;
        let mut event_b = ButtonEvent::NoChange;

        if now.wrapping_sub(self.btn_a.last_change_ms) >= DEBOUNCE_MS {
            if raw_a != self.btn_a.is_pressed {
                self.btn_a.is_pressed = raw_a;
                self.btn_a.last_change_ms = now;
                event_a = if raw_a {
                    ButtonEvent::Pressed
                } else {
                    ButtonEvent::Released
                };
            }
        }

        if now.wrapping_sub(self.btn_b.last_change_ms) >= DEBOUNCE_MS {
            if raw_b != self.btn_b.is_pressed {
                self.btn_b.is_pressed = raw_b;
                self.btn_b.last_change_ms = now;
                event_b = if raw_b {
                    ButtonEvent::Pressed
                } else {
                    ButtonEvent::Released
                };
            }
        }

        (event_a, event_b)
    }
}

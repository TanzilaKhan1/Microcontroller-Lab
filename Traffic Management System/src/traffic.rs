// traffic.rs or inside main.rs

use crate::gpio;

pub struct TrafficLights {
    // Pins for road A
    pub green_a: u8,
    pub yellow_a: u8,
    pub red_a: u8,

    // Pins for road B
    pub green_b: u8,
    pub yellow_b: u8,
    pub red_b: u8,

    // Bar graph LEDs A
    pub lvl1_a: u8,
    pub lvl2_a: u8,
    pub lvl3_a: u8,

    // Bar graph LEDs B
    pub lvl1_b: u8,
    pub lvl2_b: u8,
    pub lvl3_b: u8,

    pub port_a: u32,
    pub port_b: u32,
}

impl TrafficLights {
    pub unsafe fn set_pair_a(&self, g: bool, y: bool, r: bool) {
        gpio::drive_pin(self.port_a, self.green_a, g);
        gpio::drive_pin(self.port_a, self.yellow_a, y);
        gpio::drive_pin(self.port_a, self.red_a, r);
    }
    pub unsafe fn set_pair_b(&self, g: bool, y: bool, r: bool) {
        gpio::drive_pin(self.port_b, self.green_b, g);
        gpio::drive_pin(self.port_b, self.yellow_b, y);
        gpio::drive_pin(self.port_b, self.red_b, r);
    }
    pub unsafe fn update_bargraphs(&self, level_a: u8, level_b: u8) {
        gpio::drive_pin(self.port_a, self.lvl1_a, level_a >= 1);
        gpio::drive_pin(self.port_a, self.lvl2_a, level_a >= 2);
        gpio::drive_pin(self.port_a, self.lvl3_a, level_a >= 3);

        gpio::drive_pin(self.port_b, self.lvl1_b, level_b >= 1);
        gpio::drive_pin(self.port_b, self.lvl2_b, level_b >= 2);
        gpio::drive_pin(self.port_b, self.lvl3_b, level_b >= 3);
    }
}

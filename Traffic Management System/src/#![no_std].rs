// #![no_std]
// #![no_main]

// use core::panic::PanicInfo;
// use cortex_m::asm;
// use cortex_m_rt::entry;

// const RCC_BASE: u32 = 0x40023800;
// const RCC_AHB1ENR: u32 = RCC_BASE + 0x30;

// // GPIOA registers (D13/SCK is on PA5 for most STM32 boards)
// const GPIOA_BASE: u32 = 0x40020000;
// const GPIOA_MODER: u32 = GPIOA_BASE + 0x00;
// const GPIOA_OTYPER: u32 = GPIOA_BASE + 0x04;
// const GPIOA_OSPEEDR: u32 = GPIOA_BASE + 0x08;
// const GPIOA_PUPDR: u32 = GPIOA_BASE + 0x0C;
// const GPIOA_BSRR: u32 = GPIOA_BASE + 0x18;

// const SYST_CSR: u32 = 0xE000E010;
// const SYST_RVR: u32 = 0xE000E014;
// const SYST_CVR: u32 = 0xE000E018;

// const LED_PIN: u8 = 5;

// const SYSTEM_CLOCK_HZ: u32 = 16_000_000;

// #[entry]
// fn main() -> ! {
//     unsafe {
//         let rcc_ahb1enr = RCC_AHB1ENR as *mut u32;
//         *rcc_ahb1enr |= 1 << 0;
//     }

//     unsafe {
//         let gpioa_moder = GPIOA_MODER as *mut u32;
//         let mut moder_val = *gpioa_moder;
//         moder_val &= !(0b11 << (LED_PIN * 2));
//         moder_val |= 0b01 << (LED_PIN * 2);
//         *gpioa_moder = moder_val;

//         let gpioa_otyper = GPIOA_OTYPER as *mut u32;
//         *gpioa_otyper &= !(1 << LED_PIN);
//         let gpioa_ospeedr = GPIOA_OSPEEDR as *mut u32;
//         let mut ospeedr_val = *gpioa_ospeedr;
//         ospeedr_val &= !(0b11 << (LED_PIN * 2));
//         ospeedr_val |= 0b10 << (LED_PIN * 2);
//         *gpioa_ospeedr = ospeedr_val;

//         let gpioa_pupdr = GPIOA_PUPDR as *mut u32;
//         *gpioa_pupdr &= !(0b11 << (LED_PIN * 2));
//     }

//     unsafe {
//         let syst_csr = SYST_CSR as *mut u32;
//         *syst_csr = 0;

//         let syst_rvr = SYST_RVR as *mut u32;
//         *syst_rvr = SYSTEM_CLOCK_HZ / 1000 - 1;

//         let syst_cvr = SYST_CVR as *mut u32;
//         *syst_cvr = 0;

//         *syst_csr = 1 | (1 << 2);
//     }

//     loop {
//         unsafe {
//             let gpioa_bsrr = GPIOA_BSRR as *mut u32;
//             *gpioa_bsrr = 1 << LED_PIN;
//         }

//         delay_ms(3000);

//         unsafe {
//             let gpioa_bsrr = GPIOA_BSRR as *mut u32;
//             *gpioa_bsrr = 1 << (LED_PIN + 16); // Set PA5 low
//         }

//         delay_ms(3000);
//     }
// }

// fn delay_ms(ms: u32) {
//     for _ in 0..ms {
//         unsafe {
//             let syst_csr = SYST_CSR as *mut u32;
//             while (*syst_csr & (1 << 16)) == 0 {
//                 asm::nop();
//             }
//         }
//     }
// }

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {
//         asm::nop();
//     }
// }

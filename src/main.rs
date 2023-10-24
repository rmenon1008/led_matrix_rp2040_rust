#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use bsp::hal::pac;
use bsp::hal::prelude::_rphal_pio_PIOExt;
use bsp::hal::{clocks::StoppableClock, gpio::FunctionSpi};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::adc::OneShot;
use embedded_hal::spi::FullDuplex;
use fugit::RateExtU32;
use panic_probe as _;
use rp_pico as bsp;
mod rgb_matrix;

static mut CORE1_STACK: hal::multicore::Stack<4096> = hal::multicore::Stack::new();

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        12_000_000u32,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let led_frame = [128u8; 48 * 96 * 3];
    
    loop {
        
    }
}

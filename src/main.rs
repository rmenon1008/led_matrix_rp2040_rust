#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use bsp::hal::gpio::FunctionPio0;
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

    // Set up the RGB matrix
    let (mut pio, sm0, sm1, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut output_en_pin = pins.gpio13.into_push_pull_output();
    let _ = pins.gpio0.into_mode::<FunctionPio0>();
    let _ = pins.gpio1.into_mode::<FunctionPio0>();
    let _ = pins.gpio2.into_mode::<FunctionPio0>();
    let _ = pins.gpio3.into_mode::<FunctionPio0>();
    let _ = pins.gpio4.into_mode::<FunctionPio0>();
    let _ = pins.gpio5.into_mode::<FunctionPio0>();
    let _ = pins.gpio6.into_mode::<FunctionPio0>();
    let _ = pins.gpio7.into_mode::<FunctionPio0>();
    let _ = pins.gpio8.into_mode::<FunctionPio0>();
    let _ = pins.gpio9.into_mode::<FunctionPio0>();
    let _ = pins.gpio10.into_mode::<FunctionPio0>();
    let _ = pins.gpio11.into_mode::<FunctionPio0>();
    let _ = pins.gpio12.into_mode::<FunctionPio0>();

    let mut rgb_matrix = rgb_matrix::RgbMatrix::new(pio, sm0, sm1, output_en_pin);
    
    let led_frame = [128u8; 48 * 96 * 3];
    rgb_matrix.set_next_frame(&led_frame);
    
    loop {
        rgb_matrix.render();
    }
}

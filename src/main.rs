//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;

mod rgb_matrix;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 5_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let led_pins = rgb_matrix::RgbMatrix96x48Pins::new(pins);
    let rgb_r0 = pins.gpio0.into_push_pull_output();
    let rgb_g0 = pins.gpio1.into_push_pull_output();
    let rgb_b0 = pins.gpio2.into_push_pull_output();
    let rgb_r1 = pins.gpio3.into_push_pull_output();
    let rgb_g1 = pins.gpio4.into_push_pull_output();
    let rgb_b1 = pins.gpio5.into_push_pull_output();

    let addr_a = pins.gpio6.into_push_pull_output();
    let addr_b = pins.gpio7.into_push_pull_output();
    let addr_c = pins.gpio8.into_push_pull_output();
    let addr_d = pins.gpio9.into_push_pull_output();
    let addr_e = pins.gpio14.into_push_pull_output();

    let clock = pins.gpio10.into_push_pull_output();
    let latch = pins.gpio12.into_push_pull_output();
    let output_enable = pins.gpio13.into_push_pull_output();

    let rgb_pins = rgb_matrix::RgbPins::new(rgb_r0, rgb_g0, rgb_b0, rgb_r1, rgb_g1, rgb_b1);
    let addr_pins = rgb_matrix::AddrPins::new(addr_a, addr_b, addr_c, addr_d, addr_e);
    let latch = rgb_matrix::LatchPin::new(latch);
    let clock = rgb_matrix::ClockPin::new(clock);
    let output_enable = rgb_matrix::OutputEnablePin::new(output_enable);

    let mut matrix =
        rgb_matrix::RgbMatrix96x48::new(rgb_pins, addr_pins, latch, clock, output_enable);

    let mut color_frame = [0; 96 * 48 * 3];
    let mut counter = 0;

    loop {
        for i in 0..(96 * 48) {
            if (i + i / 96) % 2 == 0 {
                color_frame[i * 3] = counter;
                color_frame[i * 3 + 1] = counter;
                color_frame[i * 3 + 2] = counter;
            } else {
                color_frame[i * 3] = 255 - counter;
                color_frame[i * 3 + 1] = 255 - counter;
                color_frame[i * 3 + 2] = 255 - counter;
            }
        }
        matrix.set_next_frame(&color_frame);
        matrix.render();

        counter = counter % 255;
        // counter += 1;
    }
}

// End of file

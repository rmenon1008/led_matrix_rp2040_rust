#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use bsp::hal::pac;
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
static mut LED_FRAME: [u8; 96 * 48 * 3] = [0u8; 96 * 48 * 3];
static mut CURRENT_POSITION: usize = 0;
static BRIGHTNESS_EXP_ALPHA: f32 = 0.995;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let mut sio = hal::sio::Sio::new(pac.SIO);

    // Step 1. Set up clocks. We're doing this manually here, because we're overclocking it.
    // This is to reduce the flicker on the matrix.

    // Set up the system clock to use the external oscillator, running at 12 MHz.
    let xosc = hal::xosc::setup_xosc_blocking(pac.XOSC, bsp::XOSC_CRYSTAL_FREQ.Hz())
        .map_err(|_x| false)
        .unwrap();

    // Set up the watchdog to generate a tick every 1 us.
    watchdog.enable_tick_generation((bsp::XOSC_CRYSTAL_FREQ / 1_000_000) as u8);

    // Set up the clock manager.
    let mut clocks = hal::clocks::ClocksManager::new(pac.CLOCKS);
    let pll_sys = hal::pll::setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency(),
        hal::pll::PLLConfig {
            vco_freq: 1512.MHz(),
            refdiv: 1,
            post_div1: 5,
            post_div2: 1,
        },
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(|_x| false)
    .unwrap();

    // Set up the USB clock.
    let pll_usb = hal::pll::setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        hal::pll::common_configs::PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(|_x| false)
    .unwrap();

    // Initialize the clocks.
    clocks
        .init_default(&xosc, &pll_sys, &pll_usb)
        .map_err(|_x| false)
        .unwrap();

    clocks.gpio_output0_clock.disable();
    clocks.gpio_output1_clock.disable();
    clocks.gpio_output2_clock.disable();
    clocks.gpio_output3_clock.disable();
    clocks.rtc_clock.disable();

    // Set up the peripherals.
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the RGB matrix.
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
    let addr_e = pins.gpio10.into_push_pull_output();

    let clock = pins.gpio11.into_push_pull_output();
    let latch = pins.gpio12.into_push_pull_output();
    let output_enable = pins.gpio13.into_push_pull_output();

    let rgb_pins = rgb_matrix::RgbPins::new(rgb_r0, rgb_g0, rgb_b0, rgb_r1, rgb_g1, rgb_b1);
    let addr_pins = rgb_matrix::AddrPins::new(addr_a, addr_b, addr_c, addr_d, addr_e);
    let latch = rgb_matrix::LatchPin::new(latch);
    let clock = rgb_matrix::ClockPin::new(clock);
    let output_enable = rgb_matrix::OutputEnablePin::new(output_enable);

    let mut matrix =
        rgb_matrix::RgbMatrix96x48::new(rgb_pins, addr_pins, latch, clock, output_enable);

    // Set up the second core to read the SPI data and write it to the buffer.
    let mut mc = hal::multicore::Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            let mut pac = unsafe { pac::Peripherals::steal() };

            // Set up the SPI driver
            let _spi_mosi = pins.gpio19.into_mode::<FunctionSpi>();
            let _spi_miso = pins.gpio16.into_mode::<FunctionSpi>();
            let _spi_sck = pins.gpio18.into_mode::<FunctionSpi>();
            let spi = hal::spi::Spi::<_, _, 8>::new(pac.SPI0);

            let mut spi = spi.init_slave(&mut pac.RESETS, &embedded_hal::spi::MODE_3);

            loop {
                if let Ok(value) = spi.read() {
                    unsafe {
                        LED_FRAME[CURRENT_POSITION] = value;
                        CURRENT_POSITION += 1;
                        if CURRENT_POSITION >= 96 * 48 * 3 {
                            CURRENT_POSITION = 0;
                        }
                    }
                }
            }
        })
        .unwrap();

    // Set up the ADC and the brightness sensor.
    let mut adc = hal::Adc::new(pac.ADC, &mut pac.RESETS);
    let mut brightness_sensor_adc = pins.gpio28.into_floating_input();

    // Keep track of the brightness with an exponential moving average
    let mut brightness: f32 = 1600.0;
    loop {
        unsafe {
            matrix.set_next_frame(&LED_FRAME);
        }

        // // Read the brightness sensor and store the value in the array
        let brightness_sensor_value: u16 = adc.read(&mut brightness_sensor_adc).unwrap();

        // // Update the brightness
        brightness = BRIGHTNESS_EXP_ALPHA * brightness
            + (1.0 - BRIGHTNESS_EXP_ALPHA) * brightness_sensor_value as f32;

        // Render the matrix
        matrix.render(brightness_n(brightness as u16));
    }
}

fn brightness_n(adc: u16) -> u8 {
    if adc < 50 {
        1
    } else if adc < 100 {
        2
    } else if adc < 200 {
        3
    } else if adc < 400 {
        4
    } else if adc < 700 {
        5
    } else if adc < 1200 {
        6
    } else {
        7
    }
}

// End of file

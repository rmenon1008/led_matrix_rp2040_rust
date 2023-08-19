//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::hal::multicore::{Multicore, Stack};
use bsp::hal::pac::interrupt;
use bsp::{entry, hal::clocks::StoppableClock};
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use fugit::RateExtU32;
use panic_probe as _;
use rp_pico as bsp;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

static mut CORE1_STACK: Stack<4096> = Stack::new();
static mut USB_DEVICE: Option<UsbDevice<bsp::hal::usb::UsbBus>> = None;
static mut USB_BUS: Option<UsbBusAllocator<bsp::hal::usb::UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<bsp::hal::usb::UsbBus>> = None;

mod rgb_matrix;

use bsp::hal::{clocks::Clock, pac, sio::Sio, watchdog::Watchdog};

static mut LED_FRAME: [u8; 96 * 48 * 3] = [0u8; 96 * 48 * 3];
static mut CURRENT_POSITION: usize = 0;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    // Step 1. Set up clocks. We're doing this manually here, because we're overclocking it.
    // This is to reduce the flicker on the matrix.

    // Set up the system clock to use the external oscillator, running at 12 MHz.
    let xosc = bsp::hal::xosc::setup_xosc_blocking(pac.XOSC, bsp::XOSC_CRYSTAL_FREQ.Hz())
        .map_err(|_x| false)
        .unwrap();

    // Set up the watchdog to generate a tick every 1 us.
    watchdog.enable_tick_generation((rp_pico::XOSC_CRYSTAL_FREQ / 1_000_000) as u8);

    // Set up the clock manager.
    let mut clocks = bsp::hal::clocks::ClocksManager::new(pac.CLOCKS);
    let pll_sys = bsp::hal::pll::setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency(),
        bsp::hal::pll::PLLConfig {
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
    let pll_usb = bsp::hal::pll::setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency(),
        bsp::hal::pll::common_configs::PLL_USB_48MHZ,
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
    clocks.adc_clock.disable();
    clocks.rtc_clock.disable();

    // Step 2. Set up the peripherals.
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

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

    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            // Set up the USB driver
            let usb_bus = UsbBusAllocator::new(bsp::hal::usb::UsbBus::new(
                pac.USBCTRL_REGS,
                pac.USBCTRL_DPRAM,
                clocks.usb_clock,
                true,
                &mut pac.RESETS,
            ));
            unsafe {
                USB_BUS = Some(usb_bus);
            }
            let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

            // Set up the USB Communications Class Device driver
            let serial = SerialPort::new(bus_ref);
            unsafe {
                USB_SERIAL = Some(serial);
            }

            let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Rohan Menon")
                .product("Serial port")
                .serial_number("_led_matrix_")
                .device_class(USB_CLASS_CDC) // from: https://www.usb.org/defined-class-codes
                .build();
            unsafe {
                USB_DEVICE = Some(usb_dev);
            }

            unsafe {
                pac::NVIC::unmask(bsp::hal::pac::Interrupt::USBCTRL_IRQ);
            };

            loop {}
        })
        .unwrap();

    loop {
        unsafe {
            matrix.set_next_frame(&LED_FRAME);
        }
        matrix.render();
    }
}

#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // use core::sync::atomic::{AtomicBool, Ordering};

    // Grab the global objects. This is OK as we only access them under interrupt.
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let serial = USB_SERIAL.as_mut().unwrap();

    // Poll the USB driver with all of our supported USB Classes
    if usb_dev.poll(&mut [serial]) {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Err(_e) => {}
            Ok(0) => {}
            Ok(count) => {
                for i in 0..count {
                    LED_FRAME[CURRENT_POSITION] = buf[i];
                    CURRENT_POSITION += 1;
                }

                if CURRENT_POSITION >= 96 * 48 * 3 {
                    CURRENT_POSITION = 0;
                }
            }
        }
    }
}

// End of file

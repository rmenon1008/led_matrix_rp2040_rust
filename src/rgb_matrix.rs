// Path: src/rgb_matrix.rs
use cortex_m::asm;
use embedded_hal::digital::v2::OutputPin;

const WIDTH: usize = 96;
const HEIGHT: usize = 48;

#[derive(Debug)]
pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

pub struct RgbPins<
    R0: OutputPin,
    G0: OutputPin,
    B0: OutputPin,
    R1: OutputPin,
    G1: OutputPin,
    B1: OutputPin,
> {
    pub r0: R0,
    pub g0: G0,
    pub b0: B0,
    pub r1: R1,
    pub g1: G1,
    pub b1: B1,
}

impl<R0: OutputPin, G0: OutputPin, B0: OutputPin, R1: OutputPin, G1: OutputPin, B1: OutputPin>
    RgbPins<R0, G0, B0, R1, G1, B1>
{
    pub fn new(r0: R0, g0: G0, b0: B0, r1: R1, g1: G1, b1: B1) -> RgbPins<R0, G0, B0, R1, G1, B1> {
        RgbPins {
            r0,
            g0,
            b0,
            r1,
            g1,
            b1,
        }
    }

    pub fn set_rgb_bits(&mut self, data: u8) -> Result<()> {
        if data > 0b0011_1111 {
            return Err(Error);
        }

        let r0 = data & 0b0000_0001 != 0;
        let g0 = data & 0b0000_0010 != 0;
        let b0 = data & 0b0000_0100 != 0;
        let r1 = data & 0b0000_1000 != 0;
        let g1 = data & 0b0001_0000 != 0;
        let b1 = data & 0b0010_0000 != 0;

        if r0 {
            self.r0.set_high().map_err(|_| Error)?;
        } else {
            self.r0.set_low().map_err(|_| Error)?;
        }

        if g0 {
            self.g0.set_high().map_err(|_| Error)?;
        } else {
            self.g0.set_low().map_err(|_| Error)?;
        }

        if b0 {
            self.b0.set_high().map_err(|_| Error)?;
        } else {
            self.b0.set_low().map_err(|_| Error)?;
        }

        if r1 {
            self.r1.set_high().map_err(|_| Error)?;
        } else {
            self.r1.set_low().map_err(|_| Error)?;
        }

        if g1 {
            self.g1.set_high().map_err(|_| Error)?;
        } else {
            self.g1.set_low().map_err(|_| Error)?;
        }

        if b1 {
            self.b1.set_high().map_err(|_| Error)?;
        } else {
            self.b1.set_low().map_err(|_| Error)?;
        }

        Ok(())
    }
}

pub struct AddrPins<A: OutputPin, B: OutputPin, C: OutputPin, D: OutputPin, E: OutputPin> {
    pub a: A,
    pub b: B,
    pub c: C,
    pub d: D,
    pub e: E,
}

impl<A: OutputPin, B: OutputPin, C: OutputPin, D: OutputPin, E: OutputPin> AddrPins<A, B, C, D, E> {
    pub fn new(a: A, b: B, c: C, d: D, e: E) -> AddrPins<A, B, C, D, E> {
        AddrPins { a, b, c, d, e }
    }

    pub fn set_addr_bits(&mut self, data: u8) -> Result<()> {
        if data > 0b0001_1111 {
            return Err(Error);
        }

        let a = data & 0b0000_0001 != 0;
        let b = data & 0b0000_0010 != 0;
        let c = data & 0b0000_0100 != 0;
        let d = data & 0b0000_1000 != 0;
        let e = data & 0b0001_0000 != 0;

        if a {
            self.a.set_high().map_err(|_| Error)?;
        } else {
            self.a.set_low().map_err(|_| Error)?;
        }

        if b {
            self.b.set_high().map_err(|_| Error)?;
        } else {
            self.b.set_low().map_err(|_| Error)?;
        }

        if c {
            self.c.set_high().map_err(|_| Error)?;
        } else {
            self.c.set_low().map_err(|_| Error)?;
        }

        if d {
            self.d.set_high().map_err(|_| Error)?;
        } else {
            self.d.set_low().map_err(|_| Error)?;
        }

        if e {
            self.e.set_high().map_err(|_| Error)?;
        } else {
            self.e.set_low().map_err(|_| Error)?;
        }

        Ok(())
    }
}

pub struct LatchPin<L: OutputPin> {
    pub latch: L,
}

impl<L: OutputPin> LatchPin<L> {
    pub fn new(latch: L) -> LatchPin<L> {
        LatchPin { latch }
    }

    pub fn set_latch(&mut self, data: bool) -> Result<()> {
        if data {
            self.latch.set_high().map_err(|_| Error)?;
        } else {
            self.latch.set_low().map_err(|_| Error)?;
        }

        Ok(())
    }
}

pub struct ClockPin<C: OutputPin> {
    pub clock: C,
}

impl<C: OutputPin> ClockPin<C> {
    pub fn new(clock: C) -> ClockPin<C> {
        ClockPin { clock }
    }

    pub fn set_clock(&mut self, data: bool) -> Result<()> {
        if data {
            self.clock.set_high().map_err(|_| Error)?;
        } else {
            self.clock.set_low().map_err(|_| Error)?;
        }

        Ok(())
    }
}

pub struct OutputEnablePin<O: OutputPin> {
    pub output_enable: O,
}

impl<O: OutputPin> OutputEnablePin<O> {
    pub fn new(output_enable: O) -> OutputEnablePin<O> {
        OutputEnablePin { output_enable }
    }

    pub fn set_output_enable(&mut self, data: bool) -> Result<()> {
        if data {
            self.output_enable.set_high().map_err(|_| Error)?;
        } else {
            self.output_enable.set_low().map_err(|_| Error)?;
        }

        Ok(())
    }
}

pub struct RgbMatrix96x48<
    R0: OutputPin,
    G0: OutputPin,
    B0: OutputPin,
    R1: OutputPin,
    G1: OutputPin,
    B1: OutputPin,
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    L: OutputPin,
    Clk: OutputPin,
    Oe: OutputPin,
> {
    rgb_pins: RgbPins<R0, G0, B0, R1, G1, B1>,
    addr_pins: AddrPins<A, B, C, D, E>,
    latch_pin: LatchPin<L>,
    clock_pin: ClockPin<Clk>,
    output_enable_pin: OutputEnablePin<Oe>,
    current_frame: [u8; 96 * 48 * 3],
    next_frame: [u8; 96 * 48 * 3],
    swap_frames: bool,
}

impl<
        R0: OutputPin,
        G0: OutputPin,
        B0: OutputPin,
        R1: OutputPin,
        G1: OutputPin,
        B1: OutputPin,
        A: OutputPin,
        B: OutputPin,
        C: OutputPin,
        D: OutputPin,
        E: OutputPin,
        L: OutputPin,
        Clk: OutputPin,
        Oe: OutputPin,
    > RgbMatrix96x48<R0, G0, B0, R1, G1, B1, A, B, C, D, E, L, Clk, Oe>
{
    pub fn new(
        rgb_pins: RgbPins<R0, G0, B0, R1, G1, B1>,
        addr_pins: AddrPins<A, B, C, D, E>,
        latch_pin: LatchPin<L>,
        clock_pin: ClockPin<Clk>,
        output_enable_pin: OutputEnablePin<Oe>,
    ) -> RgbMatrix96x48<R0, G0, B0, R1, G1, B1, A, B, C, D, E, L, Clk, Oe> {
        RgbMatrix96x48 {
            rgb_pins,
            addr_pins,
            latch_pin,
            clock_pin,
            output_enable_pin,
            current_frame: [0; 96 * 48 * 3],
            next_frame: [0; 96 * 48 * 3],
            swap_frames: false,
        }
    }

    pub fn set_next_frame(&mut self, data: &[u8]) {
        self.next_frame.copy_from_slice(data);
        self.swap_frames = true;
    }

    fn get_data_bits(&mut self, row: usize, col: usize, depth_level: u8) -> u8 {
        let mut data = 0;

        let r0 = (self.current_frame[3 * (row * WIDTH + col) + 0] >> depth_level) & 0x01;
        let g0 = (self.current_frame[3 * (row * WIDTH + col) + 1] >> depth_level) & 0x01;
        let b0 = (self.current_frame[3 * (row * WIDTH + col) + 2] >> depth_level) & 0x01;
        let r1 =
            (self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 0] >> depth_level) & 0x01;
        let g1 =
            (self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 1] >> depth_level) & 0x01;
        let b1 =
            (self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 2] >> depth_level) & 0x01;

        data |= r0 << 0;
        data |= g0 << 1;
        data |= b0 << 2;
        data |= r1 << 3;
        data |= g1 << 4;
        data |= b1 << 5;

        data
    }

    pub fn render(&mut self) {
        if self.swap_frames {
            self.current_frame.copy_from_slice(&self.next_frame);
            self.swap_frames = false;
        }

        for row in 0..HEIGHT / 2 {
            for depth in 0..8 {
                for col in 0..WIDTH {
                    // let r0 = self.current_frame[3 * (row * WIDTH + col) + 0] > 0;
                    // let g0 = self.current_frame[3 * (row * WIDTH + col) + 1] > 0;
                    // let b0 = self.current_frame[3 * (row * WIDTH + col) + 2] > 0;
                    // let r1 = self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 0] > 0;
                    // let g1 = self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 1] > 0;
                    // let b1 = self.current_frame[3 * ((row + HEIGHT / 2) * WIDTH + col) + 2] > 0;

                    // // Append these into a single byte
                    // let mut data = 0u8;
                    // data |= (r0 as u8) << 0;
                    // data |= (g0 as u8) << 1;
                    // data |= (b0 as u8) << 2;
                    // data |= (r1 as u8) << 3;
                    // data |= (g1 as u8) << 4;
                    // data |= (b1 as u8) << 5;

                    //
                    let data = self.get_data_bits(row, col, depth);

                    // Set the data
                    self.rgb_pins.set_rgb_bits(data).unwrap();

                    // Pulse the clock
                    self.clock_pin.set_clock(true).unwrap();
                    asm::delay(1);
                    self.clock_pin.set_clock(false).unwrap();
                }
                // // Disable the output
                // self.output_enable_pin.set_output_enable(true).unwrap();

                // Set the address
                self.addr_pins.set_addr_bits((row) as u8).unwrap();

                // Pulse the latch
                self.latch_pin.set_latch(true).unwrap();
                asm::delay(1);
                self.latch_pin.set_latch(false).unwrap();

                // Enable the output
                self.output_enable_pin.set_output_enable(false).unwrap();
                asm::delay(2u32.pow(depth as u32) * 100);
                self.output_enable_pin.set_output_enable(true).unwrap();
            }
        }
    }
}

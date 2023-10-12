// Path: src/rgb_matrix.rs
use cortex_m::asm;
use embedded_hal::digital::v2::OutputPin;

const WIDTH: usize = 96;
const HEIGHT: usize = 48;
const HALF_HEIGHT: usize = HEIGHT / 2;
const DELAY_TABLE_8: [u32; 11] = [6, 12, 24, 48, 96, 192, 384, 768, 1536, 3072, 6144];
const DELAY_TABLE_7: [u32; 11] = [5, 10, 20, 40, 80, 160, 320, 640, 1280, 2560, 5120];
const DELAY_TABLE_6: [u32; 11] = [4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
const DELAY_TABLE_5: [u32; 11] = [3, 6, 12, 24, 48, 96, 192, 384, 768, 1536, 3072];
const DELAY_TABLE_4: [u32; 11] = [2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048];
const DELAY_TABLE_3: [u32; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const DELAY_TABLE_2: [u32; 11] = [0, 0, 1, 1, 2, 4, 8, 16, 32, 64, 128];
const DELAY_TABLE_1: [u32; 11] = [
    6144, 6144, 6144, 6144, 6144, 6144, 6144, 6144, 6144, 6144, 6144,
];
// const DELAY_TABLE: [u32; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const GAMMA_RED_TABLE: [u16; 256] = [
    // 2.9 gamma
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5,
    5, 5, 6, 6, 7, 8, 8, 9, 10, 10, 11, 12, 13, 13, 14, 15, 16, 17, 18, 19, 20, 22, 23, 24, 25, 27,
    28, 29, 31, 32, 34, 36, 37, 39, 41, 42, 44, 46, 48, 50, 52, 54, 57, 59, 61, 64, 66, 68, 71, 74,
    76, 79, 82, 85, 88, 91, 94, 97, 100, 103, 106, 110, 113, 117, 120, 124, 128, 132, 136, 140,
    144, 148, 152, 156, 161, 165, 169, 174, 179, 183, 188, 193, 198, 203, 208, 214, 219, 225, 230,
    236, 241, 247, 253, 259, 265, 271, 277, 284, 290, 297, 303, 310, 317, 324, 331, 338, 345, 352,
    360, 367, 375, 382, 390, 398, 406, 414, 423, 431, 439, 448, 457, 465, 474, 483, 492, 501, 511,
    520, 530, 539, 549, 559, 569, 579, 589, 600, 610, 621, 632, 642, 653, 664, 676, 687, 698, 710,
    722, 734, 745, 758, 770, 782, 795, 807, 820, 833, 846, 859, 872, 885, 899, 913, 926, 940, 954,
    969, 983, 997, 1012, 1027, 1042, 1057, 1072, 1087, 1102, 1118, 1134, 1150, 1166, 1182, 1198,
    1215, 1231, 1248, 1265, 1282, 1299, 1317, 1334, 1352, 1370, 1388, 1406, 1424, 1442, 1461, 1480,
    1499, 1518, 1537, 1556, 1576, 1595, 1615, 1635, 1655, 1676, 1696, 1717, 1738, 1759, 1780, 1801,
    1823, 1844, 1866, 1888, 1910, 1933, 1955, 1978, 2001, 2024, 2047,
];
const GAMMA_GREEN_TABLE: [u16; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 4, 5, 6, 6, 7, 7, 8, 9, 10,
    11, 11, 12, 13, 14, 15, 16, 18, 19, 20, 21, 23, 24, 25, 27, 28, 30, 31, 33, 35, 37, 38, 40, 42,
    44, 46, 48, 51, 53, 55, 57, 60, 62, 65, 67, 70, 72, 75, 78, 81, 84, 87, 90, 93, 96, 99, 103,
    106, 109, 113, 116, 120, 124, 127, 131, 135, 139, 143, 147, 151, 156, 160, 164, 169, 173, 178,
    183, 187, 192, 197, 202, 207, 212, 217, 223, 228, 233, 239, 245, 250, 256, 262, 268, 274, 280,
    286, 292, 298, 305, 311, 317, 324, 331, 338, 344, 351, 358, 365, 373, 380, 387, 395, 402, 410,
    417, 425, 433, 441, 449, 457, 465, 474, 482, 491, 499, 508, 516, 525, 534, 543, 552, 562, 571,
    580, 590, 599, 609, 619, 628, 638, 648, 658, 669, 679, 689, 700, 710, 721, 732, 743, 754, 765,
    776, 787, 799, 810, 822, 833, 845, 857, 869, 881, 893, 905, 918, 930, 943, 955, 968, 981, 994,
    1007, 1020, 1033, 1047, 1060, 1074, 1088, 1101, 1115, 1129, 1143, 1157, 1172, 1186, 1201, 1215,
    1230, 1245, 1260, 1275, 1290, 1305, 1321, 1336, 1352, 1367, 1383, 1399, 1415, 1431, 1448, 1464,
    1480, 1497, 1514, 1530, 1547, 1564, 1582, 1599, 1616, 1634, 1651, 1669, 1687, 1705, 1723, 1741,
    1759, 1778, 1796, 1815, 1833, 1852, 1871, 1890, 1909, 1929, 1948, 1968, 1987, 2007, 2027, 2047,
];
const GAMMA_BLUE_TABLE: [u16; 256] = [
    // 2.8 gamma
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5, 5, 6,
    6, 7, 7, 8, 9, 9, 10, 11, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25, 27, 28, 29,
    31, 32, 34, 36, 37, 39, 41, 43, 45, 47, 49, 51, 53, 55, 57, 59, 62, 64, 67, 69, 72, 74, 77, 80,
    83, 85, 88, 91, 94, 98, 101, 104, 107, 111, 114, 118, 121, 125, 129, 133, 137, 141, 145, 149,
    153, 157, 162, 166, 171, 175, 180, 185, 189, 194, 199, 204, 210, 215, 220, 226, 231, 237, 242,
    248, 254, 260, 266, 272, 278, 284, 291, 297, 304, 310, 317, 324, 331, 338, 345, 352, 359, 367,
    374, 382, 390, 397, 405, 413, 421, 430, 438, 446, 455, 463, 472, 481, 490, 499, 508, 517, 526,
    536, 545, 555, 565, 575, 585, 595, 605, 615, 626, 636, 647, 658, 669, 680, 691, 702, 713, 725,
    736, 748, 760, 772, 784, 796, 808, 821, 833, 846, 859, 872, 885, 898, 911, 925, 938, 952, 966,
    980, 994, 1008, 1022, 1037, 1051, 1066, 1081, 1096, 1111, 1126, 1142, 1157, 1173, 1189, 1204,
    1221, 1237, 1253, 1270, 1286, 1303, 1320, 1337, 1354, 1371, 1389, 1406, 1424, 1442, 1460, 1478,
    1496, 1515, 1533, 1552, 1571, 1590, 1609, 1629, 1648, 1668, 1687, 1707, 1727, 1748, 1768, 1789,
    1809, 1830, 1851, 1872, 1894, 1915, 1937, 1958, 1980, 2002, 2025, 2047,
];

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

fn brightness_adjust(frame: &mut [u8], brightness: u8) {
    if brightness > 2 {
        return;
    }
    for i in 0..frame.len() {
        frame[i] = (frame[i] as u16 * brightness as u16 / 8) as u8;
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
        let base_index = 3 * (row * WIDTH + col);
        let base_index_bottom = 3 * ((row + HALF_HEIGHT) * WIDTH + col);

        let r0 = ((GAMMA_RED_TABLE[self.current_frame[base_index] as usize] >> depth_level) & 0x01)
            as u8;
        let g0 = ((GAMMA_GREEN_TABLE[self.current_frame[base_index + 1] as usize] >> depth_level)
            & 0x01) as u8;
        let b0 = ((GAMMA_BLUE_TABLE[self.current_frame[base_index + 2] as usize] >> depth_level)
            & 0x01) as u8;
        let r1 = ((GAMMA_RED_TABLE[self.current_frame[base_index_bottom] as usize] >> depth_level)
            & 0x01) as u8;
        let g1 = ((GAMMA_GREEN_TABLE[self.current_frame[base_index_bottom + 1] as usize]
            >> depth_level)
            & 0x01) as u8;
        let b1 = ((GAMMA_BLUE_TABLE[self.current_frame[base_index_bottom + 2] as usize]
            >> depth_level)
            & 0x01) as u8;

        r0 << 0 | g0 << 1 | b0 << 2 | r1 << 3 | g1 << 4 | b1 << 5
    }

    pub fn render(&mut self, brightness: u8) {
        if self.swap_frames {
            self.current_frame.copy_from_slice(&self.next_frame);

            brightness_adjust(&mut self.current_frame, brightness);
            self.swap_frames = false;
        }
        for depth in 0..11 {
            for row in 0..HEIGHT / 2 {
                for col in 0..WIDTH {
                    // Get the data from the current frame
                    let data = self.get_data_bits(row, col, depth);

                    // Set the data
                    self.rgb_pins.set_rgb_bits(data).unwrap();

                    // Pulse the clock
                    self.clock_pin.set_clock(false).unwrap();
                    asm::nop();
                    self.clock_pin.set_clock(true).unwrap();
                }

                // Set the address
                self.addr_pins.set_addr_bits((row) as u8).unwrap();

                // Pulse the latch
                self.latch_pin.set_latch(true).unwrap();
                asm::nop();
                self.latch_pin.set_latch(false).unwrap();

                // Enable the output
                self.output_enable_pin.set_output_enable(false).unwrap();

                let delay_table;
                match brightness {
                    0 => delay_table = DELAY_TABLE_1,
                    1 => delay_table = DELAY_TABLE_2,
                    2 => delay_table = DELAY_TABLE_3,
                    3 => delay_table = DELAY_TABLE_4,
                    4 => delay_table = DELAY_TABLE_5,
                    5 => delay_table = DELAY_TABLE_6,
                    6 => delay_table = DELAY_TABLE_7,
                    7 => delay_table = DELAY_TABLE_8,
                    _ => delay_table = DELAY_TABLE_8,
                }

                asm::delay(delay_table[depth as usize]);
                self.output_enable_pin.set_output_enable(true).unwrap();
            }
        }
    }
}

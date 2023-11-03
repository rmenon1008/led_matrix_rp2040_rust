use cortex_m::asm;
use embedded_hal::digital::v2::OutputPin;
// Path: src/rgb_matrix.rs
use rp_pico as bsp;
use bsp::hal::prelude::_rphal_pio_PIOExt;
use bsp::hal::gpio::{FunctionPio0, Pin, Output};
use crate::hal::gpio::DynPin;
use bsp::hal::pac;
use bsp::hal::pio;
use bsp::hal::pio::{PIO, SM0, SM1, UninitStateMachine};
use pio_proc::pio_asm;

pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

const WIDTH: u32 = 48;
const HEIGHT: u32 = 96;
const HALF_HEIGHT: u32 = HEIGHT / 2;

const RGB_PIN_START: u8 = 0;
const CLK_PIN : u8 = 11;
const ADDR_PIN_START: u8 = 6;
const LATCH_PIN: u8 = 12;

const DELAY_TABLE: [u32; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const GAMMA_TABLE: [u16; 256] = [ // 2.8 gamma
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

pub struct RgbMatrix<OE: OutputPin> {
    this_frame: [u8; 48 * 96 * 3],
    next_frame: [u8; 48 * 96 * 3],
    swap_frames: bool,
    addr_tx: pio::Tx<(pac::PIO0, SM0)>,
    rgb_tx: pio::Tx<(pac::PIO0, SM1)>,
    output_en_pin: OE,
}

impl <OE: OutputPin> RgbMatrix <OE> {
    pub fn new(
        mut pio0: PIO<pac::PIO0>,
        mut sm0: UninitStateMachine<(pac::PIO0, SM0)>,
        mut sm1: UninitStateMachine<(pac::PIO0, SM1)>,
        mut output_en_pin: OE,
    ) -> Self {
        // Address program
        // Set address pins, pulse latch
        // Called once per row
        //
        // Each FIFO record consists of:
        //  - address data (5 LSBs - A B C D E)
        let address_program = pio_asm!(
            ".side_set 1",                      // One side set pin, Latch (0)
            ".wrap_target",
                "pull               side 0x0",  // Pull 32 bits from FIFO
                "out pins, 5 [7]    side 0x1",  // Set address data to pins, set Latch
            ".wrap"
        );

        let address_program_installed = pio0.install(&address_program.program).unwrap();
        let (mut sm_addr, _, addr_tx) = pio::PIOBuilder::from_program(address_program_installed)
            .out_pins(ADDR_PIN_START, 5)
            .side_set_pin_base(LATCH_PIN)
            .clock_divisor_fixed_point(1, 0)
            .build(sm0);
        sm_addr.set_pindirs([
            (ADDR_PIN_START + 0, pio::PinDir::Output),
            (ADDR_PIN_START + 1, pio::PinDir::Output),
            (ADDR_PIN_START + 2, pio::PinDir::Output),
            (ADDR_PIN_START + 3, pio::PinDir::Output),
            (ADDR_PIN_START + 4, pio::PinDir::Output),
            (LATCH_PIN, pio::PinDir::Output)
        ]);
        sm_addr.start();

        // RGB program
        // Set pixel data, pulse clock
        // Called once per set of 2 pixels (top and bottom)
        //
        // Each FIFO record consists of:
        //  - pixel data (6 LSBs - R0 G0 B0 R1 G1 B1)
        let rgb_program = pio_asm!(
            ".side_set 1",                      // One side set pin, Clk (0)
            ".wrap_target",
                "pull               side 0x0",  // Pull 32 bits from FIFO
                "out pins, 6 [7]    side 0x1",  // Set RGB data to pins, set Clk
            ".wrap"
        );

        let rgb_program_installed = pio0.install(&rgb_program.program).unwrap();
        let (mut sm_rgb, _, rgb_tx) = pio::PIOBuilder::from_program(rgb_program_installed)
            .out_pins(RGB_PIN_START, 6)
            .side_set_pin_base(CLK_PIN)
            .clock_divisor_fixed_point(1, 0)
            .build(sm1);
        sm_rgb.set_pindirs([
            (RGB_PIN_START + 0, pio::PinDir::Output),
            (RGB_PIN_START + 1, pio::PinDir::Output),
            (RGB_PIN_START + 2, pio::PinDir::Output),
            (RGB_PIN_START + 3, pio::PinDir::Output),
            (RGB_PIN_START + 4, pio::PinDir::Output),
            (RGB_PIN_START + 5, pio::PinDir::Output),
            (CLK_PIN, pio::PinDir::Output)
        ]);
        sm_rgb.start();

        Self {
            this_frame: [128; 48 * 96 * 3],
            next_frame: [0; 48 * 96 * 3],
            swap_frames: false,
            addr_tx,
            rgb_tx,
            output_en_pin: output_en_pin,
        }
    }

    pub fn get_data_bits(&mut self, row: u32, col: u32, depth_level: u32) -> u8 {
        let base_index = 3 * (row * WIDTH + col) as usize;
        let base_index_bottom = 3 * ((row + HALF_HEIGHT) * WIDTH + col) as usize;

        let r0 = ((GAMMA_TABLE[self.this_frame[base_index] as usize] >> depth_level) & 0x01)
            as u8;
        let g0 = ((GAMMA_TABLE[self.this_frame[base_index + 1] as usize] >> depth_level)
            & 0x01) as u8;
        let b0 = ((GAMMA_TABLE[self.this_frame[base_index + 2] as usize] >> depth_level)
            & 0x01) as u8;
        let r1 = ((GAMMA_TABLE[self.this_frame[base_index_bottom] as usize] >> depth_level)
            & 0x01) as u8;
        let g1 = ((GAMMA_TABLE[self.this_frame[base_index_bottom + 1] as usize]
            >> depth_level)
            & 0x01) as u8;
        let b1 = ((GAMMA_TABLE[self.this_frame[base_index_bottom + 2] as usize]
            >> depth_level)
            & 0x01) as u8;

        r0 << 0 | g0 << 1 | b0 << 2 | r1 << 3 | g1 << 4 | b1 << 5
    }

    pub fn set_next_frame(&mut self, frame: &[u8]) {
        self.next_frame.copy_from_slice(frame);
        self.swap_frames = true;
    }

    pub fn render(&mut self) -> Result<()> {
        if self.swap_frames {
            self.this_frame.copy_from_slice(&self.next_frame);
            self.swap_frames = false;
        }

        for depth in 0..11 {
            for row in 0..HALF_HEIGHT {
                for col in 0..WIDTH {
                    let data = self.get_data_bits(col, row, depth);
                    self.rgb_tx.write(data as u32);
                }
                self.addr_tx.write(row);

                self.output_en_pin.set_low().map_err(|_| Error)?;
                asm::delay(DELAY_TABLE[depth as usize]);
                self.output_en_pin.set_high().map_err(|_| Error)?;
            }
        }
        Ok(())
    }
}

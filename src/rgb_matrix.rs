// Path: src/rgb_matrix.rs
use rp_pico as bsp;
use bsp::hal::prelude::_rphal_pio_PIOExt;
use bsp::hal::gpio::{FunctionPio0, Pin};
use bsp::hal::pac;
use bsp::hal::pio::{PIO, SM0, UninitStateMachine};
use pio_proc::pio_asm;

const WIDTH: u32 = 48;
const HEIGHT: u32 = 96;
const HALF_HEIGHT: u32 = HEIGHT / 2;

pub struct RgbMatrix {
    this_frame: [u8; 48 * 96 * 3],
    next_frame: [u8; 48 * 96 * 3],
    swap_frames: bool,
}

impl RgbMatrix {
    pub fn new(pio0: PIO<pac::PIO0>, sm0: UninitStateMachine<(pac::PIO0, SM0)>) -> Self {
        // Row program
        // Repeatedly select a row, pulse LATCH, and generate
        // a pulse of a certain width on OEn.
        //
        // Each FIFO record consists of:
        //  - row select addr (5 LSBs)
        //  - pulse width duration (27 MSBs)
        
        let row_program = pio_asm!(
            ".side_set 2",                      // Two side set pins, L (0) and OEn (1)
            ".wrap_target",
                "out pins, 5 [7]    side 0x2",  // Set row address to pins,
                                                // turn off L, disable OEn (high)
                "out x, 27   [7]    side 0x3",  // Get pulse width, pulse L, keep OEn high
            "pulse_loop:"
                "jmp x-- pulse_loop side 0x0",  // Loop for pulse width (x + 1)
            ".wrap"
        );

        let rgb_program: = pio_asm!(
            ".side_set 1",                      // One side set pin, Clk (0)
            ".wrap_target",
                "out pins, 8 [7]    side 0x1",  // Set RGB data to pins, pulse Clk
        )

        
    }
}
        
    
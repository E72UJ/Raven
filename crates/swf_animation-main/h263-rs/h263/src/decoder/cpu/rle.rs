//! Block run decompression

use crate::{types::Block, types::DecodedDctBlock};

// These are (x, y) coords, not (row, col).
const DEZIGZAG_MAPPING: [(u8, u8); 64] = [
    (0, 0),
    (1, 0),
    (0, 1),
    (0, 2),
    (1, 1),
    (2, 0),
    (3, 0),
    (2, 1),
    (1, 2),
    (0, 3),
    (0, 4),
    (1, 3),
    (2, 2),
    (3, 1),
    (4, 0),
    (5, 0),
    (4, 1),
    (3, 2),
    (2, 3),
    (1, 4),
    (0, 5),
    (0, 6),
    (1, 5),
    (2, 4),
    (3, 3),
    (4, 2),
    (5, 1),
    (6, 0),
    (7, 0),
    (6, 1),
    (5, 2),
    (4, 3),
    (3, 4),
    (2, 5),
    (1, 6),
    (0, 7),
    (1, 7),
    (2, 6),
    (3, 5),
    (4, 4),
    (5, 3),
    (6, 2),
    (7, 1),
    (7, 2),
    (6, 3),
    (5, 4),
    (4, 5),
    (3, 6),
    (2, 7),
    (3, 7),
    (4, 6),
    (5, 5),
    (6, 4),
    (7, 3),
    (7, 4),
    (6, 5),
    (5, 6),
    (4, 7),
    (5, 7),
    (6, 6),
    (7, 5),
    (7, 6),
    (6, 7),
    (7, 7),
];

/// Inverse RLE, dezigzag, and dequantize encoded block coefficient data.
///
/// `encoded_block` should be the block data as returned from `decode_block`.
/// `levels` will be filled with a row-major (x + y*8) decompressed list of
/// coefficients at the position `pos` (assuming a stride of
/// `samples_per_line`.)
///
/// This function assumes `levels` has already been initialized to zero. If the
/// levels array is reused, you must reinitialize it again.
pub fn inverse_rle(
    encoded_block: &Block,
    levels: &mut [DecodedDctBlock],
    pos: (usize, usize),
    blk_per_line: usize,
    quant: u8,
) {
    let block_id = pos.0 / 8 + (pos.1 / 8 * blk_per_line);
    let block = &mut levels[block_id];

    // Taking care of some special cases of outputs (IDCT inputs) first,
    // where at most the DC coefficient is present.
    *block = if encoded_block.tcoef.is_empty() {
        match encoded_block.intradc {
            Some(dc) => {
                // The block is DC only.
                let dc_level = dc.into_level();

                if dc_level == 0 {
                    // This isn't really supposed to happen, but just in case...
                    // (If the DC coefficient is zero, it shouldn't have been coded.)
                    DecodedDctBlock::Zero
                } else {
                    DecodedDctBlock::Dc(dc_level.into())
                }
            }
            None => DecodedDctBlock::Zero, // The block is empty.
        }
    } else {
        // The slightly less special cases: `Horiz`, `Vert`, and `Full`.
        let mut block_data = [[0.0f32; 8]; 8];

        let mut is_horiz = true;
        let mut is_vert = true;

        let mut zigzag_index = 0;
        if let Some(dc) = encoded_block.intradc {
            block_data[0][0] = dc.into_level().into();
            zigzag_index += 1;
        }
        for tcoef in encoded_block.tcoef.iter() {
            zigzag_index += tcoef.run as usize;

            if zigzag_index >= DEZIGZAG_MAPPING.len() {
                return;
            }

            let (zig_x, zig_y) = DEZIGZAG_MAPPING[zigzag_index];
            let dequantized_level = quant as i16 * ((2 * tcoef.level.abs()) + 1);
            let parity = if quant % 2 == 1 { 0 } else { -1 };

            let value = (tcoef.level.signum() * (dequantized_level + parity)).clamp(-2048, 2047);
            let val = value.into();
            block_data[zig_y as usize][zig_x as usize] = val;
            zigzag_index += 1;

            if val != 0.0 {
                if zig_y > 0 {
                    is_horiz = false;
                }
                if zig_x > 0 {
                    is_vert = false;
                }
            }
        }

        match (is_horiz, is_vert) {
            (true, true) => {
                // This shouldn't really happen, but just in case...
                // (If the block is DC, it shouldn't have had TCOEF runs.)
                if block_data[0][0] == 0.0 {
                    DecodedDctBlock::Zero
                } else {
                    DecodedDctBlock::Dc(block_data[0][0])
                }
            }
            (true, false) => DecodedDctBlock::Horiz(block_data[0]),
            (false, true) => DecodedDctBlock::Vert([
                block_data[0][0],
                block_data[1][0],
                block_data[2][0],
                block_data[3][0],
                block_data[4][0],
                block_data[5][0],
                block_data[6][0],
                block_data[7][0],
            ]),
            (false, false) => DecodedDctBlock::Full(block_data),
        }
    }
}

//! Intra block data collection

use crate::decoder::picture::DecodedPicture;
use crate::error::Error;
use crate::types::{MacroblockType, MotionVector};

/// Read a sample from the pixel array at a given position.
///
/// Sample coordinates in `pos` will be clipped to the bounds of the pixel
/// data. This is in accordance with H.263 (2005/01) D.1, which states that
/// motion vectors that cross picture boundaries instead clip the last row,
/// column, or individual pixel off the edge of the picture. (This is
/// equivalent to, say OpenGL `GL_CLAMP_TO_EDGE` behavior.)
///
/// Pixel array data is read as a row-major (x + y*width) array.
fn read_sample(
    pixel_array: &[u8],
    samples_per_row: usize,
    num_rows: usize,
    pos: (isize, isize),
) -> u8 {
    let (x, y) = pos;

    let x = x.clamp(0, samples_per_row.saturating_sub(1) as isize) as usize;
    let y = y.clamp(0, num_rows.saturating_sub(1) as isize) as usize;

    pixel_array
        .get(x + (y * samples_per_row))
        .copied()
        .expect("pixel array index out of bounds")
}

/// Linear interpolation between two values by 0 or 50%.
fn lerp(sample_a: u8, sample_b: u8, middle: bool) -> u8 {
    if middle {
        ((sample_a as u16 + sample_b as u16 + 1) / 2) as u8
    } else {
        sample_a
    }
}

/// Copy pixel data from a pixel array, motion-compensate it, and fill a block
/// with the given data.
///
/// Target block and source pixel array are written to in row-major (x + y*8)
/// order.
fn gather_block(
    pixel_array: &[u8],
    samples_per_row: usize,
    pos: (usize, usize),
    mv: MotionVector,
    target: &mut [u8],
) {
    let ((x_delta, x_interp), (y_delta, y_interp)) = mv.into_lerp_parameters();

    let src_x = pos.0 as isize + x_delta as isize;
    let src_y = pos.1 as isize + y_delta as isize;
    let array_height = pixel_array.len() / samples_per_row;

    let block_cols = (samples_per_row as isize - pos.0 as isize).clamp(0, 8);
    let block_rows = (array_height as isize - pos.1 as isize).clamp(0, 8);

    if !x_interp && !y_interp {
        // Fast path: No interpolation needed in either direction.

        if block_cols == 8
            && block_rows == 8
            && (0..=samples_per_row as isize - 8).contains(&src_x)
            && (0..=array_height as isize - 8).contains(&src_y)
        {
            // Fast path: Both the destination and source are full 8x8 blocks entirely within the frame,
            // so no need for coordinate clamping, and we can copy pixels in (horizontal) groups of 8.

            for j in 0..8 {
                let src_offset = src_x as usize + ((src_y + j as isize) as usize * samples_per_row);
                let dest_offset = pos.0 + (pos.1 + j) * samples_per_row;
                target[dest_offset..dest_offset + 8]
                    .copy_from_slice(&pixel_array[src_offset..src_offset + 8]);
            }
        } else {
            // Generic path: Copy pixels one at a time, with coordinate clamping.

            for (j, v) in (src_y..src_y + block_rows).enumerate() {
                for (i, u) in (src_x..src_x + block_cols).enumerate() {
                    target[pos.0 + i + ((pos.1 + j) * samples_per_row)] =
                        read_sample(pixel_array, samples_per_row, array_height, (u, v));
                }
            }
        }
    } else {
        // Generic path: Interpolate in at least one direction.

        for (j, v) in (src_y..src_y + block_rows).enumerate() {
            for (i, u) in (src_x..src_x + block_cols).enumerate() {
                let sample_0_0 = read_sample(pixel_array, samples_per_row, array_height, (u, v));
                let sample_1_0 =
                    read_sample(pixel_array, samples_per_row, array_height, (u + 1, v));
                let sample_0_1 =
                    read_sample(pixel_array, samples_per_row, array_height, (u, v + 1));
                let sample_1_1 =
                    read_sample(pixel_array, samples_per_row, array_height, (u + 1, v + 1));

                if x_interp && y_interp {
                    // Special case: Only round once at the end when interpolating in both directions.

                    let sample = ((sample_0_0 as u16
                        + sample_1_0 as u16
                        + sample_0_1 as u16
                        + sample_1_1 as u16
                        + 2) // for proper rounding
                        / 4) as u8;

                    target[pos.0 + i + ((pos.1 + j) * samples_per_row)] = sample;
                } else {
                    // Interpolating in exactly one of the directions.

                    let sample_mid_0 = lerp(sample_0_0, sample_1_0, x_interp);
                    let sample_mid_1 = lerp(sample_0_1, sample_1_1, x_interp);

                    target[pos.0 + i + ((pos.1 + j) * samples_per_row)] =
                        lerp(sample_mid_0, sample_mid_1, y_interp);
                }
            }
        }
    }
}

/// Copy pixels from a previously decoded reference picture into a new picture.
///
/// This function works on the entire picture's macroblocks as a batch. You
/// will need to provide a list of macroblock types, each macroblock's motion
/// vectors,
///
/// For `INTER` coded macroblocks, the gather process performs motion
/// compensation using the reference picture to produce the block data to be
/// mixed with the result of the IDCT.
///
/// For `INTRA` coded macroblocks, the returned set of blocks will be all
/// zeroes.
pub fn gather(
    mb_types: &[MacroblockType],
    reference_picture: Option<&DecodedPicture>,
    mvs: &[[MotionVector; 4]],
    mb_per_line: usize,
    new_picture: &mut DecodedPicture,
) -> Result<(), Error> {
    for (i, (mb_type, mv)) in mb_types.iter().zip(mvs.iter()).enumerate() {
        if mb_type.is_inter() {
            let reference_picture = reference_picture.ok_or(Error::UncodedIFrameBlocks)?;
            let luma_samples_per_row = reference_picture.luma_samples_per_row();
            let pos = ((i % mb_per_line) * 16, (i / mb_per_line) * 16);

            gather_block(
                reference_picture.as_luma(),
                luma_samples_per_row,
                pos,
                mv[0],
                new_picture.as_luma_mut(),
            );
            gather_block(
                reference_picture.as_luma(),
                luma_samples_per_row,
                (pos.0 + 8, pos.1),
                mv[1],
                new_picture.as_luma_mut(),
            );
            gather_block(
                reference_picture.as_luma(),
                luma_samples_per_row,
                (pos.0, pos.1 + 8),
                mv[2],
                new_picture.as_luma_mut(),
            );
            gather_block(
                reference_picture.as_luma(),
                luma_samples_per_row,
                (pos.0 + 8, pos.1 + 8),
                mv[3],
                new_picture.as_luma_mut(),
            );

            let mv_chr = (mv[0] + mv[1] + mv[2] + mv[3]).average_sum_of_mvs();
            let chroma_samples_per_row = reference_picture.chroma_samples_per_row();
            let chroma_pos = ((i % mb_per_line) * 8, (i / mb_per_line) * 8);

            gather_block(
                reference_picture.as_chroma_b(),
                chroma_samples_per_row,
                (chroma_pos.0, chroma_pos.1),
                mv_chr,
                new_picture.as_chroma_b_mut(),
            );
            gather_block(
                reference_picture.as_chroma_r(),
                chroma_samples_per_row,
                (chroma_pos.0, chroma_pos.1),
                mv_chr,
                new_picture.as_chroma_r_mut(),
            );
        }
    }

    Ok(())
}

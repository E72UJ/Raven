//! YUV-to-RGB decode

// TODO: Replace with `std::simd` when it's stable
use wide::{i32x4, u8x16};

// Operates on 4 pixels at a time, one pixel per SIMD lane,
// with 32 bits of intermediate per-component precision for
// each, so as to fill the 128-bit SIMD registers on WASM.
// And i32x4 also allows the neat transpose trick at the end.
// The output is an interleaved array of 4 RGBA pixels.
#[inline]
fn yuv_to_rgba_4x(yuv: (&[u8; 4], &[u8; 2], &[u8; 2]), rgba: &mut [u8; 16]) {
    let (y, cb, cr) = yuv;

    // Expanding the 4 bytes into a i32x4, and duplicating chroma samples horizontally.
    // The -16 and -128 are simply undoing the offsets in the input representation.
    let y = i32x4::from([y[0] as i32, y[1] as i32, y[2] as i32, y[3] as i32]) - i32x4::splat(16);
    let cb =
        i32x4::from([cb[0] as i32, cb[0] as i32, cb[1] as i32, cb[1] as i32]) - i32x4::splat(128);
    let cr =
        i32x4::from([cr[0] as i32, cr[0] as i32, cr[1] as i32, cr[1] as i32]) - i32x4::splat(128);

    // The rest of the magic numbers are the coefficients converted to 16.16 fixed point, and rounded.
    // They also include the extension from reduced (16..235 and 16...240) to full-range (0..255).
    let gray = y * i32x4::splat(76309); // 76309 == round((255.0 / 219.0) * 65536.0)
    let cr2r = cr * i32x4::splat(104597); // 104597 == round((255.0 / 224.0) * 1.402 * 65536.0)
    let cr2g = cr * i32x4::splat(-53279); // -53279 == round(-(255.0 / 224.0) * 1.402 * (0.299 / 0.587) * 65536.0)
    let cb2g = cb * i32x4::splat(-25675); // -25675 == round(-(255.0 / 224.0) * 1.772 * (0.114 / 0.587) * 65536.0)
    let cb2b = cb * i32x4::splat(132201); // 132201 == round((255.0 / 224.0) * 1.772 * 65536.0)

    // This is 0.5 in 16.16 format, added to make the rightshift round correctly
    let half = i32x4::splat(32768);

    // We could skip the shift here, then simply cast the result into [u8; 16], and take
    // bytes 2, 4, 10, 14 instead (after clamping), but it's not any faster, it seems.
    let r: i32x4 = (gray + cr2r + half) >> 16;
    let g: i32x4 = (gray + cr2g + cb2g + half) >> 16;
    let b: i32x4 = (gray + cb2b + half) >> 16;

    // Clamping to the valid output range
    // A simple clamp(x, 0, 255) doesn't work, because it seems to
    // operate on entire tuples, instead of each element separately.
    let max = i32x4::splat(255);

    let r = r.max(i32x4::ZERO).min(max);
    let g = g.max(i32x4::ZERO).min(max);
    let b = b.max(i32x4::ZERO).min(max);

    // The output alpha values are fixed
    let a = i32x4::splat(255);
    // Transposing the separate RGBA components into a single interleaved vector
    // Thanks for the tip, Lokathor!
    #[cfg(target_endian = "little")]
    let rgba_4x = ((r) | (g << 8)) | ((b << 16) | (a << 24));
    #[cfg(target_endian = "big")] // I haven't tested this, but should work
    let rgba_4x = ((r << 24) | (g << 16)) | ((b << 8) | (a));

    rgba.copy_from_slice(bytemuck::cast::<i32x4, u8x16>(rgba_4x).as_array_ref())
}

// A single-pixel version, only for testing.
#[cfg(test)]
#[inline]
fn yuv_to_rgb(yuv: (u8, u8, u8)) -> (u8, u8, u8) {
    let mut rgba_4x = [0u8; 16];
    yuv_to_rgba_4x(
        (
            &[yuv.0, yuv.0, yuv.0, yuv.0],
            &[yuv.1, yuv.1],
            &[yuv.2, yuv.2],
        ),
        &mut rgba_4x,
    );

    // all output pixels should be the same
    assert!(rgba_4x[3] == 255);
    assert!(rgba_4x[4] == rgba_4x[0]);
    assert!(rgba_4x[5] == rgba_4x[1]);
    assert!(rgba_4x[6] == rgba_4x[2]);
    assert!(rgba_4x[7] == 255);
    assert!(rgba_4x[8] == rgba_4x[0]);
    assert!(rgba_4x[9] == rgba_4x[1]);
    assert!(rgba_4x[10] == rgba_4x[2]);
    assert!(rgba_4x[11] == 255);
    assert!(rgba_4x[12] == rgba_4x[0]);
    assert!(rgba_4x[13] == rgba_4x[1]);
    assert!(rgba_4x[14] == rgba_4x[2]);
    assert!(rgba_4x[15] == 255);

    (rgba_4x[0], rgba_4x[1], rgba_4x[2])
}

/// Convert planar YUV 4:2:0 data into interleaved RGBA 8888 data.
///
/// This function yields an RGBA picture with the same number of pixels as were
/// provided in the `y` picture. The `chroma_b` and `chroma_r` samples are
/// simply reused without any interpolation for all four corresponding pixels.
/// This is not the most correct, or nicest, but it's what Flash Player does.
///
/// Preconditions:
///  - `y.len()` must be an integer multiple of `y_width`
///  - `chroma_b` and `chroma_r` must be the same size
///  - With `y_height` computed as `y.len() / y_width`, `br_width` as half of `y_width` (rounded up),
///    and `br_height` as `chroma_b.len() / br_width`: `br_height` must be half of `y_height`, rounded up
pub fn yuv420_to_rgba(y: &[u8], chroma_b: &[u8], chroma_r: &[u8], y_width: usize) -> Vec<u8> {
    // Shortcut for the no-op case to avoid all kinds of overflows below
    if y.is_empty() {
        debug_assert_eq!(chroma_b.len(), 0);
        debug_assert_eq!(chroma_r.len(), 0);
        debug_assert_eq!(y_width, 0);
        return vec![];
    }

    // the + 1 is for rounding odd numbers up
    let br_width = (y_width + 1) / 2;

    debug_assert_eq!(y.len() % y_width, 0);
    debug_assert_eq!(chroma_b.len() % br_width, 0);
    debug_assert_eq!(chroma_r.len() % br_width, 0);
    debug_assert_eq!(chroma_b.len(), chroma_r.len());

    let y_height = y.len() / y_width;
    let br_height = chroma_b.len() / br_width;

    // the + 1 is for rounding odd numbers up
    debug_assert_eq!((y_height + 1) / 2, br_height);

    let mut rgba = vec![0; y.len() * 4];
    let rgba_stride = y_width * 4; // 4 bytes per pixel, interleaved

    // Iteration is done in a row-major order to fit the slice layouts.
    for luma_rowindex in 0..y_height {
        let chroma_rowindex = luma_rowindex / 2;

        let y_remainder = y_width % 4;
        let br_remainder = br_width % 2;
        let rgba_remainder = y_remainder * 4;

        // This block is here just so the mutable borrow of rgba_row expires sooner.
        {
            // These borrows only include whole chunks of lengths 4 and 2.
            let y_row = &y[luma_rowindex * y_width..(luma_rowindex + 1) * y_width - y_remainder];
            let cb_row = &chroma_b
                [chroma_rowindex * br_width..(chroma_rowindex + 1) * br_width - br_remainder];
            let cr_row = &chroma_r
                [chroma_rowindex * br_width..(chroma_rowindex + 1) * br_width - br_remainder];
            let rgba_row = &mut rgba
                [luma_rowindex * rgba_stride..(luma_rowindex + 1) * rgba_stride - rgba_remainder];

            // TODO: Replace `bytemuck::cast_slice` with `std::slice::array_chunks` when it's stable.

            // Iterating on 4 pixels (in a horizontal row arrangement) at a time,
            // leaving off the last few on the right if width is not divisible by 4.
            let y_iter = bytemuck::cast_slice::<u8, [u8; 4]>(y_row).iter();
            // We need half as many chroma samples for each iteration
            let cb_iter = bytemuck::cast_slice::<u8, [u8; 2]>(cb_row).iter();
            let cr_iter = bytemuck::cast_slice::<u8, [u8; 2]>(cr_row).iter();
            // Similar to how Y is iterated on, but with 4 channels per pixel
            let rgba_iter = bytemuck::cast_slice_mut::<u8, [u8; 16]>(rgba_row).iter_mut();

            for (((y, cb), cr), rgba) in y_iter.zip(cb_iter).zip(cr_iter).zip(rgba_iter) {
                yuv_to_rgba_4x((y, cb, cr), rgba);
            }
        }

        // On pictures with width not divisible by 4, the last few pixels are not
        // covered by the iteration above, so doing them here, at once in each row.
        if y_remainder != 0 {
            // These are the same borrows as above, but with the whole row, not rounded down to multiples of 4 or 2.
            let y_row = &y[luma_rowindex * y_width..(luma_rowindex + 1) * y_width];
            let cb_row = &chroma_b[chroma_rowindex * br_width..(chroma_rowindex + 1) * br_width];
            let cr_row = &chroma_r[chroma_rowindex * br_width..(chroma_rowindex + 1) * br_width];
            let rgba_row =
                &mut rgba[luma_rowindex * rgba_stride..(luma_rowindex + 1) * rgba_stride];

            let mut y = [0u8; 4];
            let mut cb = [0u8; 2];
            let mut cr = [0u8; 2];

            for x in y_width - y_remainder..y_width {
                y[x % 4] = y_row[x];
                cb[(x % 4) / 2] = cb_row[x / 2];
                cr[(x % 4) / 2] = cr_row[x / 2];
            }

            let mut rgba_4x = [0u8; 16];
            yuv_to_rgba_4x((&y, &cb, &cr), &mut rgba_4x);

            for i in rgba_stride - rgba_remainder..rgba_stride {
                rgba_row[i] = rgba_4x[i % 16];
            }
        }
    }

    rgba
}

#[test]
fn test_yuv_to_rgb() {
    // From the H.263 Rec.:
    // Black = 16
    // White = 235
    // Zero colour difference = 128
    // Peak colour difference = 16 and 240

    // not quite black
    assert_eq!(yuv_to_rgb((17, 128, 128)), (1, 1, 1));
    // exactly black
    assert_eq!(yuv_to_rgb((16, 128, 128)), (0, 0, 0));
    // and clamping also works
    assert_eq!(yuv_to_rgb((15, 128, 128)), (0, 0, 0));
    assert_eq!(yuv_to_rgb((0, 128, 128)), (0, 0, 0));

    // not quite white
    assert_eq!(yuv_to_rgb((234, 128, 128)), (254, 254, 254));
    // exactly white
    assert_eq!(yuv_to_rgb((235, 128, 128)), (255, 255, 255));
    // and clamping also works
    assert_eq!(yuv_to_rgb((236, 128, 128)), (255, 255, 255));
    assert_eq!(yuv_to_rgb((255, 128, 128)), (255, 255, 255));

    // (16 + 235) / 2 = 125.5, for middle grays
    assert_eq!(yuv_to_rgb((125, 128, 128)), (127, 127, 127));
    assert_eq!(yuv_to_rgb((126, 128, 128)), (128, 128, 128));
}

// Inverse conversion, for testing purposes only
#[cfg(test)]
fn rgb_to_yuv(rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    let (red, green, blue) = rgb;
    let (red, green, blue) = (red as f32, green as f32, blue as f32);

    // From the same Wikipedia article as LUTs::new()
    let y = 16.0 + (65.481 * red) / 255.0 + (128.553 * green) / 255.0 + (24.966 * blue) / 255.0;
    let u = 128.0 - (37.797 * red) / 255.0 - (74.203 * green) / 255.0 + (112.0 * blue) / 255.0;
    let v = 128.0 + (112.0 * red) / 255.0 - (93.786 * green) / 255.0 - (18.214 * blue) / 255.0;

    (y.round() as u8, u.round() as u8, v.round() as u8)
}

// The function used for testing should also have its own tests :)
#[test]
fn test_rgb_to_yuv() {
    // black is Y=16
    assert_eq!(rgb_to_yuv((0, 0, 0)), (16, 128, 128));
    assert_eq!(rgb_to_yuv((1, 1, 1)), (17, 128, 128));

    // white is Y=235
    assert_eq!(rgb_to_yuv((254, 254, 254)), (234, 128, 128));
    assert_eq!(rgb_to_yuv((255, 255, 255)), (235, 128, 128));

    assert_eq!(
        rgb_to_yuv((255, 0, 0)),
        (81, 90, 240) // 240 is the full color difference
    );
    assert_eq!(rgb_to_yuv((0, 255, 0)), (145, 54, 34));
    assert_eq!(
        rgb_to_yuv((0, 0, 255)),
        (41, 240, 110) // 240 is the full color difference
    );

    assert_eq!(
        rgb_to_yuv((0, 255, 255)),
        (170, 166, 16) // 16 is the full color difference
    );
    assert_eq!(rgb_to_yuv((255, 0, 255)), (106, 202, 222));
    assert_eq!(
        rgb_to_yuv((255, 255, 0)),
        (210, 16, 146) // 16 is the full color difference
    );
}

#[test]
fn test_rgb_yuv_rgb_roundtrip_sanity() {
    assert_eq!(yuv_to_rgb(rgb_to_yuv((0, 0, 0))), (0, 0, 0));
    assert_eq!(yuv_to_rgb(rgb_to_yuv((127, 127, 127))), (127, 127, 127));
    assert_eq!(yuv_to_rgb(rgb_to_yuv((128, 128, 128))), (128, 128, 128));
    assert_eq!(yuv_to_rgb(rgb_to_yuv((255, 255, 255))), (255, 255, 255));

    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((255, 0, 0))),
        (254, 0, 0) // !!! there is a rounding error here
    );
    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((0, 255, 0))),
        (0, 255, 1) // !!! there is a rounding error here
    );
    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((0, 0, 255))),
        (0, 0, 255) // there is NO rounding error here
    );

    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((0, 255, 255))),
        (1, 255, 255) // !!! there is a rounding error here
    );
    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((255, 0, 255))),
        (255, 0, 254) // !!! there is a rounding error here
    );
    assert_eq!(
        yuv_to_rgb(rgb_to_yuv((255, 255, 0))),
        (255, 255, 0) // there is NO rounding error here
    );

    // the "tab10" palette:
    for rgb in [
        (31, 119, 180),
        (255, 127, 14),
        (44, 160, 44),
        (219, 39, 40),
        (148, 103, 189),
        (140, 86, 75),
        (227, 119, 194),
        (127, 127, 127),
        (188, 189, 34),
        (23, 190, 207),
    ] {
        let rgb2 = yuv_to_rgb(rgb_to_yuv(rgb));
        // Allowing for a difference of at most 1 on each component in both directions,
        // to account for the limited precision in YUV form, and two roundings
        assert!((rgb.0 as i32 - rgb2.0 as i32).abs() <= 1);
        assert!((rgb.1 as i32 - rgb2.1 as i32).abs() <= 1);
        assert!((rgb.2 as i32 - rgb2.2 as i32).abs() <= 1);
    }
}

#[test]
fn test_yuv420_to_rgba_tiny() {
    // empty picture
    assert_eq!(yuv420_to_rgba(&[], &[], &[], 0), vec![0u8; 0]);

    // a single pixel picture
    assert_eq!(
        yuv420_to_rgba(&[125u8], &[128u8], &[128u8], 1),
        vec![127u8, 127u8, 127u8, 255u8]
    );

    // a 2x2 grey picture with a single chroma sample (well, one Cb and one Cr)
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(&[125u8, 125u8, 125u8, 125u8], &[128u8], &[128u8], 2),
        vec![
            127u8, 127u8, 127u8, 255u8, 127u8, 127u8, 127u8, 255u8,
            127u8, 127u8, 127u8, 255u8, 127u8, 127u8, 127u8, 255u8,
        ]
    );

    // a 2x2 black-and-white checkerboard picture
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(&[16u8, 235u8, 235u8, 16u8], &[128u8], &[128u8], 2),
        vec![
              0u8,   0u8,   0u8, 255u8, 255u8, 255u8, 255u8, 255u8,
            255u8, 255u8, 255u8, 255u8,   0u8,   0u8,   0u8, 255u8,
        ]
    );

    // a 3x2 picture, black on the left, white on the right, grey in the middle
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(&[0u8, 125u8, 235u8,  0u8, 125u8, 235u8], &[128u8, 128u8, ], &[128u8, 128u8,], 3),
        vec![
              0u8,   0u8,   0u8, 255u8,  127u8, 127u8, 127u8, 255u8,  255u8, 255u8, 255u8, 255u8,
              0u8,   0u8,   0u8, 255u8,  127u8, 127u8, 127u8, 255u8,  255u8, 255u8, 255u8, 255u8,
        ]
    );

    // notes:
    // (81, 90, 240) is full red in YUV
    // (145, 54, 34) is full green in YUV

    // A 3x3 picture, red on the top, green on the bottom.
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(
            &[ 81u8,  81u8,  81u8,
              125u8, 125u8, 125u8,
              145u8, 145u8, 145u8],
            &[ 90u8,  90u8,
               54u8,  54u8],
            &[240u8,  240u8,
               34u8,  34u8],
            3),
        vec![
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, // red, with rounding error
            255u8,  51u8,  50u8, 255u8,  255u8,  51u8,  50u8, 255u8,  255u8,  51u8,  50u8, 255u8, // orangish
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8, // green, with rounding error
        ]
    );
    // The middle row looks fairly off when converted back to YUV: should be (125, 90, 240), but is (112, 97, 218)
    // However, when converted back again to RGB, these are (255, 51, 50) and (255, 51, 49), respectively. So, close enough.

    // A 3x3 picture, red on the left, green on the right. Transpose of the above.
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(
            &[ 81u8, 125u8, 145u8,
               81u8, 125u8, 145u8,
               81u8, 125u8, 145u8],
            &[ 90u8,  54u8,
               90u8,  54u8],
            &[240u8,   34u8,
              240u8,   34u8],
            3),
        vec![
            254u8,   0u8,   0u8, 255u8,  255u8,  51u8,  50u8, 255u8,   0u8, 255u8,   1u8, 255u8,
            254u8,   0u8,   0u8, 255u8,  255u8,  51u8,  50u8, 255u8,   0u8, 255u8,   1u8, 255u8,
            254u8,   0u8,   0u8, 255u8,  255u8,  51u8,  50u8, 255u8,   0u8, 255u8,   1u8, 255u8,
        ]
    );

    // The middle row/column of pixels use the top/left row/column of chroma samples:
    assert_eq!(yuv_to_rgb((125, 90, 240)), (255, 51, 50));
}

#[test]
fn test_yuv420_to_rgba_medium() {
    // A 4x4 picture, red on the top, green on the bottom.
    // This should be done by SIMD now.
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(
            &[ 81u8,  81u8,  81u8,  81u8,
               81u8,  81u8,  81u8,  81u8,
              145u8, 145u8, 145u8, 145u8,
              145u8, 145u8, 145u8, 145u8],
            &[ 90u8,  90u8,
               54u8,  54u8],
            &[240u8,  240u8,
               34u8,  34u8],
            4),
        vec![
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8, // red, with rounding error
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8, // red, with rounding error
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8, // green, with rounding error
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8, // green, with rounding error
        ]
    );

    // A 5x4 picture, red on the top, green on the bottom.
    // This should be done by SIMD now, plus one row of remainder.
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(
            &[ 81u8,  81u8,  81u8,  81u8,  81u8,
               81u8,  81u8,  81u8,  81u8,  81u8,
              145u8, 145u8, 145u8, 145u8, 145u8,
              145u8, 145u8, 145u8, 145u8, 145u8],
            &[ 90u8,  90u8,  90u8,
               54u8,  54u8,  54u8],
            &[240u8,  240u8, 240u8,
               34u8,  34u8,  34u8],
            5),
        vec![
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8,
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8,
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8,
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8,
        ]
    );

    // Same as before, but the last column is upside down, to check if it uses the right values.
    #[rustfmt::skip]
    assert_eq!(
        yuv420_to_rgba(
            &[ 81u8,  81u8,  81u8,  81u8, 145u8,
               81u8,  81u8,  81u8,  81u8, 145u8,
              145u8, 145u8, 145u8, 145u8,  81u8,
              145u8, 145u8, 145u8, 145u8,  81u8],
            &[ 90u8,  90u8,  54u8,
               54u8,  54u8,  90u8],
            &[240u8, 240u8,  34u8,
               34u8,  34u8, 240u8],
            5),
        vec![
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8,   0u8, 255u8,   1u8, 255u8, // red, with rounding error
            254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8,  254u8,   0u8,   0u8, 255u8, 254u8,   0u8,   0u8, 255u8,   0u8, 255u8,   1u8, 255u8, // red, with rounding error
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8, 254u8,   0u8,   0u8, 255u8, // green, with rounding error
              0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,    0u8, 255u8,   1u8, 255u8,   0u8, 255u8,   1u8, 255u8, 254u8,   0u8,   0u8, 255u8, // green, with rounding error
        ]
    );
}

use swf::DefineBitsLossless;

use super::decode::{Bitmap, decode_define_bits_jpeg, decode_define_bits_lossless, error::Error};

#[derive(Clone, Debug)]
pub enum CompressedBitmap {
    Jpeg {
        data: Vec<u8>,
        alpha: Option<Vec<u8>>,
        width: u16,
        height: u16,
    },
    Lossless(DefineBitsLossless<'static>),
}

impl CompressedBitmap {
    pub fn size(&self) -> BitmapSize {
        match self {
            CompressedBitmap::Jpeg { width, height, .. } => BitmapSize {
                width: *width,
                height: *height,
            },
            CompressedBitmap::Lossless(DefineBitsLossless { width, height, .. }) => BitmapSize {
                width: *width,
                height: *height,
            },
        }
    }

    pub fn decode(&self) -> Result<Bitmap, Error> {
        match self {
            CompressedBitmap::Jpeg { data, alpha, .. } => {
                decode_define_bits_jpeg(data, alpha.as_deref())
            }
            CompressedBitmap::Lossless(define_bits_lossless) => {
                decode_define_bits_lossless(define_bits_lossless)
            }
        }
    }
}

pub struct BitmapSize {
    pub width: u16,
    pub height: u16,
}

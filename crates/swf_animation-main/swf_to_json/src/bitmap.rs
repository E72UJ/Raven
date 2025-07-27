use std::collections::HashMap;

use ruffle_render::bitmap::BitmapSize;
use swf::{CharacterId, DefineBitsLossless};

pub struct BitmapLib {
    bitmap: HashMap<CharacterId, CompressedBitmap>,
}

impl BitmapLib {
    pub fn get_bitmap(&self, id: CharacterId) -> Option<&CompressedBitmap> {
        self.bitmap.get(&id)
    }
}

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

    pub fn decode(&self) -> Result<ruffle_render::bitmap::Bitmap, ruffle_render::error::Error> {
        match self {
            CompressedBitmap::Jpeg { data, alpha, .. } => {
                ruffle_render::utils::decode_define_bits_jpeg(data, alpha.as_deref())
            }
            CompressedBitmap::Lossless(define_bits_lossless) => {
                ruffle_render::utils::decode_define_bits_lossless(define_bits_lossless)
            }
        }
    }
}

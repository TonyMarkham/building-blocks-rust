use std::fmt::{Display, Formatter, Result as FormatResult};

/// Every PNG file starts with these 8 bytes.
const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];

/// Bytes needed to read the IHDR fields we care about:
/// 8 signature + 4 chunk length + 4 chunk type + 4 width + 4 height + 1 bit depth + 1 colour type.
pub(crate) const HEADER_LEN: u64 = 26;

#[derive(Debug)]
pub struct Info {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bit_depth: u8,
    pub(crate) color_type: u8,
}

/// Parse the PNG signature and IHDR chunk. Returns `None` if the bytes aren't a PNG.
pub(crate) fn parse(bytes: &[u8]) -> Option<Info> {
    let (signature, rest) = bytes.split_first_chunk::<8>()?;
    if *signature != SIGNATURE {
        return None;
    }

    let (_chunk_len, rest) = rest.split_first_chunk::<4>()?;
    let (chunk_type, rest) = rest.split_first_chunk::<4>()?;
    if chunk_type != b"IHDR" {
        return None;
    }

    // PNG stores integers big-endian.
    let (width, rest) = rest.split_first_chunk::<4>()?;
    let (height, rest) = rest.split_first_chunk::<4>()?;
    let &[bit_depth, color_type, ..] = rest else {
        return None;
    };

    Some(Info {
        width: u32::from_be_bytes(*width),
        height: u32::from_be_bytes(*height),
        bit_depth,
        color_type,
    })
}

impl Info {
    fn color_name(&self) -> &'static str {
        match self.color_type {
            0 => "grayscale",
            2 => "RGB",
            3 => "indexed (palette)",
            4 => "grayscale + alpha",
            6 => "RGBA",
            _ => "unknown",
        }
    }
}

impl Display for Info {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write!(
            formatter,
            "{} x {} px, {}-bit {}",
            self.width,
            self.height,
            self.bit_depth,
            self.color_name()
        )
    }
}
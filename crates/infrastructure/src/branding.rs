use std::io::Cursor;

use png::{ColorType, Decoder, Transformations};
use rustpaper_application::AppError;

const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-64.png");
const RGBA_CHANNELS: usize = 4;

#[derive(Debug, Clone)]
pub struct RgbaImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn app_icon() -> Result<RgbaImage, AppError> {
    decode_png(ICON_PNG)
}

pub fn decode_png(bytes: &[u8]) -> Result<RgbaImage, AppError> {
    let image_error = |error: png::DecodingError| AppError::Storage(error.to_string());
    let mut decoder = Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::normalize_to_color8() | Transformations::ALPHA);
    let mut reader = decoder.read_info().map_err(image_error)?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| AppError::Storage("image trop grande".into()))?;
    let mut buffer = vec![0; buffer_size];
    let frame = reader.next_frame(&mut buffer).map_err(image_error)?;
    buffer.truncate(frame.buffer_size());
    let rgba = match frame.color_type {
        ColorType::Rgba => buffer,
        ColorType::GrayscaleAlpha => buffer
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[gray, alpha]| [gray, gray, gray, alpha])
            .collect(),
        other => {
            return Err(AppError::Storage(format!(
                "format PNG non géré : {other:?}"
            )))
        }
    };
    debug_assert_eq!(
        rgba.len(),
        frame.width as usize * frame.height as usize * RGBA_CHANNELS
    );
    Ok(RgbaImage {
        rgba,
        width: frame.width,
        height: frame.height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_embedded_icon() {
        let icon = app_icon().unwrap();
        assert_eq!(
            icon.rgba.len(),
            (icon.width * icon.height) as usize * RGBA_CHANNELS
        );
    }
}

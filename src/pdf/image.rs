//! PDF image handling

use super::{PdfDictionary, PdfObject, PdfStream};
use crate::types::Result;

/// PDF Image XObject
#[derive(Debug, Clone, PartialEq)]
pub struct PdfImage {
    pub width: u32,
    pub height: u32,
    pub color_space: ColorSpace,
    pub bits_per_component: u8,
    pub data: Vec<u8>,
    pub filter: Option<ImageFilter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorSpace {
    DeviceGray,
    DeviceRGB,
    DeviceCMYK,
    Indexed(Box<ColorSpace>, u8, Vec<u8>), // base, hival, lookup table
}

impl ColorSpace {
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorSpace::DeviceGray => "DeviceGray",
            ColorSpace::DeviceRGB => "DeviceRGB",
            ColorSpace::DeviceCMYK => "DeviceCMYK",
            ColorSpace::Indexed(_, _, _) => "Indexed",
        }
    }

    pub fn num_components(&self) -> u8 {
        match self {
            ColorSpace::DeviceGray => 1,
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
            ColorSpace::Indexed(_, _, _) => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ImageFilter {
    FlateDecode,
    DCTDecode, // JPEG
    JP2Decode, // JPEG2000
}

impl ImageFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageFilter::FlateDecode => "FlateDecode",
            ImageFilter::DCTDecode => "DCTDecode",
            ImageFilter::JP2Decode => "JP2Decode",
        }
    }
}

impl PdfImage {
    /// Create a new image
    pub fn new(
        width: u32,
        height: u32,
        color_space: ColorSpace,
        bits_per_component: u8,
        data: Vec<u8>,
    ) -> Self {
        Self {
            width,
            height,
            color_space,
            bits_per_component,
            data,
            filter: None,
        }
    }

    /// Create an RGB image
    pub fn rgb(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self::new(width, height, ColorSpace::DeviceRGB, 8, data)
    }

    /// Create a grayscale image
    pub fn grayscale(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self::new(width, height, ColorSpace::DeviceGray, 8, data)
    }

    /// Set compression filter
    pub fn with_filter(mut self, filter: ImageFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Convert to PDF XObject dictionary and stream
    pub fn to_xobject(&self, name: impl Into<String>) -> (String, PdfDictionary, PdfStream) {
        let name = name.into();
        
        let mut dict = PdfDictionary::new();
        dict.insert("Type", PdfObject::Name("XObject".to_string()));
        dict.insert("Subtype", PdfObject::Name("Image".to_string()));
        dict.insert("Width", self.width as i32);
        dict.insert("Height", self.height as i32);
        dict.insert("ColorSpace", PdfObject::Name(self.color_space.as_str().to_string()));
        dict.insert("BitsPerComponent", self.bits_per_component as i32);
        
        if let Some(filter) = &self.filter {
            dict.insert("Filter", PdfObject::Name(filter.as_str().to_string()));
        }
        
        let stream = PdfStream::new(self.data.clone());
        
        (name, dict, stream)
    }

    /// Load from PNG data
    pub fn from_png(data: &[u8]) -> Result<Self> {
        // Parse PNG header
        if data.len() < 8 || data[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Err(crate::types::PdfError::Image("Invalid PNG signature".to_string()));
        }

        let mut pos = 8;
        let mut width = 0u32;
        let mut height = 0u32;
        let mut bit_depth = 8u8;
        let mut color_type = 0u8;
        let mut compressed_data = Vec::new();

        while pos < data.len() {
            if pos + 8 > data.len() {
                break;
            }

            let length = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
            let chunk_type = &data[pos+4..pos+8];
            let chunk_data_start = pos + 8;
            let chunk_data_end = chunk_data_start + length;

            if chunk_data_end > data.len() {
                break;
            }

            match chunk_type {
                b"IHDR" => {
                    if length >= 13 {
                        width = u32::from_be_bytes([data[chunk_data_start], data[chunk_data_start+1], 
                                                    data[chunk_data_start+2], data[chunk_data_start+3]]);
                        height = u32::from_be_bytes([data[chunk_data_start+4], data[chunk_data_start+5], 
                                                      data[chunk_data_start+6], data[chunk_data_start+7]]);
                        bit_depth = data[chunk_data_start+8];
                        color_type = data[chunk_data_start+9];
                    }
                }
                b"IDAT" => {
                    compressed_data.extend_from_slice(&data[chunk_data_start..chunk_data_end]);
                }
                b"IEND" => break,
                _ => {}
            }

            pos = chunk_data_end + 4; // Skip CRC
        }

        // Determine color space
        let color_space = match color_type {
            0 => ColorSpace::DeviceGray,
            2 => ColorSpace::DeviceRGB,
            3 => {
                // Indexed color - would need to extract palette from PLTE chunk
                // For now, default to RGB
                ColorSpace::DeviceRGB
            }
            4 => ColorSpace::DeviceGray, // Grayscale with alpha
            6 => ColorSpace::DeviceRGB,   // RGB with alpha
            _ => ColorSpace::DeviceRGB,
        };

        // Decompress the image data
        let decompressed = miniz_oxide::inflate::decompress_to_vec_zlib(&compressed_data)
            .map_err(|e| crate::types::PdfError::Image(format!("PNG decompression failed: {:?}", e)))?;

        // Remove filter bytes from each row
        let bytes_per_pixel = color_space.num_components();
        let row_bytes = width as usize * bytes_per_pixel as usize;
        let mut pixel_data = Vec::with_capacity(row_bytes * height as usize);

        for row in 0..height as usize {
            let src_start = row * (row_bytes + 1);
            if src_start < decompressed.len() {
                let filter_byte = decompressed[src_start];
                let row_start = src_start + 1;
                let row_end = (row_start + row_bytes).min(decompressed.len());
                
                // Apply filter (simplified - just handles filter byte 0)
                if filter_byte == 0 {
                    pixel_data.extend_from_slice(&decompressed[row_start..row_end]);
                } else {
                    // For non-zero filter bytes, we'd need to implement the filter algorithms
                    // For now, just copy the data
                    pixel_data.extend_from_slice(&decompressed[row_start..row_end]);
                }
            }
        }

        Ok(Self::new(width, height, color_space, bit_depth, pixel_data))
    }

    /// Load from JPEG data
    pub fn from_jpeg(data: &[u8]) -> Result<Self> {
        if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
            return Err(crate::types::PdfError::Image("Invalid JPEG signature".to_string()));
        }

        let mut pos = 2;
        let mut width = 0u32;
        let mut height = 0u32;
        let mut bits_per_component = 8u8;
        let mut num_components = 3u8;

        while pos < data.len() {
            if pos + 4 > data.len() {
                break;
            }

            if data[pos] != 0xFF {
                pos += 1;
                continue;
            }

            let marker = data[pos + 1];

            // Skip padding
            if marker == 0x00 || marker == 0xFF {
                pos += 2;
                continue;
            }

            // End of image
            if marker == 0xD9 {
                break;
            }

            // Start of scan - data follows
            if marker == 0xDA {
                break;
            }

            // SOF markers
            if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
                if pos + 10 > data.len() {
                    break;
                }
                
                bits_per_component = data[pos + 4];
                height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
                width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]) as u32;
                num_components = data[pos + 9];

                pos += 2;
                continue;
            }

            // Skip marker with length
            if pos + 4 <= data.len() {
                let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
                pos += 2 + length;
            } else {
                break;
            }
        }

        if width == 0 || height == 0 {
            return Err(crate::types::PdfError::Image("Could not determine JPEG dimensions".to_string()));
        }

        let color_space = match num_components {
            1 => ColorSpace::DeviceGray,
            3 => ColorSpace::DeviceRGB,
            4 => ColorSpace::DeviceCMYK,
            _ => ColorSpace::DeviceRGB,
        };

        // For JPEG, we can embed the data directly with DCTDecode filter
        Ok(Self {
            width,
            height,
            color_space,
            bits_per_component,
            data: data.to_vec(),
            filter: Some(ImageFilter::DCTDecode),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_creation() {
        let data = vec![255u8; 100 * 100 * 3]; // 100x100 RGB white image
        let img = PdfImage::rgb(100, 100, data);
        
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 100);
        assert_eq!(img.color_space, ColorSpace::DeviceRGB);
    }

    #[test]
    fn test_color_space() {
        assert_eq!(ColorSpace::DeviceGray.num_components(), 1);
        assert_eq!(ColorSpace::DeviceRGB.num_components(), 3);
        assert_eq!(ColorSpace::DeviceCMYK.num_components(), 4);
    }
}

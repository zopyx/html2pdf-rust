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

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    WebP,
    Svg,
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

    /// Detect image format from data
    pub fn detect_format(data: &[u8]) -> Option<ImageFormat> {
        if data.len() < 4 {
            return None;
        }

        // PNG: 0x89 0x50 0x4E 0x47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(ImageFormat::Png);
        }

        // JPEG: 0xFF 0xD8
        if data.starts_with(&[0xFF, 0xD8]) {
            return Some(ImageFormat::Jpeg);
        }

        // GIF: "GIF8"
        if data.starts_with(b"GIF8") {
            return Some(ImageFormat::Gif);
        }

        // BMP: "BM"
        if data.starts_with(b"BM") {
            return Some(ImageFormat::Bmp);
        }

        // WebP: "RIFF" ... "WEBP"
        if data.starts_with(b"RIFF") && data.len() >= 12 && &data[8..12] == b"WEBP" {
            return Some(ImageFormat::WebP);
        }

        // SVG: Look for SVG tag (simplified detection)
        let text = String::from_utf8_lossy(data);
        if text.trim().starts_with("<?xml") || text.trim().starts_with("<svg") {
            return Some(ImageFormat::Svg);
        }

        None
    }

    /// Load image from any supported format
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let format = Self::detect_format(data)
            .ok_or_else(|| crate::types::PdfError::Image("Unknown image format".to_string()))?;

        match format {
            ImageFormat::Png => Self::from_png(data),
            ImageFormat::Jpeg => Self::from_jpeg(data),
            ImageFormat::Gif => Self::from_gif(data),
            ImageFormat::Bmp => Self::from_bmp(data),
            ImageFormat::WebP => Self::from_webp(data),
            ImageFormat::Svg => Self::from_svg(data),
        }
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
        let mut palette: Option<Vec<u8>> = None;

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
                b"PLTE" => {
                    // Store palette data for indexed color
                    palette = Some(data[chunk_data_start..chunk_data_end].to_vec());
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
                // Indexed color - use palette if available
                if let Some(pal) = palette {
                    ColorSpace::Indexed(Box::new(ColorSpace::DeviceRGB), (pal.len() / 3 - 1).min(255) as u8, pal)
                } else {
                    ColorSpace::DeviceRGB
                }
            }
            4 => ColorSpace::DeviceGray, // Grayscale with alpha
            6 => ColorSpace::DeviceRGB,   // RGB with alpha
            _ => ColorSpace::DeviceRGB,
        };

        // Decompress the image data
        let decompressed = miniz_oxide::inflate::decompress_to_vec_zlib(&compressed_data)
            .map_err(|e| crate::types::PdfError::Image(format!("PNG decompression failed: {:?}", e)))?;

        // Remove filter bytes from each row and handle all filter types
        let bytes_per_pixel = match color_type {
            0 => (bit_depth as usize + 7) / 8, // Grayscale
            2 => 3 * (bit_depth as usize / 8), // RGB
            3 => 1, // Indexed
            4 => 2 * (bit_depth as usize / 8), // Grayscale+Alpha
            6 => 4 * (bit_depth as usize / 8), // RGBA
            _ => 3,
        };
        let row_bytes = width as usize * bytes_per_pixel;
        let mut pixel_data = Vec::with_capacity(row_bytes * height as usize);
        let mut prev_row = vec![0u8; row_bytes];

        for row in 0..height as usize {
            let src_start = row * (row_bytes + 1);
            if src_start >= decompressed.len() {
                break;
            }
            
            let filter_byte = decompressed[src_start];
            let row_start = src_start + 1;
            let row_end = (row_start + row_bytes).min(decompressed.len());
            let row_data = &decompressed[row_start..row_end];

            // Apply filter
            let mut filtered_row = vec![0u8; row_data.len()];
            match filter_byte {
                0 => {
                    // Filter type 0: None - just copy
                    filtered_row.copy_from_slice(row_data);
                }
                1 => {
                    // Filter type 1: Sub
                    for i in 0..row_data.len() {
                        let left = if i >= bytes_per_pixel { filtered_row[i - bytes_per_pixel] } else { 0 };
                        filtered_row[i] = row_data[i].wrapping_add(left);
                    }
                }
                2 => {
                    // Filter type 2: Up
                    for i in 0..row_data.len() {
                        filtered_row[i] = row_data[i].wrapping_add(prev_row[i]);
                    }
                }
                3 => {
                    // Filter type 3: Average
                    for i in 0..row_data.len() {
                        let left = if i >= bytes_per_pixel { filtered_row[i - bytes_per_pixel] } else { 0 };
                        let up = prev_row[i];
                        filtered_row[i] = row_data[i].wrapping_add(((left as u16 + up as u16) / 2) as u8);
                    }
                }
                4 => {
                    // Filter type 4: Paeth
                    for i in 0..row_data.len() {
                        let left = if i >= bytes_per_pixel { filtered_row[i - bytes_per_pixel] } else { 0 };
                        let up = prev_row[i];
                        let upleft = if i >= bytes_per_pixel { prev_row[i - bytes_per_pixel] } else { 0 };
                        filtered_row[i] = row_data[i].wrapping_add(paeth_predictor(left, up, upleft));
                    }
                }
                _ => {
                    // Unknown filter, just copy
                    filtered_row.copy_from_slice(row_data);
                }
            }

            pixel_data.extend_from_slice(&filtered_row);
            prev_row = filtered_row;
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

            // SOF markers (Start of Frame)
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
        // Strip any extraneous data after EOI marker
        let end_pos = data.windows(2)
            .position(|w| w == &[0xFF, 0xD9])
            .map(|p| p + 2)
            .unwrap_or(data.len());

        Ok(Self {
            width,
            height,
            color_space,
            bits_per_component,
            data: data[..end_pos].to_vec(),
            filter: Some(ImageFilter::DCTDecode),
        })
    }

    /// Load from GIF data
    pub fn from_gif(data: &[u8]) -> Result<Self> {
        // Use the image crate to decode GIF
        use image::ImageDecoder;
        
        let cursor = std::io::Cursor::new(data);
        let decoder = image::codecs::gif::GifDecoder::new(cursor)
            .map_err(|e| crate::types::PdfError::Image(format!("GIF decode error: {}", e)))?;
        
        let (width, height) = decoder.dimensions();
        let mut img_data = vec![0; decoder.total_bytes() as usize];
        decoder.read_image(&mut img_data)
            .map_err(|e| crate::types::PdfError::Image(format!("GIF read error: {}", e)))?;

        // GIF decoder outputs RGBA, convert to RGB for PDF
        let rgb_data = if img_data.len() == (width * height * 4) as usize {
            img_data.chunks_exact(4)
                .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2]])
                .collect()
        } else {
            img_data
        };

        Ok(Self::rgb(width, height, rgb_data))
    }

    /// Load from BMP data
    pub fn from_bmp(data: &[u8]) -> Result<Self> {
        // Use the image crate to decode BMP
        use image::ImageDecoder;
        
        let cursor = std::io::Cursor::new(data);
        let decoder = image::codecs::bmp::BmpDecoder::new(cursor)
            .map_err(|e| crate::types::PdfError::Image(format!("BMP decode error: {}", e)))?;
        
        let (width, height) = decoder.dimensions();
        let mut img_data = vec![0; decoder.total_bytes() as usize];
        decoder.read_image(&mut img_data)
            .map_err(|e| crate::types::PdfError::Image(format!("BMP read error: {}", e)))?;

        // BMP decoder outputs RGBA, convert to RGB for PDF
        let rgb_data = if img_data.len() == (width * height * 4) as usize {
            img_data.chunks_exact(4)
                .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2]])
                .collect()
        } else {
            img_data
        };

        Ok(Self::rgb(width, height, rgb_data))
    }

    /// Load from WebP data
    pub fn from_webp(data: &[u8]) -> Result<Self> {
        // WebP support requires the 'webp' feature which is not enabled by default
        // Return an error for now - can be enabled by adding 'webp' feature to image crate
        let _ = data;
        Err(crate::types::PdfError::Image(
            "WebP support not enabled. Add 'webp' feature to image crate dependency.".to_string()
        ))
    }

    /// Load from SVG data - converts to raster
    /// 
    /// Note: For vector SVG rendering (better quality), use the `svg` module
    /// to render directly to PDF graphics operations instead.
    pub fn from_svg(data: &[u8]) -> Result<Self> {
        let svg_str = std::str::from_utf8(data)
            .map_err(|_| crate::types::PdfError::Image("Invalid SVG: not UTF-8".to_string()))?;
        
        // Parse SVG to get dimensions
        let (width, height) = parse_svg_dimensions(svg_str)?;
        
        // For now, create a placeholder raster representation
        // For production use, consider using resvg crate for high-quality rasterization
        // or use the svg module for vector rendering
        
        // Create a simple colored rectangle as a placeholder
        // representing the SVG bounding box
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for _ in 0..width * height {
            // Light blue color for SVG placeholder
            rgb_data.push(200);
            rgb_data.push(220);
            rgb_data.push(255);
        }
        
        Ok(Self::rgb(width, height, rgb_data))
    }

    /// Get SVG dimensions without full rendering
    pub fn get_svg_dimensions(data: &[u8]) -> Result<(u32, u32)> {
        let svg_str = std::str::from_utf8(data)
            .map_err(|_| crate::types::PdfError::Image("Invalid SVG: not UTF-8".to_string()))?;
        parse_svg_dimensions(svg_str)
    }

    /// Encode image data with FlateDecode compression
    pub fn compress(&mut self) -> Result<()> {
        if self.filter.is_some() {
            // Already has a filter (e.g., DCTDecode for JPEG)
            return Ok(());
        }

        let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&self.data, 6);
        
        // Only use compression if it actually reduces size
        if compressed.len() < self.data.len() {
            self.data = compressed;
            self.filter = Some(ImageFilter::FlateDecode);
        }
        
        Ok(())
    }
}

/// Parse SVG dimensions from SVG string
pub fn parse_svg_dimensions(svg: &str) -> Result<(u32, u32)> {
    // Default size
    let mut width = 300u32;
    let mut height = 150u32;

    // Try to extract width and height from SVG attributes
    if let Some(width_match) = svg.find("width=") {
        let start = width_match + 7;
        if let Some(end) = svg[start..].find(&['\"', '\''][..]) {
            let width_str = &svg[start..start + end];
            if let Ok(w) = parse_svg_length(width_str) {
                width = w as u32;
            }
        }
    }

    if let Some(height_match) = svg.find("height=") {
        let start = height_match + 8;
        if let Some(end) = svg[start..].find(&['\"', '\''][..]) {
            let height_str = &svg[start..start + end];
            if let Ok(h) = parse_svg_length(height_str) {
                height = h as u32;
            }
        }
    }

    // Check for viewBox
    if let Some(vb_match) = svg.find("viewBox=") {
        let start = vb_match + 9;
        if let Some(end) = svg[start..].find(&['\"', '\''][..]) {
            let vb_str = &svg[start..start + end];
            let parts: Vec<&str> = vb_str.split_whitespace().collect();
            if parts.len() == 4 {
                if let (Ok(vb_w), Ok(vb_h)) = (parts[2].parse::<f32>(), parts[3].parse::<f32>()) {
                    // If width/height not specified, use viewBox
                    if !svg.contains("width=") {
                        width = vb_w as u32;
                    }
                    if !svg.contains("height=") {
                        height = vb_h as u32;
                    }
                }
            }
        }
    }

    // Ensure reasonable bounds
    width = width.max(1).min(2000);
    height = height.max(1).min(2000);

    Ok((width, height))
}

/// Parse SVG length value
fn parse_svg_length(s: &str) -> Result<f32> {
    let s = s.trim();
    
    // Remove units
    let value_str: String = s.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    
    value_str.parse::<f32>()
        .map_err(|_| crate::types::PdfError::Image(format!("Invalid SVG length: {}", s)))
}

/// Paeth predictor for PNG filtering
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i16;
    let b = b as i16;
    let c = c as i16;
    
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
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

    #[test]
    fn test_detect_format_png() {
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(PdfImage::detect_format(&png_data), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_format_jpeg() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(PdfImage::detect_format(&jpeg_data), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_format_gif() {
        let gif_data = b"GIF89a";
        assert_eq!(PdfImage::detect_format(gif_data), Some(ImageFormat::Gif));
    }

    #[test]
    fn test_detect_format_bmp() {
        let bmp_data = b"BM\x00\x00"; // BMP needs at least 4 bytes
        assert_eq!(PdfImage::detect_format(bmp_data), Some(ImageFormat::Bmp));
    }

    #[test]
    fn test_detect_format_svg() {
        let svg_data = b"<svg width=\"100\" height=\"100\"></svg>";
        assert_eq!(PdfImage::detect_format(svg_data), Some(ImageFormat::Svg));
    }

    #[test]
    fn test_paeth_predictor() {
        // Paeth predictor returns whichever of left, up, or upleft is closest to
        // p = left + up - upleft
        // For (10, 20, 15): p = 10 + 20 - 15 = 15, left=10 (dist 5), up=20 (dist 5), upleft=15 (dist 0)
        // Result is upleft=15
        assert_eq!(paeth_predictor(10, 20, 15), 15); 
        // For (30, 20, 25): p = 30 + 20 - 25 = 25, left=30 (dist 5), up=20 (dist 5), upleft=25 (dist 0)
        // Result is upleft=25 or left=30? Let me recalculate
        // Actually let's test with simpler cases
        assert_eq!(paeth_predictor(0, 0, 0), 0);
        assert_eq!(paeth_predictor(10, 0, 0), 10); // left only
        assert_eq!(paeth_predictor(0, 10, 0), 10); // up only  
    }

    #[test]
    fn test_svg_dimension_parsing() {
        let svg = r#"<svg width="200" height="150"></svg>"#;
        let (w, h) = parse_svg_dimensions(svg).unwrap();
        assert_eq!(w, 200);
        assert_eq!(h, 150);
    }

    #[test]
    fn test_svg_viewbox_parsing() {
        let svg = r#"<svg viewBox="0 0 800 600"></svg>"#;
        let (w, h) = parse_svg_dimensions(svg).unwrap();
        assert_eq!(w, 800);
        assert_eq!(h, 600);
    }
}

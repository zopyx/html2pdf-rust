//! PDF stream handling with Flate compression

use miniz_oxide::deflate::compress_to_vec_zlib;

/// PDF stream object
#[derive(Debug, Clone, PartialEq)]
pub struct PdfStream {
    pub data: Vec<u8>,
    pub dictionary: super::PdfDictionary,
}

impl PdfStream {
    /// Create a new stream
    pub fn new(data: Vec<u8>) -> Self {
        let mut dict = super::PdfDictionary::new();
        dict.insert("Length", data.len() as i32);
        
        Self {
            data,
            dictionary: dict,
        }
    }

    /// Create a new stream with custom dictionary
    pub fn with_dictionary(data: Vec<u8>, dictionary: super::PdfDictionary) -> Self {
        Self { data, dictionary }
    }

    /// Get the length of the stream data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Update the Length entry in dictionary
    pub fn update_length(&mut self) {
        self.dictionary.insert("Length", self.data.len() as i32);
    }
}

/// Trait for stream encoding
pub trait StreamEncode {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
    fn filter_name(&self) -> &'static str;
}

/// Flate (zlib) compression encoder
pub struct FlateEncode {
    level: u8,
}

impl FlateEncode {
    pub const fn new() -> Self {
        Self { level: 6 }
    }

    pub const fn with_level(level: u8) -> Self {
        Self { level }
    }
}

impl Default for FlateEncode {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamEncode for FlateEncode {
    fn encode(&self, data: &[u8]) -> Vec<u8> {
        compress_to_vec_zlib(data, self.level)
    }

    fn filter_name(&self) -> &'static str {
        "FlateDecode"
    }
}

/// Encode a stream with the given encoder
pub fn encode_stream(encoder: &dyn StreamEncode, data: &[u8]) -> (Vec<u8>, String) {
    let encoded = encoder.encode(data);
    let filter_name = encoder.filter_name().to_string();
    (encoded, filter_name)
}

/// Predictor filter for image data
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Predictor {
    None = 1,
    Tiff = 2,
    PngNone = 10,
    PngSub = 11,
    PngUp = 12,
    PngAverage = 13,
    PngPaeth = 14,
    PngOptimum = 15,
}

impl Predictor {
    pub fn apply(&self, data: &[u8], width: usize, components: usize) -> Vec<u8> {
        match self {
            Predictor::None | Predictor::Tiff => data.to_vec(),
            Predictor::PngNone => {
                // PNG None: just add filter byte (0) before each row
                let row_size = width * components;
                let rows = data.len() / row_size;
                let mut result = Vec::with_capacity(data.len() + rows);
                
                for i in 0..rows {
                    result.push(0); // Filter byte
                    let start = i * row_size;
                    result.extend_from_slice(&data[start..start + row_size]);
                }
                result
            }
            Predictor::PngUp => {
                // PNG Up: difference from pixel above
                let row_size = width * components;
                let rows = (data.len() + row_size - 1) / row_size;
                let mut result = Vec::with_capacity(data.len() + rows);
                let mut prev_row = vec![0u8; row_size];
                
                for i in 0..rows {
                    result.push(2); // Filter byte for "Up"
                    let start = i * row_size;
                    let end = (start + row_size).min(data.len());
                    
                    for j in 0..(end - start) {
                        result.push(data[start + j].wrapping_sub(prev_row[j]));
                    }
                    prev_row[..(end - start)].copy_from_slice(&data[start..end]);
                }
                result
            }
            _ => data.to_vec(), // Fallback for unimplemented predictors
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_creation() {
        let data = b"Hello, World!".to_vec();
        let stream = PdfStream::new(data.clone());
        
        assert_eq!(stream.len(), data.len());
        assert!(!stream.is_empty());
    }

    #[test]
    fn test_flate_encode() {
        let data = b"This is a test string that should be compressed. "
            .repeat(100);
        
        let encoder = FlateEncode::new();
        let encoded = encoder.encode(&data);
        
        // Compressed should be smaller for repetitive data
        assert!(encoded.len() < data.len());
    }

    #[test]
    fn test_predictor_png_none() {
        let data: Vec<u8> = (0..12).collect(); // 4x3 RGB image
        let predictor = Predictor::PngNone;
        let result = predictor.apply(&data, 4, 3);
        
        // Should have filter byte + 12 bytes
        assert_eq!(result.len(), 13);
        assert_eq!(result[0], 0); // Filter byte
    }
}

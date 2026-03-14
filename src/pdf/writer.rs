//! PDF file writer - assembles all objects into a valid PDF document

use super::{
    object::{PdfDictionary, PdfObject, PdfReference, PdfArray},
    stream::PdfStream,
    PageContent,
};
use crate::types::{PaperSize, Orientation, Margins};
use std::collections::HashMap;
use std::io::{self, Write};

/// PDF document writer
pub struct PdfWriter {
    objects: Vec<(PdfReference, PdfObject)>,
    pages: Vec<PdfReference>,
    fonts: HashMap<String, PdfReference>,
    images: HashMap<String, PdfReference>,
    page_width: f32,
    page_height: f32,
    margins: Margins,
    next_object_number: u32,
    catalog_ref: Option<PdfReference>,
    pages_tree_ref: Option<PdfReference>,
    info_ref: Option<PdfReference>,
}

impl PdfWriter {
    /// Create a new PDF writer
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            pages: Vec::new(),
            fonts: HashMap::new(),
            images: HashMap::new(),
            page_width: 595.28, // A4 width in points
            page_height: 841.89, // A4 height in points
            margins: Margins::all(72.0), // 1 inch margins
            next_object_number: 1,
            catalog_ref: None,
            pages_tree_ref: None,
            info_ref: None,
        }
    }

    /// Set paper size
    pub fn set_paper_size(&mut self, size: PaperSize, orientation: Orientation) {
        let (w, h) = size.size();
        match orientation {
            Orientation::Portrait => {
                self.page_width = w;
                self.page_height = h;
            }
            Orientation::Landscape => {
                self.page_width = h;
                self.page_height = w;
            }
        }
    }

    /// Set page margins
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = margins;
    }

    /// Get the current page dimensions
    pub fn page_size(&self) -> (f32, f32) {
        (self.page_width, self.page_height)
    }

    /// Get the content area (page minus margins)
    pub fn content_area(&self) -> (f32, f32, f32, f32) {
        (
            self.margins.left,
            self.margins.bottom,
            self.page_width - self.margins.left - self.margins.right,
            self.page_height - self.margins.top - self.margins.bottom,
        )
    }

    /// Allocate a new object reference
    fn allocate_ref(&mut self) -> PdfReference {
        let ref_num = PdfReference::new(self.next_object_number, 0);
        self.next_object_number += 1;
        ref_num
    }

    /// Add an object and return its reference
    fn add_object(&mut self, obj: PdfObject) -> PdfReference {
        let reference = self.allocate_ref();
        self.objects.push((reference, obj));
        reference
    }

    /// Initialize the document structure (catalog, pages tree)
    pub fn init_document(&mut self) {
        // Create pages tree root
        let pages_tree = PdfDictionary::new();
        let pages_tree_ref = self.allocate_ref();
        self.pages_tree_ref = Some(pages_tree_ref);
        self.objects.push((pages_tree_ref, PdfObject::Dictionary(pages_tree)));

        // Create catalog
        let mut catalog = PdfDictionary::new();
        catalog.insert("Type", PdfObject::Name("Catalog".to_string()));
        catalog.insert_ref("Pages", pages_tree_ref);
        
        self.catalog_ref = Some(self.add_object(PdfObject::Dictionary(catalog)));
    }

    /// Add a page with content
    pub fn add_page(&mut self, content: PageContent) -> PdfReference {
        // Create content stream
        let content_bytes = content.into_bytes();
        let mut stream_dict = PdfDictionary::new();
        stream_dict.insert("Length", content_bytes.len() as i32);
        
        let stream = PdfStream::with_dictionary(content_bytes, stream_dict);
        let content_ref = self.add_object(PdfObject::Dictionary(PdfDictionary::new()));
        
        // We need to handle streams specially - for now, store as indirect object
        // In actual write, we'll handle the stream properly
        let mut content_obj = PdfDictionary::new();
        content_obj.insert("Length", stream.len() as i32);
        
        // Create page object
        let mut page = PdfDictionary::new();
        page.insert("Type", PdfObject::Name("Page".to_string()));
        page.insert_ref("Parent", self.pages_tree_ref.unwrap());
        page.insert_ref("Contents", content_ref);
        
        // MediaBox
        let mut media_box = PdfArray::new();
        media_box.push(0i32);
        media_box.push(0i32);
        media_box.push(self.page_width);
        media_box.push(self.page_height);
        page.insert("MediaBox", PdfObject::Array(media_box));
        
        // CropBox (same as MediaBox)
        let mut crop_box = PdfArray::new();
        crop_box.push(0i32);
        crop_box.push(0i32);
        crop_box.push(self.page_width);
        crop_box.push(self.page_height);
        page.insert("CropBox", PdfObject::Array(crop_box));
        
        // Add resources
        let mut resources = PdfDictionary::new();
        
        // Add fonts
        if !self.fonts.is_empty() {
            let mut font_dict = PdfDictionary::new();
            for (name, font_ref) in &self.fonts {
                font_dict.insert_ref(name.clone(), *font_ref);
            }
            resources.insert("Font", PdfObject::Dictionary(font_dict));
        }
        
        // Add images
        if !self.images.is_empty() {
            let mut xobject_dict = PdfDictionary::new();
            for (name, image_ref) in &self.images {
                xobject_dict.insert_ref(name.clone(), *image_ref);
            }
            resources.insert("XObject", PdfObject::Dictionary(xobject_dict));
        }
        
        // ProcSet
        let mut procset = PdfArray::new();
        procset.push(PdfObject::Name("PDF".to_string()));
        procset.push(PdfObject::Name("Text".to_string()));
        procset.push(PdfObject::Name("ImageB".to_string()));
        procset.push(PdfObject::Name("ImageC".to_string()));
        procset.push(PdfObject::Name("ImageI".to_string()));
        resources.insert("ProcSet", PdfObject::Array(procset));
        
        page.insert("Resources", PdfObject::Dictionary(resources));
        
        let page_ref = self.add_object(PdfObject::Dictionary(page));
        self.pages.push(page_ref);
        
        // Store the actual content stream separately
        // We'll need to fix this - for now, we'll overwrite with the stream
        if let Some((_, obj)) = self.objects.iter_mut().find(|(r, _)| *r == content_ref) {
            *obj = PdfObject::Stream(stream.data);
        }
        
        page_ref
    }

    /// Add a standard font
    pub fn add_standard_font(&mut self, name: &str, base_font: &str) -> PdfReference {
        let mut font = PdfDictionary::new();
        font.insert("Type", PdfObject::Name("Font".to_string()));
        font.insert("Subtype", PdfObject::Name("Type1".to_string()));
        font.insert("BaseFont", PdfObject::Name(base_font.to_string()));
        
        // Encoding
        font.insert("Encoding", PdfObject::Name("WinAnsiEncoding".to_string()));
        
        let font_ref = self.add_object(PdfObject::Dictionary(font));
        self.fonts.insert(name.to_string(), font_ref);
        font_ref
    }

    /// Add document info
    pub fn set_info(&mut self, title: &str, author: &str, creator: &str) {
        let mut info = PdfDictionary::new();
        info.insert("Title", PdfObject::String(title.as_bytes().to_vec()));
        info.insert("Author", PdfObject::String(author.as_bytes().to_vec()));
        info.insert("Creator", PdfObject::String(creator.as_bytes().to_vec()));
        info.insert("Producer", PdfObject::String("HTML2PDF Rust".as_bytes().to_vec()));
        
        // Creation date
        let now = std::time::SystemTime::now();
        let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let date = format!("D:{:014}Z00'00'", since_epoch.as_secs());
        info.insert("CreationDate", PdfObject::String(date.into_bytes()));
        
        self.info_ref = Some(self.add_object(PdfObject::Dictionary(info)));
    }

    /// Write the PDF to output
    pub fn write<W: Write + std::io::Seek>(mut self, output: &mut W) -> io::Result<()> {
        // Update pages tree
        if let Some(pages_tree_ref) = self.pages_tree_ref {
            let mut pages_tree = PdfDictionary::new();
            pages_tree.insert("Type", PdfObject::Name("Pages".to_string()));
            pages_tree.insert("Count", self.pages.len() as i32);
            
            let mut kids = PdfArray::new();
            for page_ref in &self.pages {
                kids.push_ref(*page_ref);
            }
            pages_tree.insert("Kids", PdfObject::Array(kids));
            
            // Replace the pages tree object
            if let Some((_, obj)) = self.objects.iter_mut().find(|(r, _)| *r == pages_tree_ref) {
                *obj = PdfObject::Dictionary(pages_tree);
            }
        }

        // Build cross-reference table
        let mut xref_offsets: Vec<(u32, u32)> = Vec::new();
        
        // PDF header
        output.write_all(b"%PDF-1.4\n")?;
        output.write_all(b"%\xE2\xE3\xCF\xD3\n")?; // Binary marker
        
        // Write objects
        for (reference, object) in &self.objects {
            let offset = output.stream_position()? as u32;
            xref_offsets.push((reference.object_number, offset));
            
            writeln!(output, "{} {} obj", reference.object_number, reference.generation)?;
            
            // Handle streams specially
            match object {
                PdfObject::Stream(data) => {
                    // Write stream dictionary and data
                    let mut dict_str = String::new();
                    let mut dict = PdfDictionary::new();
                    dict.insert("Length", data.len() as i32);
                    dict.write(&mut dict_str).unwrap();
                    
                    write!(output, "{}\nstream\n", dict_str)?;
                    output.write_all(data)?;
                    output.write_all(b"\nendstream\n")?;
                }
                _ => {
                    let mut obj_str = String::new();
                    object.write(&mut obj_str).unwrap();
                    writeln!(output, "{}", obj_str)?;
                }
            }
            
            writeln!(output, "endobj")?;
        }
        
        // Cross-reference table
        let xref_offset = output.stream_position()?;
        writeln!(output, "xref")?;
        writeln!(output, "0 {}", self.next_object_number)?;
        writeln!(output, "{:010} {:05} f ", 0, 65535)?;
        
        // Sort by object number
        let mut sorted_offsets: Vec<(u32, u32)> = xref_offsets;
        sorted_offsets.sort_by_key(|(num, _)| *num);
        
        for (_, offset) in sorted_offsets {
            writeln!(output, "{:010} {:05} n ", offset, 0)?;
        }
        
        // Trailer
        writeln!(output, "trailer")?;
        let mut trailer = PdfDictionary::new();
        trailer.insert("Size", self.next_object_number as i32);
        if let Some(catalog_ref) = self.catalog_ref {
            trailer.insert_ref("Root", catalog_ref);
        }
        if let Some(info_ref) = self.info_ref {
            trailer.insert_ref("Info", info_ref);
        }
        
        let mut trailer_str = String::new();
        trailer.write(&mut trailer_str).unwrap();
        writeln!(output, "{}", trailer_str)?;
        
        writeln!(output, "startxref")?;
        writeln!(output, "{}", xref_offset)?;
        writeln!(output, "%%EOF")?;
        
        Ok(())
    }
}

impl Default for PdfWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_writer() {
        let mut writer = PdfWriter::new();
        writer.init_document();
        writer.set_info("Test", "Author", "HTML2PDF");
        
        // Add a standard font
        writer.add_standard_font("F1", "Helvetica");
        
        // Create page content
        let mut content = PageContent::new();
        content.begin_text();
        content.set_font("F1", 12.0);
        content.text_position(100.0, 700.0);
        content.show_text("Hello, PDF!");
        content.end_text();
        
        writer.add_page(content);
        
        let mut output = std::io::Cursor::new(Vec::new());
        writer.write(&mut output).unwrap();
        let bytes = output.into_inner();
        
        // Check it's a valid PDF
        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF\n"));
    }
}

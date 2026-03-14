//! PDF file writer - assembles all objects into a valid PDF document
//!
//! Enhanced with PrintCSS support including:
//! - @page rules with selectors and named pages
//! - Margin boxes with generated content
//! - Page contexts and counters
//! - PDF bookmarks/outline

use super::{
    object::{PdfDictionary, PdfObject, PdfReference, PdfArray},
    stream::PdfStream,
    PageContent,
    print_css::{
        PageContext, PageSize, PageMaster, MarginBoxContent, MarginContentPart,
        get_margin_box_rect, Bookmark, TextAlign, VerticalAlign,
    },
};
use crate::types::{PaperSize, Orientation, Margins, Result};
use std::collections::HashMap;
use std::io::{Write};

/// PDF document writer with PrintCSS support
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
    outline_ref: Option<PdfReference>,
    /// Page masters from @page rules
    page_masters: Vec<PageMaster>,
    /// Current page context
    page_context: PageContext,
    /// Bookmarks for PDF outline
    bookmarks: Vec<Bookmark>,
    /// Page content for margin boxes (per page)
    margin_box_content: Vec<HashMap<crate::css::MarginBoxType, MarginBoxContent>>,
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
            outline_ref: None,
            page_masters: Vec::new(),
            page_context: PageContext::new(1, 1),
            bookmarks: Vec::new(),
            margin_box_content: Vec::new(),
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
        self.page_context.size.width = self.page_width;
        self.page_context.size.height = self.page_height;
    }

    /// Set page margins
    pub fn set_margins(&mut self, margins: Margins) {
        self.margins = margins;
        self.page_context.margins = margins;
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

    /// Add a page master from @page rule
    pub fn add_page_master(&mut self, master: PageMaster) {
        self.page_masters.push(master);
    }

    /// Set the current page context
    pub fn set_page_context(&mut self, context: PageContext) {
        self.page_context = context;
        self.page_width = context.size.width;
        self.page_height = context.size.height;
        self.margins = context.margins;
    }

    /// Add a bookmark for PDF outline
    pub fn add_bookmark(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    /// Set margin box content for the current page
    pub fn set_margin_boxes(&mut self, boxes: HashMap<crate::css::MarginBoxType, MarginBoxContent>) {
        self.margin_box_content.push(boxes);
    }

    /// Apply applicable page masters to current context
    pub fn apply_page_masters(&mut self) {
        for master in &self.page_masters {
            if self.page_context.matches_selectors(&master.selectors) {
                master.apply_to(&mut self.page_context);
            }
        }
        // Update dimensions from context
        self.page_width = self.page_context.size.width;
        self.page_height = self.page_context.size.height;
        self.margins = self.page_context.margins;
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

    /// Generate content for margin boxes on the current page
    fn generate_margin_box_content(&self) -> Vec<(crate::css::MarginBoxType, MarginBoxContent)> {
        let mut result = Vec::new();
        
        // Get applicable page master margin boxes
        for master in &self.page_masters {
            if self.page_context.matches_selectors(&master.selectors) {
                for (box_type, content) in &master.margin_boxes {
                    result.push((*box_type, content.clone()));
                }
            }
        }
        
        // Override with explicit margin box content if set
        if let Some(page_boxes) = self.margin_box_content.get(self.page_context.page_number as usize - 1) {
            for (box_type, content) in page_boxes {
                // Remove any existing entry for this box type
                result.retain(|(bt, _)| bt != box_type);
                result.push((*box_type, content.clone()));
            }
        }
        
        result
    }

    /// Render margin box content to PDF content stream
    fn render_margin_boxes(&self, font_name: &str) -> Vec<u8> {
        use super::PageContent;
        
        let mut content = PageContent::new();
        let margin_boxes = self.generate_margin_box_content();
        
        if margin_boxes.is_empty() {
            return content.into_bytes();
        }
        
        let page_size = PageSize { width: self.page_width, height: self.page_height };
        
        for (box_type, box_content) in margin_boxes {
            let rect = get_margin_box_rect(box_type, &page_size, &self.margins);
            let text = box_content.generate_text(&self.page_context);
            
            if text.is_empty() {
                continue;
            }
            
            // Calculate text position based on alignment
            let x = match box_content.text_align {
                TextAlign::Left => rect.x + 4.0,
                TextAlign::Center => rect.x + rect.width / 2.0,
                TextAlign::Right => rect.x + rect.width - 4.0,
                TextAlign::Justify => rect.x + 4.0, // Left align for single line
            };
            
            let y = match box_content.vertical_align {
                VerticalAlign::Top => rect.y + rect.height - box_content.font_size,
                VerticalAlign::Middle => rect.y + rect.height / 2.0 - box_content.font_size / 2.0,
                VerticalAlign::Bottom => rect.y + box_content.font_size,
            };
            
            content.save_state();
            
            // Set color if specified
            if let Some(color) = box_content.color {
                content.set_fill_color(color);
            }
            
            content.begin_text();
            content.set_font(font_name, box_content.font_size);
            content.text_position(x, y);
            
            // For center/right align, we need to use TJ operator with positioning
            match box_content.text_align {
                TextAlign::Center => {
                    // Approximate center by using text positioning
                    content.text_position(x - text.len() as f32 * box_content.font_size * 0.25, y);
                    content.show_text(&text);
                }
                TextAlign::Right => {
                    content.text_position(x - text.len() as f32 * box_content.font_size * 0.5, y);
                    content.show_text(&text);
                }
                _ => content.show_text(&text),
            }
            
            content.end_text();
            content.restore_state();
        }
        
        content.into_bytes()
    }

    /// Add a page with content (optionally including margin boxes)
    pub fn add_page(&mut self, content: PageContent) -> PdfReference {
        self.add_page_with_margin_boxes(content, true)
    }

    /// Add a page with content and optional margin boxes
    pub fn add_page_with_margin_boxes(&mut self, content: PageContent, include_margin_boxes: bool) -> PdfReference {
        // Apply page masters
        self.apply_page_masters();
        
        // Combine main content with margin box content
        let mut final_content_bytes = content.into_bytes();
        
        if include_margin_boxes {
            // Get a font reference for margin boxes
            let font_name = self.fonts.keys().next()
                .cloned()
                .unwrap_or_else(|| "F1".to_string());
            
            let margin_bytes = self.render_margin_boxes(&font_name);
            final_content_bytes.extend_from_slice(&margin_bytes);
        }
        
        // Create content stream
        let mut stream_dict = PdfDictionary::new();
        stream_dict.insert("Length", final_content_bytes.len() as i32);
        
        let stream = PdfStream::with_dictionary(final_content_bytes, stream_dict);
        let content_ref = self.add_object(PdfObject::Dictionary(PdfDictionary::new()));
        
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
        
        // Store the actual content stream
        if let Some((_, obj)) = self.objects.iter_mut().find(|(r, _)| *r == content_ref) {
            *obj = PdfObject::Stream(stream.data);
        }
        
        // Increment page counter
        self.page_context.page_number += 1;
        
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

    /// Build the outline (bookmarks) structure
    fn build_outline(&mut self) -> Option<PdfReference> {
        if self.bookmarks.is_empty() {
            return None;
        }

        let mut outline_items = Vec::new();
        
        for bookmark in &self.bookmarks {
            let item_ref = self.create_outline_item(bookmark, None);
            outline_items.push(item_ref);
        }

        // Create outline root
        let mut outline_root = PdfDictionary::new();
        outline_root.insert("Type", PdfObject::Name("Outlines".to_string()));
        outline_root.insert("Count", self.bookmarks.len() as i32);
        
        if let Some(first) = outline_items.first() {
            outline_root.insert_ref("First", *first);
        }
        if let Some(last) = outline_items.last() {
            outline_root.insert_ref("Last", *last);
        }
        
        let outline_ref = self.add_object(PdfObject::Dictionary(outline_root));
        self.outline_ref = Some(outline_ref);
        
        Some(outline_ref)
    }

    /// Create a single outline item
    fn create_outline_item(&mut self, bookmark: &Bookmark, parent: Option<PdfReference>) -> PdfReference {
        let page_ref = self.pages.get(bookmark.page as usize - 1)
            .copied()
            .unwrap_or_else(|| self.pages_tree_ref.unwrap());
        
        let mut item = PdfDictionary::new();
        item.insert("Title", PdfObject::String(bookmark.title.as_bytes().to_vec()));
        item.insert_ref("Parent", parent.unwrap_or_else(|| self.outline_ref.unwrap_or(self.catalog_ref.unwrap())));
        item.insert_ref("Dest", page_ref);
        
        // Create destination array [page /Fit]
        let mut dest = PdfArray::new();
        dest.push(PdfObject::Reference(page_ref));
        dest.push(PdfObject::Name("Fit".to_string()));
        item.insert("DestArray", PdfObject::Array(dest));
        
        // Child count
        if !bookmark.children.is_empty() {
            item.insert("Count", bookmark.children.len() as i32);
        }
        
        self.add_object(PdfObject::Dictionary(item))
    }

    /// Write the PDF to output
    ///
    /// # Errors
    ///
    /// Returns `RenderError` if PDF generation fails.
    pub fn write<W: Write + std::io::Seek>(mut self, output: &mut W) -> Result<()> {
        // Build outline if bookmarks exist
        self.build_outline();
        
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

        // Update catalog with outline if present
        if let (Some(catalog_ref), Some(outline_ref)) = (self.catalog_ref, self.outline_ref) {
            if let Some((_, obj)) = self.objects.iter_mut().find(|(r, _)| *r == catalog_ref) {
                if let PdfObject::Dictionary(dict) = obj {
                    dict.insert_ref("Outlines", outline_ref);
                    dict.insert("PageMode", PdfObject::Name("UseOutlines".to_string()));
                }
            }
        }

        // Build cross-reference table
        let mut xref_offsets: Vec<(u32, u32)> = Vec::new();
        
        // PDF header
        output.write_all(b"%PDF-1.4\n")
            .map_err(|e| errors::render(format!("Failed to write PDF header: {}", e)))?;
        output.write_all(b"%\xE2\xE3\xCF\xD3\n") // Binary marker
            .map_err(|e| errors::render(format!("Failed to write PDF binary marker: {}", e)))?;
        
        // Write objects
        for (reference, object) in &self.objects {
            let offset = output.stream_position()
                .map_err(|e| errors::render(format!("Failed to get stream position: {}", e)))? as u32;
            xref_offsets.push((reference.object_number, offset));
            
            writeln!(output, "{} {} obj", reference.object_number, reference.generation)
                .map_err(|e| errors::render(format!("Failed to write object header: {}", e)))?;
            
            // Handle streams specially
            match object {
                PdfObject::Stream(data) => {
                    // Write stream dictionary and data
                    let mut dict_str = String::new();
                    let mut dict = PdfDictionary::new();
                    dict.insert("Length", data.len() as i32);
                    dict.write(&mut dict_str).unwrap();
                    
                    write!(output, "{}\nstream\n", dict_str)
                        .map_err(|e| errors::render(format!("Failed to write stream header: {}", e)))?;
                    output.write_all(data)
                        .map_err(|e| errors::render(format!("Failed to write stream data: {}", e)))?;
                    output.write_all(b"\nendstream\n")
                        .map_err(|e| errors::render(format!("Failed to write stream end: {}", e)))?;
                }
                _ => {
                    let mut obj_str = String::new();
                    object.write(&mut obj_str).unwrap();
                    writeln!(output, "{}", obj_str)
                        .map_err(|e| errors::render(format!("Failed to write object: {}", e)))?;
                }
            }
            
            writeln!(output, "endobj")
                .map_err(|e| errors::render(format!("Failed to write endobj: {}", e)))?;
        }
        
        // Cross-reference table
        let xref_offset = output.stream_position()
            .map_err(|e| errors::render(format!("Failed to get xref position: {}", e)))?;
        writeln!(output, "xref")
            .map_err(|e| errors::render(format!("Failed to write xref header: {}", e)))?;
        writeln!(output, "0 {}", self.next_object_number)
            .map_err(|e| errors::render(format!("Failed to write xref size: {}", e)))?;
        writeln!(output, "{:010} {:05} f ", 0, 65535)
            .map_err(|e| errors::render(format!("Failed to write xref free entry: {}", e)))?;
        
        // Sort by object number
        let mut sorted_offsets: Vec<(u32, u32)> = xref_offsets;
        sorted_offsets.sort_by_key(|(num, _)| *num);
        
        for (_, offset) in sorted_offsets {
            writeln!(output, "{:010} {:05} n ", offset, 0)
                .map_err(|e| errors::render(format!("Failed to write xref entry: {}", e)))?;
        }
        
        // Trailer
        writeln!(output, "trailer")
            .map_err(|e| errors::render(format!("Failed to write trailer header: {}", e)))?;
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
        writeln!(output, "{}", trailer_str)
            .map_err(|e| errors::render(format!("Failed to write trailer: {}", e)))?;
        
        writeln!(output, "startxref")
            .map_err(|e| errors::render(format!("Failed to write startxref: {}", e)))?;
        writeln!(output, "{}", xref_offset)
            .map_err(|e| errors::render(format!("Failed to write xref offset: {}", e)))?;
        writeln!(output, "%%EOF")
            .map_err(|e| errors::render(format!("Failed to write EOF marker: {}", e)))?;
        
        Ok(())
    }

    /// Get current page context
    pub fn page_context(&self) -> &PageContext {
        &self.page_context
    }

    /// Get mutable page context
    pub fn page_context_mut(&mut self) -> &mut PageContext {
        &mut self.page_context
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

    #[test]
    fn test_page_context() {
        let mut writer = PdfWriter::new();
        writer.init_document();
        writer.add_standard_font("F1", "Helvetica");
        
        // Set up page context
        let mut context = PageContext::new(1, 1);
        context.size.width = 612.0;
        context.size.height = 792.0;
        writer.set_page_context(context);
        
        assert_eq!(writer.page_width, 612.0);
        assert_eq!(writer.page_height, 792.0);
    }

    #[test]
    fn test_margin_boxes() {
        use crate::css::at_rules::{PageRule, PageMarginBox, MarginBoxType};
        use crate::css::parser::Declaration;
        use crate::css::values::CssValue;
        use crate::pdf::print_css::PageMaster;
        
        let mut writer = PdfWriter::new();
        writer.init_document();
        writer.add_standard_font("F1", "Helvetica");
        
        // Create a page master with margin boxes
        let mut rule = PageRule::new();
        let mut margin_box = PageMarginBox::new(MarginBoxType::TopCenter);
        margin_box.add_declaration(Declaration::new(
            "content",
            CssValue::String("Test Header".to_string())
        ));
        rule.add_margin_box(margin_box);
        
        let master = PageMaster::from_page_rule(&rule);
        writer.add_page_master(master);
        
        // Create content
        let content = PageContent::new();
        writer.add_page(content);
        
        let mut output = std::io::Cursor::new(Vec::new());
        writer.write(&mut output).unwrap();
        let bytes = output.into_inner();
        
        assert!(bytes.starts_with(b"%PDF"));
    }
}

//! CSS Box Model
//!
//! Implements the CSS box model with content, padding, border, and margin areas.
//! Handles box tree construction from DOM and box dimensions calculations.

use crate::html::{Element, Node, TextNode};
use crate::layout::form::{FormBox, is_form_element, create_form_box};
use crate::types::{Rect, Length};

/// Type of layout box
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxType {
    /// Block-level box
    #[default]
    Block,
    /// Inline-level box
    Inline,
    /// Inline-block box
    InlineBlock,
    /// Anonymous block box (for mixed content)
    Anonymous,
    /// Text run
    TextRun,
    /// Image box
    Image,
    /// Flex container
    Flex,
    /// Grid container
    Grid,
    /// Form control box
    Form,
    /// Table container
    Table,
    /// Table row
    TableRow,
    /// Table cell
    TableCell,
}

impl BoxType {
    /// Check if this is a block-level box type
    pub fn is_block_level(&self) -> bool {
        matches!(self, BoxType::Block | BoxType::Flex | BoxType::Grid | BoxType::Table)
    }

    /// Check if this is an inline-level box type
    pub fn is_inline_level(&self) -> bool {
        matches!(self, BoxType::Inline | BoxType::InlineBlock | BoxType::TextRun | BoxType::Image | BoxType::Form)
    }

    /// Check if this establishes a block formatting context
    pub fn establishes_bfc(&self) -> bool {
        matches!(self, BoxType::Block | BoxType::Flex | BoxType::Grid | BoxType::Anonymous | BoxType::Table)
    }

    /// Check if this is a table-related box type
    pub fn is_table_type(&self) -> bool {
        matches!(self, BoxType::Table | BoxType::TableRow | BoxType::TableCell)
    }

    /// Check if this is a form control box
    pub fn is_form(&self) -> bool {
        matches!(self, BoxType::Form)
    }
}

/// Edge sizes (margin, border, padding)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeSizes {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeSizes {
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Get total horizontal edges
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical edges
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Dimensions of a box (content + padding + border + margin)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dimensions {
    /// Content box position and size
    pub content: Rect,
    /// Padding area
    pub padding: EdgeSizes,
    /// Border area
    pub border: EdgeSizes,
    /// Margin area
    pub margin: EdgeSizes,
}

impl Dimensions {
    pub fn new(content: Rect) -> Self {
        Self {
            content,
            padding: EdgeSizes::default(),
            border: EdgeSizes::default(),
            margin: EdgeSizes::default(),
        }
    }

    /// Get the padding box rectangle
    pub fn padding_box(&self) -> Rect {
        Rect::new(
            self.content.x - self.padding.left,
            self.content.y - self.padding.top,
            self.content.width + self.padding.horizontal(),
            self.content.height + self.padding.vertical(),
        )
    }

    /// Get the border box rectangle
    pub fn border_box(&self) -> Rect {
        let padding = self.padding_box();
        Rect::new(
            padding.x - self.border.left,
            padding.y - self.border.top,
            padding.width + self.border.horizontal(),
            padding.height + self.border.vertical(),
        )
    }

    /// Get the margin box rectangle
    pub fn margin_box(&self) -> Rect {
        let border = self.border_box();
        Rect::new(
            border.x - self.margin.left,
            border.y - self.margin.top,
            border.width + self.margin.horizontal(),
            border.height + self.margin.vertical(),
        )
    }

    /// Get the total width including padding and border
    pub fn border_box_width(&self) -> f32 {
        self.content.width + self.padding.horizontal() + self.border.horizontal()
    }

    /// Get the total height including padding and border
    pub fn border_box_height(&self) -> f32 {
        self.content.height + self.padding.vertical() + self.border.vertical()
    }

    /// Get the total width including margins
    pub fn margin_box_width(&self) -> f32 {
        self.border_box_width() + self.margin.horizontal()
    }

    /// Get the total height including margins
    pub fn margin_box_height(&self) -> f32 {
        self.border_box_height() + self.margin.vertical()
    }
}

/// Image intrinsic dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IntrinsicSize {
    /// Natural width of the image
    pub width: Option<f32>,
    /// Natural height of the image
    pub height: Option<f32>,
    /// Aspect ratio (width / height)
    pub aspect_ratio: Option<f32>,
}

impl IntrinsicSize {
    /// Create from explicit dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            aspect_ratio: Some(width / height),
        }
    }

    /// Create with only aspect ratio (e.g., from SVG viewBox)
    pub fn with_aspect_ratio(ratio: f32) -> Self {
        Self {
            width: None,
            height: None,
            aspect_ratio: Some(ratio),
        }
    }
}

/// Object-fit property values
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ObjectFit {
    /// Fill the box, may distort aspect ratio
    Fill,
    /// Preserve aspect ratio, may be cropped
    #[default]
    Cover,
    /// Preserve aspect ratio, may have empty space
    Contain,
    /// No resizing
    None,
    /// Like 'contain' but if smaller than box, not scaled up
    ScaleDown,
}

impl ObjectFit {
    /// Parse from CSS string value
    pub fn from_css(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "fill" => ObjectFit::Fill,
            "cover" => ObjectFit::Cover,
            "contain" => ObjectFit::Contain,
            "none" => ObjectFit::None,
            "scale-down" | "scaledown" => ObjectFit::ScaleDown,
            _ => ObjectFit::Cover,
        }
    }
}

/// Object-position property (horizontal, vertical)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ObjectPosition {
    pub horizontal: f32, // 0.0 = left, 1.0 = right
    pub vertical: f32,   // 0.0 = top, 1.0 = bottom
}

impl ObjectPosition {
    /// Default centered position
    pub const CENTER: Self = Self { horizontal: 0.5, vertical: 0.5 };

    /// Parse from CSS value like "center" or "50% 75%"
    pub fn from_css(value: &str) -> Self {
        let parts: Vec<&str> = value.split_whitespace().collect();
        
        let h = parse_position_value(parts.get(0).unwrap_or(&"center"));
        let v = parse_position_value(parts.get(1).unwrap_or(&"center"));
        
        Self { horizontal: h, vertical: v }
    }

    /// Calculate actual position given container and content sizes
    pub fn calculate_position(&self, container_size: f32, content_size: f32) -> f32 {
        // Position is percentage of remaining space
        let remaining = (container_size - content_size).max(0.0);
        -(remaining * self.horizontal)
    }
}

fn parse_position_value(s: &str) -> f32 {
    let s = s.trim().to_ascii_lowercase();
    match s.as_str() {
        "left" | "top" => 0.0,
        "center" => 0.5,
        "right" | "bottom" => 1.0,
        _ => {
            // Parse percentage
            if let Some(num) = s.strip_suffix('%') {
                num.parse::<f32>().unwrap_or(50.0) / 100.0
            } else {
                0.5 // Default to center
            }
        }
    }
}

/// Image-specific box data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImageBox {
    /// Image source URL or data URI
    pub src: String,
    /// Alternative text
    pub alt: String,
    /// Intrinsic (natural) dimensions of the image
    pub intrinsic_size: IntrinsicSize,
    /// Specified width (from HTML attribute or CSS)
    pub specified_width: Option<Length>,
    /// Specified height (from HTML attribute or CSS)
    pub specified_height: Option<Length>,
    /// Object-fit property
    pub object_fit: ObjectFit,
    /// Object-position property
    pub object_position: ObjectPosition,
    /// Whether the image is loaded and ready
    pub is_loaded: bool,
}

impl ImageBox {
    /// Create a new image box
    pub fn new(src: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: String::new(),
            intrinsic_size: IntrinsicSize::default(),
            specified_width: None,
            specified_height: None,
            object_fit: ObjectFit::default(),
            object_position: ObjectPosition::CENTER,
            is_loaded: false,
        }
    }

    /// Set alt text
    pub fn with_alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
        self
    }

    /// Set intrinsic size
    pub fn with_intrinsic_size(mut self, width: f32, height: f32) -> Self {
        self.intrinsic_size = IntrinsicSize::new(width, height);
        self.is_loaded = true;
        self
    }

    /// Calculate the concrete image dimensions based on:
    /// - Intrinsic size
    /// - Specified width/height
    /// - Object-fit
    /// - Available space
    pub fn calculate_concrete_size(&self, available_width: f32, available_height: Option<f32>, base_font_size: f32) -> (f32, f32) {
        // Get specified dimensions
        let specified_width_pt = self.specified_width
            .map(|l| if l.is_auto() { None } else { Some(l.to_pt(base_font_size)) })
            .flatten();
        
        let specified_height_pt = self.specified_height
            .map(|l| if l.is_auto() { None } else { Some(l.to_pt(base_font_size)) })
            .flatten();

        // Get intrinsic dimensions
        let intrinsic_width = self.intrinsic_size.width.unwrap_or(300.0);
        let intrinsic_height = self.intrinsic_size.height.unwrap_or(150.0);
        let aspect_ratio = self.intrinsic_size.aspect_ratio
            .unwrap_or(intrinsic_width / intrinsic_height);

        // Calculate used dimensions
        let (used_width, used_height) = match (specified_width_pt, specified_height_pt) {
            // Both dimensions specified
            (Some(w), Some(h)) => (w, h),
            // Only width specified
            (Some(w), None) => {
                let h = if self.object_fit == ObjectFit::Fill {
                    // Fill uses available height or intrinsic
                    available_height.unwrap_or(intrinsic_height)
                } else {
                    // Preserve aspect ratio
                    w / aspect_ratio
                };
                (w, h)
            }
            // Only height specified
            (None, Some(h)) => {
                let w = if self.object_fit == ObjectFit::Fill {
                    // Fill uses available width
                    available_width
                } else {
                    // Preserve aspect ratio
                    h * aspect_ratio
                };
                (w, h)
            }
            // Neither specified - use intrinsic or fit to container
            (None, None) => {
                // Scale down if needed
                let mut w = intrinsic_width;
                let mut h = intrinsic_height;
                
                // If intrinsic width exceeds available width, scale down
                if w > available_width {
                    let scale = available_width / w;
                    w = available_width;
                    h = if self.object_fit == ObjectFit::Fill {
                        h // Don't scale height in fill mode
                    } else {
                        h * scale
                    };
                }
                
                // Also check height constraint
                if let Some(max_h) = available_height {
                    if h > max_h {
                        let scale = max_h / h;
                        h = max_h;
                        if self.object_fit != ObjectFit::Fill {
                            w = w * scale;
                        }
                    }
                }
                
                (w, h)
            }
        };

        // Apply scale-down constraint
        if self.object_fit == ObjectFit::ScaleDown {
            let max_width = self.intrinsic_size.width.unwrap_or(used_width);
            let max_height = self.intrinsic_size.height.unwrap_or(used_height);
            
            if used_width > max_width || used_height > max_height {
                let scale_w = max_width / used_width;
                let scale_h = max_height / used_height;
                let scale = scale_w.min(scale_h);
                return (used_width * scale, used_height * scale);
            }
        }

        (used_width, used_height)
    }

    /// Calculate the actual drawing rectangle for the image content
    /// based on object-fit and object-position
    pub fn calculate_draw_rect(&self, container_rect: Rect, image_width: f32, image_height: f32) -> Rect {
        let container_aspect = container_rect.width / container_rect.height.max(0.01);
        let image_aspect = image_width / image_height.max(0.01);

        let (draw_width, draw_height, draw_x, draw_y) = match self.object_fit {
            ObjectFit::Fill => {
                (container_rect.width, container_rect.height, container_rect.x, container_rect.y)
            }
            ObjectFit::Contain => {
                if image_aspect > container_aspect {
                    // Image is wider - fit to width
                    let h = container_rect.width / image_aspect;
                    let y = container_rect.y + (container_rect.height - h) * self.object_position.vertical;
                    (container_rect.width, h, container_rect.x, y)
                } else {
                    // Image is taller - fit to height
                    let w = container_rect.height * image_aspect;
                    let x = container_rect.x + (container_rect.width - w) * self.object_position.horizontal;
                    (w, container_rect.height, x, container_rect.y)
                }
            }
            ObjectFit::Cover => {
                if image_aspect > container_aspect {
                    // Image is wider - cover height, overflow width
                    let h = container_rect.height;
                    let w = h * image_aspect;
                    let x = container_rect.x + (container_rect.width - w) * self.object_position.horizontal;
                    (w, h, x, container_rect.y)
                } else {
                    // Image is taller - cover width, overflow height
                    let w = container_rect.width;
                    let h = w / image_aspect;
                    let y = container_rect.y + (container_rect.height - h) * self.object_position.vertical;
                    (w, h, container_rect.x, y)
                }
            }
            ObjectFit::None => {
                // No scaling - use original size (clipped to container)
                let x = container_rect.x + (container_rect.width - image_width) * self.object_position.horizontal;
                let y = container_rect.y + (container_rect.height - image_height) * self.object_position.vertical;
                (image_width, image_height, x, y)
            }
            ObjectFit::ScaleDown => {
                // Like contain but don't scale up
                if image_width <= container_rect.width && image_height <= container_rect.height {
                    // Image is smaller - no scaling (like none)
                    let x = container_rect.x + (container_rect.width - image_width) * self.object_position.horizontal;
                    let y = container_rect.y + (container_rect.height - image_height) * self.object_position.vertical;
                    (image_width, image_height, x, y)
                } else {
                    // Scale down like contain
                    if image_aspect > container_aspect {
                        let h = container_rect.width / image_aspect;
                        let y = container_rect.y + (container_rect.height - h) * self.object_position.vertical;
                        (container_rect.width, h, container_rect.x, y)
                    } else {
                        let w = container_rect.height * image_aspect;
                        let x = container_rect.x + (container_rect.width - w) * self.object_position.horizontal;
                        (w, container_rect.height, x, container_rect.y)
                    }
                }
            }
        };

        Rect::new(draw_x, draw_y, draw_width, draw_height)
    }
}

/// A layout box representing a node in the box tree
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutBox {
    /// Type of box
    pub box_type: BoxType,
    /// Associated DOM node (None for anonymous boxes)
    pub node: Option<Node>,
    /// Computed dimensions
    pub dimensions: Dimensions,
    /// Child boxes
    pub children: Vec<LayoutBox>,
    /// Text content (for text runs)
    pub text_content: Option<String>,
    /// Image data (for image boxes)
    pub image_data: Option<ImageBox>,
    /// Form data (for form control boxes)
    pub form_data: Option<FormBox>,
    /// Whether this box has been laid out
    pub is_laid_out: bool,
}

impl LayoutBox {
    pub fn new(box_type: BoxType, node: Option<Node>) -> Self {
        Self {
            box_type,
            node,
            dimensions: Dimensions::default(),
            children: Vec::new(),
            text_content: None,
            image_data: None,
            form_data: None,
            is_laid_out: false,
        }
    }

    /// Create a block box for an element
    pub fn block_box(element: &Element) -> Self {
        Self::new(BoxType::Block, Some(Node::Element(element.clone())))
    }

    /// Create an inline box for an element
    pub fn inline_box(element: &Element) -> Self {
        Self::new(BoxType::Inline, Some(Node::Element(element.clone())))
    }

    /// Create an anonymous block box
    pub fn anonymous_box() -> Self {
        Self::new(BoxType::Anonymous, None)
    }

    /// Create a text run box
    pub fn text_box(text: &TextNode) -> Self {
        let mut box_ = Self::new(BoxType::TextRun, Some(Node::Text(text.clone())));
        box_.text_content = Some(text.data.clone());
        box_
    }

    /// Create an image box
    pub fn image_box(element: &Element, src: impl Into<String>) -> Self {
        let mut box_ = Self::new(BoxType::Image, Some(Node::Element(element.clone())));
        box_.image_data = Some(ImageBox::new(src));
        box_
    }

    /// Create a form control box
    pub fn form_box(element: &Element, form_data: FormBox) -> Self {
        let mut box_ = Self::new(BoxType::Form, Some(Node::Element(element.clone())));
        box_.form_data = Some(form_data);
        box_
    }

    /// Append a child box
    pub fn append_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }

    /// Get the content width
    pub fn content_width(&self) -> f32 {
        self.dimensions.content.width
    }

    /// Get the content height
    pub fn content_height(&self) -> f32 {
        self.dimensions.content.height
    }

    /// Set content width
    pub fn set_content_width(&mut self, width: f32) {
        self.dimensions.content.width = width;
    }

    /// Set content height
    pub fn set_content_height(&mut self, height: f32) {
        self.dimensions.content.height = height;
    }

    /// Set content position
    pub fn set_content_position(&mut self, x: f32, y: f32) {
        self.dimensions.content.x = x;
        self.dimensions.content.y = y;
    }

    /// Check if this box contains a block-level child
    pub fn has_block_child(&self) -> bool {
        self.children.iter().any(|c| c.box_type.is_block_level())
    }

    /// Get the element if this box represents an element
    pub fn element(&self) -> Option<&Element> {
        self.node.as_ref()?.as_element()
    }

    /// Get the tag name if this box represents an element
    pub fn tag_name(&self) -> Option<&str> {
        Some(self.element()?.tag_name())
    }

    /// Get image data if this is an image box
    pub fn image_data(&self) -> Option<&ImageBox> {
        self.image_data.as_ref()
    }

    /// Get mutable image data if this is an image box
    pub fn image_data_mut(&mut self) -> Option<&mut ImageBox> {
        self.image_data.as_mut()
    }

    /// Get form data if this is a form box
    pub fn form_data(&self) -> Option<&FormBox> {
        self.form_data.as_ref()
    }

    /// Get mutable form data if this is a form box
    pub fn form_data_mut(&mut self) -> Option<&mut FormBox> {
        self.form_data.as_mut()
    }
}

/// Build a box tree from a DOM element
pub fn build_box_tree(element: &Element, display_resolver: &dyn Fn(&Element) -> BoxType) -> LayoutBox {
    let box_type = display_resolver(element);
    let mut box_ = LayoutBox::new(box_type, Some(Node::Element(element.clone())));

    // Special handling for img elements
    if element.tag_name().eq_ignore_ascii_case("img") {
        let src = element.attr("src").unwrap_or("").to_string();
        let mut img_box = LayoutBox::image_box(element, src);
        
        // Parse width/height attributes
        if let Some(width_attr) = element.attr("width") {
            if let Ok(w) = width_attr.parse::<f32>() {
                if let Some(img_data) = img_box.image_data_mut() {
                    img_data.specified_width = Some(Length::Px(w));
                }
            }
        }
        if let Some(height_attr) = element.attr("height") {
            if let Ok(h) = height_attr.parse::<f32>() {
                if let Some(img_data) = img_box.image_data_mut() {
                    img_data.specified_height = Some(Length::Px(h));
                }
            }
        }
        
        return img_box;
    }

    // Special handling for form elements
    if is_form_element(element.tag_name()) {
        if let Some(form_data) = create_form_box(element) {
            // Skip hidden inputs entirely
            if form_data.control_type.is_visible() {
                return LayoutBox::form_box(element, form_data);
            } else {
                // Return an anonymous box for hidden inputs (won't be rendered)
                return LayoutBox::anonymous_box();
            }
        }
    }

    match box_type {
        BoxType::Block | BoxType::Flex | BoxType::Grid => {
            build_block_children(element, &mut box_, display_resolver);
        }
        BoxType::Inline => {
            build_inline_children(element, &mut box_, display_resolver);
        }
        BoxType::InlineBlock => {
            // Inline-block contains both inline and block content
            build_inline_children(element, &mut box_, display_resolver);
        }
        BoxType::Table => {
            // Table elements build their own structure
            build_table_children(element, &mut box_, display_resolver);
        }
        BoxType::TableRow => {
            build_table_row_children(element, &mut box_, display_resolver);
        }
        BoxType::TableCell => {
            build_table_cell_children(element, &mut box_, display_resolver);
        }
        _ => {}
    }

    box_
}

/// Build children for a block container
fn build_block_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    let mut inline_buffer: Vec<Node> = Vec::new();

    for child in element.children() {
        match child {
            Node::Element(child_el) => {
                // Special handling for img elements (they're inline by default)
                if child_el.tag_name().eq_ignore_ascii_case("img") {
                    // Flush any buffered inline content
                    if !inline_buffer.is_empty() {
                        let anon = build_anonymous_box(&inline_buffer, display_resolver);
                        parent_box.append_child(anon);
                        inline_buffer.clear();
                    }
                    // Add image box
                    let src = child_el.attr("src").unwrap_or("").to_string();
                    let mut img_box = LayoutBox::image_box(child_el, src);
                    
                    if let Some(width_attr) = child_el.attr("width") {
                        if let Ok(w) = width_attr.parse::<f32>() {
                            if let Some(img_data) = img_box.image_data_mut() {
                                img_data.specified_width = Some(Length::Px(w));
                            }
                        }
                    }
                    if let Some(height_attr) = child_el.attr("height") {
                        if let Ok(h) = height_attr.parse::<f32>() {
                            if let Some(img_data) = img_box.image_data_mut() {
                                img_data.specified_height = Some(Length::Px(h));
                            }
                        }
                    }
                    
                    parent_box.append_child(img_box);
                    continue;
                }

                // Special handling for form elements
                if is_form_element(child_el.tag_name()) {
                    // Flush any buffered inline content
                    if !inline_buffer.is_empty() {
                        let anon = build_anonymous_box(&inline_buffer, display_resolver);
                        parent_box.append_child(anon);
                        inline_buffer.clear();
                    }
                    // Add form box
                    if let Some(form_data) = create_form_box(child_el) {
                        if form_data.control_type.is_visible() {
                            let form_box = LayoutBox::form_box(child_el, form_data);
                            parent_box.append_child(form_box);
                        }
                    }
                    continue;
                }

                let child_display = display_resolver(child_el);

                if child_display.is_block_level() {
                    // Flush any buffered inline content as anonymous block
                    if !inline_buffer.is_empty() {
                        let anon = build_anonymous_box(&inline_buffer, display_resolver);
                        parent_box.append_child(anon);
                        inline_buffer.clear();
                    }
                    // Add block child
                    let child_box = build_box_tree(child_el, display_resolver);
                    parent_box.append_child(child_box);
                } else {
                    // Collect inline content
                    inline_buffer.push(child.clone());
                }
            }
            Node::Text(text) => {
                // Only add non-whitespace text or if we're in an inline context
                if !text.data.trim().is_empty() {
                    inline_buffer.push(child.clone());
                }
            }
            _ => {}
        }
    }

    // Flush remaining inline content
    if !inline_buffer.is_empty() {
        let anon = build_anonymous_box(&inline_buffer, display_resolver);
        parent_box.append_child(anon);
    }
}

/// Build children for an inline container
fn build_inline_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    for child in element.children() {
        match child {
            Node::Element(child_el) => {
                // Special handling for img elements
                if child_el.tag_name().eq_ignore_ascii_case("img") {
                    let src = child_el.attr("src").unwrap_or("").to_string();
                    let mut img_box = LayoutBox::image_box(child_el, src);
                    
                    if let Some(width_attr) = child_el.attr("width") {
                        if let Ok(w) = width_attr.parse::<f32>() {
                            if let Some(img_data) = img_box.image_data_mut() {
                                img_data.specified_width = Some(Length::Px(w));
                            }
                        }
                    }
                    if let Some(height_attr) = child_el.attr("height") {
                        if let Ok(h) = height_attr.parse::<f32>() {
                            if let Some(img_data) = img_box.image_data_mut() {
                                img_data.specified_height = Some(Length::Px(h));
                            }
                        }
                    }
                    
                    parent_box.append_child(img_box);
                    continue;
                }

                // Special handling for form elements
                if is_form_element(child_el.tag_name()) {
                    if let Some(form_data) = create_form_box(child_el) {
                        if form_data.control_type.is_visible() {
                            let form_box = LayoutBox::form_box(child_el, form_data);
                            parent_box.append_child(form_box);
                        }
                    }
                    continue;
                }

                let child_box = build_box_tree(child_el, display_resolver);
                parent_box.append_child(child_box);
            }
            Node::Text(text) => {
                if !text.data.is_empty() {
                    let text_box = LayoutBox::text_box(text);
                    parent_box.append_child(text_box);
                }
            }
            _ => {}
        }
    }
}

/// Build an anonymous block box from inline content
fn build_anonymous_box(
    nodes: &[Node],
    display_resolver: &dyn Fn(&Element) -> BoxType,
) -> LayoutBox {
    let mut anon = LayoutBox::anonymous_box();

    for node in nodes {
        match node {
            Node::Element(el) => {
                let child_box = build_box_tree(el, display_resolver);
                anon.append_child(child_box);
            }
            Node::Text(text) => {
                let text_box = LayoutBox::text_box(text);
                anon.append_child(text_box);
            }
            _ => {}
        }
    }

    anon
}

/// Build children for a table element
fn build_table_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    for child in element.children() {
        if let Node::Element(child_el) = child {
            let tag_name = child_el.tag_name().to_ascii_lowercase();
            
            match tag_name.as_str() {
                "caption" | "colgroup" | "col" | "thead" | "tbody" | "tfoot" | "tr" => {
                    let child_box = build_box_tree(child_el, display_resolver);
                    parent_box.append_child(child_box);
                }
                _ => {
                    // Other elements are treated as block children
                    let child_box = build_box_tree(child_el, display_resolver);
                    parent_box.append_child(child_box);
                }
            }
        }
    }
}

/// Build children for a table row element
fn build_table_row_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    for child in element.children() {
        if let Node::Element(child_el) = child {
            let tag_name = child_el.tag_name().to_ascii_lowercase();
            
            if tag_name == "td" || tag_name == "th" {
                let child_box = build_box_tree(child_el, display_resolver);
                parent_box.append_child(child_box);
            }
        }
    }
}

/// Build children for a table cell element
fn build_table_cell_children(
    element: &Element,
    parent_box: &mut LayoutBox,
    display_resolver: &dyn Fn(&Element) -> BoxType,
) {
    // Table cells contain normal flow content
    build_block_children(element, parent_box, display_resolver);
}

/// Calculate width based on containing block and computed values
pub fn calculate_width(
    box_: &mut LayoutBox,
    containing_block_width: f32,
    specified_width: Option<Length>,
    margins: (Length, Length),
    padding: (Length, Length, Length, Length),
    borders: (Length, Length, Length, Length),
    base_font_size: f32,
) {
    let dims = &mut box_.dimensions;

    // Convert padding
    dims.padding.left = padding.0.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.right = padding.1.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.top = padding.2.to_pt_with_container(base_font_size, containing_block_width);
    dims.padding.bottom = padding.3.to_pt_with_container(base_font_size, containing_block_width);

    // Convert borders
    dims.border.left = borders.0.to_pt(base_font_size);
    dims.border.right = borders.1.to_pt(base_font_size);
    dims.border.top = borders.2.to_pt(base_font_size);
    dims.border.bottom = borders.3.to_pt(base_font_size);

    // Calculate available width
    let padding_border_width = dims.padding.horizontal() + dims.border.horizontal();

    // Handle width calculation
    let width = if let Some(w) = specified_width {
        if w.is_auto() {
            // Width depends on margins and containing block
            let margin_left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
            let margin_right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
            
            // For auto width: width = containing_block - margins - padding - border
            let available = containing_block_width - margin_left - margin_right - padding_border_width;
            
            dims.margin.left = margin_left;
            dims.margin.right = margin_right;
            available.max(0.0)
        } else {
            // Fixed width
            dims.margin.left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
            dims.margin.right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
            w.to_pt_with_container(base_font_size, containing_block_width)
        }
    } else {
        // Default to auto
        let margin_left = margins.0.to_pt_with_container(base_font_size, containing_block_width);
        let margin_right = margins.1.to_pt_with_container(base_font_size, containing_block_width);
        let available = containing_block_width - margin_left - margin_right - padding_border_width;
        
        dims.margin.left = margin_left;
        dims.margin.right = margin_right;
        available.max(0.0)
    };

    dims.content.width = width.max(0.0);
}

/// Calculate height based on content or specified value
#[allow(clippy::too_many_arguments)]
pub fn calculate_height(
    box_: &mut LayoutBox,
    containing_block_height: f32,
    specified_height: Option<Length>,
    margins: (Length, Length),
    padding: (Length, Length, Length, Length),
    borders: (Length, Length, Length, Length),
    base_font_size: f32,
    content_height: Option<f32>,
) {
    let dims = &mut box_.dimensions;

    // Convert vertical padding
    dims.padding.top = padding.0.to_pt_with_container(base_font_size, containing_block_height);
    dims.padding.bottom = padding.1.to_pt_with_container(base_font_size, containing_block_height);

    // Convert vertical borders
    dims.border.top = borders.0.to_pt(base_font_size);
    dims.border.bottom = borders.1.to_pt(base_font_size);

    // Convert margins
    dims.margin.top = margins.0.to_pt_with_container(base_font_size, containing_block_height);
    dims.margin.bottom = margins.1.to_pt_with_container(base_font_size, containing_block_height);

    // Calculate height
    let height = if let Some(h) = specified_height {
        if h.is_auto() {
            // Height depends on content
            content_height.unwrap_or(0.0)
        } else {
            h.to_pt_with_container(base_font_size, containing_block_height)
        }
    } else {
        // Default to content height
        content_height.unwrap_or(0.0)
    };

    dims.content.height = height.max(0.0);
}

/// Calculate position based on normal flow
pub fn calculate_position(
    box_: &mut LayoutBox,
    containing_block: &Rect,
    x: f32,
    y: f32,
) {
    box_.dimensions.content.x = containing_block.x + x;
    box_.dimensions.content.y = containing_block.y + y;
}

/// Shrink-to-fit width calculation
#[allow(dead_code)]
pub fn shrink_to_fit_width(box_: &LayoutBox, available_width: f32) -> f32 {
    // For block boxes with auto width in certain contexts
    // Returns the preferred minimum width based on content
    let content_width = box_.children.iter()
        .map(|child| child.dimensions.margin_box_width())
        .fold(0.0, f32::max);
    
    content_width.min(available_width).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_sizes() {
        let edges = EdgeSizes::all(10.0);
        assert_eq!(edges.horizontal(), 20.0);
        assert_eq!(edges.vertical(), 20.0);

        let edges = EdgeSizes::symmetric(10.0, 20.0);
        assert_eq!(edges.horizontal(), 40.0);
        assert_eq!(edges.vertical(), 20.0);
    }

    #[test]
    fn test_dimensions() {
        let mut dims = Dimensions::new(Rect::new(0.0, 0.0, 100.0, 50.0));
        dims.padding = EdgeSizes::all(10.0);
        dims.border = EdgeSizes::all(5.0);
        dims.margin = EdgeSizes::all(15.0);

        // Content: 100x50
        assert_eq!(dims.content.width, 100.0);
        assert_eq!(dims.content.height, 50.0);

        // Padding box: (100 + 20) x (50 + 20) = 120 x 70
        let padding = dims.padding_box();
        assert_eq!(padding.width, 120.0);
        assert_eq!(padding.height, 70.0);

        // Border box: (120 + 10) x (70 + 10) = 130 x 80
        let border = dims.border_box();
        assert_eq!(border.width, 130.0);
        assert_eq!(border.height, 80.0);

        // Margin box: (130 + 30) x (80 + 30) = 160 x 110
        let margin = dims.margin_box();
        assert_eq!(margin.width, 160.0);
        assert_eq!(margin.height, 110.0);
    }

    #[test]
    fn test_box_type() {
        assert!(BoxType::Block.is_block_level());
        assert!(!BoxType::Block.is_inline_level());
        
        assert!(BoxType::Inline.is_inline_level());
        assert!(!BoxType::Inline.is_block_level());
        
        assert!(BoxType::Block.establishes_bfc());
        assert!(BoxType::Anonymous.establishes_bfc());
    }

    #[test]
    fn test_layout_box_creation() {
        let el = Element::new("div", vec![]);
        let box_ = LayoutBox::block_box(&el);
        
        assert_eq!(box_.box_type, BoxType::Block);
        assert!(box_.element().is_some());
        assert_eq!(box_.tag_name(), Some("div"));
    }

    #[test]
    fn test_text_box() {
        let text = TextNode::new("Hello World");
        let box_ = LayoutBox::text_box(&text);
        
        assert_eq!(box_.box_type, BoxType::TextRun);
        assert_eq!(box_.text_content, Some("Hello World".to_string()));
    }

    #[test]
    fn test_image_box_creation() {
        let el = Element::new("img", vec![]);
        let box_ = LayoutBox::image_box(&el, "test.png");
        
        assert_eq!(box_.box_type, BoxType::Image);
        assert!(box_.image_data.is_some());
        assert_eq!(box_.image_data.unwrap().src, "test.png");
    }

    #[test]
    fn test_intrinsic_size() {
        let size = IntrinsicSize::new(800.0, 600.0);
        assert_eq!(size.width, Some(800.0));
        assert_eq!(size.height, Some(600.0));
        assert_eq!(size.aspect_ratio, Some(800.0 / 600.0));
    }

    #[test]
    fn test_object_fit_parse() {
        assert_eq!(ObjectFit::from_css("fill"), ObjectFit::Fill);
        assert_eq!(ObjectFit::from_css("cover"), ObjectFit::Cover);
        assert_eq!(ObjectFit::from_css("contain"), ObjectFit::Contain);
        assert_eq!(ObjectFit::from_css("none"), ObjectFit::None);
        assert_eq!(ObjectFit::from_css("scale-down"), ObjectFit::ScaleDown);
    }

    #[test]
    fn test_object_position_parse() {
        let pos = ObjectPosition::from_css("center");
        assert_eq!(pos.horizontal, 0.5);
        assert_eq!(pos.vertical, 0.5);

        let pos = ObjectPosition::from_css("left top");
        assert_eq!(pos.horizontal, 0.0);
        assert_eq!(pos.vertical, 0.0);

        let pos = ObjectPosition::from_css("100% 0%");
        assert_eq!(pos.horizontal, 1.0);
        assert_eq!(pos.vertical, 0.0);
    }

    #[test]
    fn test_image_box_calculate_size() {
        // Test 1: No specified dimensions - use intrinsic, scaled to fit
        let mut img_box = ImageBox::new("test.png");
        img_box.intrinsic_size = IntrinsicSize::new(400.0, 300.0);
        
        let (w, h) = img_box.calculate_concrete_size(200.0, None, 16.0);
        assert_eq!(w, 200.0); // Scaled down to fit
        assert_eq!(h, 150.0); // Preserved aspect ratio

        // Test 2: Specified width (use a fresh ImageBox to avoid state issues)
        // Note: Px(100.0) converts to 75.0 pt (at 96 DPI where 1px = 0.75pt)
        let mut img_box2 = ImageBox::new("test2.png");
        img_box2.intrinsic_size = IntrinsicSize::new(400.0, 300.0);
        img_box2.specified_width = Some(Length::Px(100.0));
        let (w, h) = img_box2.calculate_concrete_size(200.0, None, 16.0);
        assert_eq!(w, 75.0); // 100px = 75pt
        assert_eq!(h, 56.25); // Preserved aspect ratio (75 * 300/400 = 56.25)
    }

    #[test]
    fn test_image_box_object_fit() {
        let container = Rect::new(0.0, 0.0, 100.0, 100.0);
        
        // Test Fill - should fill container
        let img_box = ImageBox::new("test.png")
            .with_intrinsic_size(200.0, 100.0)
            .with_object_fit(ObjectFit::Fill);
        let draw_rect = img_box.calculate_draw_rect(container, 200.0, 100.0);
        assert_eq!(draw_rect.width, 100.0);
        assert_eq!(draw_rect.height, 100.0);

        // Test Contain - should preserve aspect ratio
        let img_box = ImageBox::new("test.png")
            .with_intrinsic_size(200.0, 100.0)
            .with_object_fit(ObjectFit::Contain);
        let draw_rect = img_box.calculate_draw_rect(container, 200.0, 100.0);
        assert_eq!(draw_rect.width, 100.0);
        assert_eq!(draw_rect.height, 50.0); // Half height to preserve ratio
    }
}

// Extension trait for ImageBox builder pattern
impl ImageBox {
    fn with_object_fit(mut self, fit: ObjectFit) -> Self {
        self.object_fit = fit;
        self
    }
}

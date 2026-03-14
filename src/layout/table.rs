//! HTML Table Layout
//!
//! Implements CSS Table Layout Module Level 3
//! Supports: fixed and auto table layout, cell spanning, border-collapse

use crate::types::{Rect, Length};
use crate::layout::box_model::{LayoutBox, BoxType, EdgeSizes};
use crate::layout::style::ComputedStyle;
use crate::layout::flow::BlockFormattingContext;

/// Table layout mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TableLayout {
    /// Fixed table layout - uses specified column widths
    #[default]
    Fixed,
    /// Auto table layout - calculates column widths from content
    Auto,
}

/// Border collapse mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderCollapse {
    /// Separate borders (default)
    #[default]
    Separate,
    /// Collapsed borders
    Collapse,
}

/// Caption side position
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CaptionSide {
    /// Caption at the top (default)
    #[default]
    Top,
    /// Caption at the bottom
    Bottom,
    /// Caption on the left
    Left,
    /// Caption on the right
    Right,
}

/// Empty cells handling
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EmptyCells {
    /// Show empty cells with borders/background
    #[default]
    Show,
    /// Hide empty cells (show nothing)
    Hide,
}

/// Table box container
#[derive(Debug, Clone, PartialEq)]
pub struct TableBox {
    /// Table layout mode
    pub layout: TableLayout,
    /// Border collapse mode
    pub border_collapse: BorderCollapse,
    /// Border spacing (horizontal, vertical)
    pub border_spacing: (f32, f32),
    /// Caption side
    pub caption_side: CaptionSide,
    /// Empty cells handling
    pub empty_cells: EmptyCells,
    /// Table caption if any
    pub caption: Option<Box<LayoutBox>>,
    /// Column groups
    pub colgroups: Vec<TableColGroup>,
    /// Columns (can be outside colgroups)
    pub columns: Vec<TableColumn>,
    /// Table header (thead)
    pub header: Option<TableRowGroup>,
    /// Table body (tbody) - can have multiple
    pub bodies: Vec<TableRowGroup>,
    /// Table footer (tfoot)
    pub footer: Option<TableRowGroup>,
    /// Direct table rows (when no tbody)
    pub rows: Vec<TableRowBox>,
    /// Number of columns in the table
    pub col_count: usize,
    /// Number of rows in the table
    pub row_count: usize,
    /// Computed column widths
    pub column_widths: Vec<f32>,
    /// Computed row heights
    pub row_heights: Vec<f32>,
}

impl Default for TableBox {
    fn default() -> Self {
        Self {
            layout: TableLayout::Auto,
            border_collapse: BorderCollapse::Separate,
            border_spacing: (2.0, 2.0),
            caption_side: CaptionSide::Top,
            empty_cells: EmptyCells::Show,
            caption: None,
            colgroups: Vec::new(),
            columns: Vec::new(),
            header: None,
            bodies: Vec::new(),
            footer: None,
            rows: Vec::new(),
            col_count: 0,
            row_count: 0,
            column_widths: Vec::new(),
            row_heights: Vec::new(),
        }
    }
}

impl TableBox {
    /// Create a new table box with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create table box from computed style
    pub fn from_style(style: &ComputedStyle) -> Self {
        let mut table = Self::new();
        
        // These would be parsed from style in full implementation
        table.layout = TableLayout::Auto;
        table.border_collapse = BorderCollapse::Separate;
        table.border_spacing = (2.0, 2.0);
        table.caption_side = CaptionSide::Top;
        table.empty_cells = EmptyCells::Show;
        
        table
    }

    /// Get total width including spacing
    pub fn total_width(&self) -> f32 {
        let content_width: f32 = self.column_widths.iter().sum();
        let spacing_count = if self.col_count > 0 { self.col_count + 1 } else { 0 };
        let spacing_width = self.border_spacing.0 * spacing_count as f32;
        content_width + spacing_width
    }

    /// Get total height including spacing
    pub fn total_height(&self) -> f32 {
        let content_height: f32 = self.row_heights.iter().sum();
        let spacing_count = if self.row_count > 0 { self.row_count + 1 } else { 0 };
        let spacing_height = self.border_spacing.1 * spacing_count as f32;
        content_height + spacing_height
    }

    /// Get all rows from all row groups in visual order
    pub fn all_rows(&self) -> Vec<&TableRowBox> {
        let mut all_rows = Vec::new();
        
        // Header rows
        if let Some(ref header) = self.header {
            for row in &header.rows {
                all_rows.push(row);
            }
        }
        
        // Body rows
        for body in &self.bodies {
            for row in &body.rows {
                all_rows.push(row);
            }
        }
        
        // Footer rows
        if let Some(ref footer) = self.footer {
            for row in &footer.rows {
                all_rows.push(row);
            }
        }
        
        // Direct rows (no row group)
        for row in &self.rows {
            all_rows.push(row);
        }
        
        all_rows
    }

    /// Get all rows mutably
    pub fn all_rows_mut(&mut self) -> Vec<&mut TableRowBox> {
        let mut all_rows = Vec::new();
        
        // Header rows
        if let Some(ref mut header) = self.header {
            for row in &mut header.rows {
                all_rows.push(row);
            }
        }
        
        // Body rows
        for body in &mut self.bodies {
            for row in &mut body.rows {
                all_rows.push(row);
            }
        }
        
        // Footer rows
        if let Some(ref mut footer) = self.footer {
            for row in &mut footer.rows {
                all_rows.push(row);
            }
        }
        
        // Direct rows
        for row in &mut self.rows {
            all_rows.push(row);
        }
        
        all_rows
    }
}

/// Table column group
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableColGroup {
    /// Number of columns in this group
    pub span: usize,
    /// Width of columns in this group
    pub width: Option<Length>,
    /// Individual columns in this group
    pub columns: Vec<TableColumn>,
}

/// Table column
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableColumn {
    /// Column width (if specified)
    pub width: Option<Length>,
    /// Minimum column width
    pub min_width: f32,
    /// Maximum column width
    pub max_width: f32,
    /// Computed final width
    pub computed_width: f32,
}

/// Table row group (thead, tbody, tfoot)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableRowGroup {
    /// Rows in this group
    pub rows: Vec<TableRowBox>,
}

/// Table row box
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableRowBox {
    /// Cells in this row
    pub cells: Vec<TableCellBox>,
    /// Row height
    pub height: f32,
    /// Computed final height
    pub computed_height: f32,
    /// Row index
    pub row_index: usize,
}

impl TableRowBox {
    /// Create a new table row
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of columns this row spans
    pub fn col_span(&self) -> usize {
        self.cells.iter().map(|c| c.colspan).sum()
    }

    /// Get total min width of cells
    pub fn min_width(&self) -> f32 {
        self.cells.iter().map(|c| c.min_width).sum()
    }

    /// Get total max width of cells
    pub fn max_width(&self) -> f32 {
        self.cells.iter().map(|c| c.max_width).sum()
    }
}

/// Table cell box
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableCellBox {
    /// The layout box for this cell's content
    pub content: Option<LayoutBox>,
    /// Column span
    pub colspan: usize,
    /// Row span
    pub rowspan: usize,
    /// Cell width
    pub width: f32,
    /// Cell height
    pub height: f32,
    /// Minimum width based on content
    pub min_width: f32,
    /// Maximum width based on content
    pub max_width: f32,
    /// Minimum height based on content
    pub min_height: f32,
    /// Column index
    pub col_index: usize,
    /// Row index
    pub row_index: usize,
    /// Is this a header cell (th)
    pub is_header: bool,
    /// Horizontal alignment
    pub horizontal_align: CellAlign,
    /// Vertical alignment
    pub vertical_align: CellVerticalAlign,
}

/// Cell horizontal alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CellAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
    Char,
}

/// Cell vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CellVerticalAlign {
    #[default]
    Top,
    Middle,
    Bottom,
    Baseline,
}

/// Table layout context
#[derive(Debug)]
pub struct TableLayoutContext {
    /// Available width for the table
    pub available_width: f32,
    /// Base font size for calculations
    pub base_font_size: f32,
    /// Current column being processed
    pub current_col: usize,
    /// Current row being processed
    pub current_row: usize,
    /// Grid of occupied cells from rowspans
    pub occupied_grid: Vec<Vec<bool>>,
}

impl TableLayoutContext {
    pub fn new(available_width: f32, base_font_size: f32) -> Self {
        Self {
            available_width,
            base_font_size,
            current_col: 0,
            current_row: 0,
            occupied_grid: Vec::new(),
        }
    }

    /// Mark cells as occupied by a rowspan
    pub fn mark_occupied(&mut self, row: usize, col: usize, rowspan: usize, colspan: usize) {
        // Ensure grid is large enough
        let needed_rows = row + rowspan;
        while self.occupied_grid.len() < needed_rows {
            self.occupied_grid.push(Vec::new());
        }
        
        for r in row..(row + rowspan) {
            // Ensure row is large enough
            let needed_cols = col + colspan;
            while self.occupied_grid[r].len() < needed_cols {
                self.occupied_grid[r].push(false);
            }
            
            for c in col..(col + colspan) {
                if r > row || c > col {
                    // Don't mark the starting cell
                    self.occupied_grid[r][c] = true;
                }
            }
        }
    }

    /// Check if a cell position is occupied
    pub fn is_occupied(&self, row: usize, col: usize) -> bool {
        self.occupied_grid.get(row).and_then(|r| r.get(col)).copied().unwrap_or(false)
    }

    /// Find the next available column for a row
    pub fn next_available_col(&self, row: usize, start_col: usize) -> usize {
        let mut col = start_col;
        while self.is_occupied(row, col) {
            col += 1;
        }
        col
    }
}

/// Build a table box from a layout box
pub fn build_table_box(box_: &LayoutBox, style: &ComputedStyle) -> TableBox {
    let mut table = TableBox::from_style(style);
    
    // Parse table element children to extract structure
    if let Some(element) = box_.element() {
        let tag_name = element.tag_name().to_ascii_lowercase();
        
        if tag_name == "table" {
            // Process caption
            for child in element.children() {
                if let Some(el) = child.as_element() {
                    let child_tag = el.tag_name().to_ascii_lowercase();
                    
                    match child_tag.as_str() {
                        "caption" => {
                            let caption_box = LayoutBox::block_box(el);
                            table.caption = Some(Box::new(caption_box));
                        }
                        "colgroup" => {
                            let colgroup = build_colgroup(el);
                            table.colgroups.push(colgroup);
                        }
                        "col" => {
                            let col = build_column(el);
                            table.columns.push(col);
                        }
                        "thead" => {
                            table.header = Some(build_row_group(el));
                        }
                        "tbody" => {
                            table.bodies.push(build_row_group(el));
                        }
                        "tfoot" => {
                            table.footer = Some(build_row_group(el));
                        }
                        "tr" => {
                            let row = build_row(el);
                            table.rows.push(row);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Calculate column count
    table.col_count = calculate_column_count(&table);
    table.row_count = calculate_row_count(&table);
    
    // Initialize column widths array
    table.column_widths = vec![0.0; table.col_count];
    table.row_heights = vec![0.0; table.row_count];
    
    table
}

/// Build a column group from a colgroup element
fn build_colgroup(element: &crate::html::Element) -> TableColGroup {
    let mut colgroup = TableColGroup::default();
    
    // Get span attribute
    if let Some(span_attr) = element.attr("span") {
        if let Ok(span) = span_attr.parse::<usize>() {
            colgroup.span = span;
        }
    }
    
    // Process child col elements
    for child in element.children() {
        if let Some(el) = child.as_element() {
            if el.tag_name().eq_ignore_ascii_case("col") {
                let col = build_column(el);
                colgroup.columns.push(col);
            }
        }
    }
    
    // If span not specified, use number of columns
    if colgroup.span == 0 {
        colgroup.span = colgroup.columns.len().max(1);
    }
    
    colgroup
}

/// Build a column from a col element
fn build_column(element: &crate::html::Element) -> TableColumn {
    let mut col = TableColumn::default();
    
    // Parse width attribute
    if let Some(width_attr) = element.attr("width") {
        col.width = parse_table_length(width_attr);
    }
    
    // Get span attribute
    if let Some(span_attr) = element.attr("span") {
        if let Ok(_span) = span_attr.parse::<usize>() {
            // span handled at colgroup level
        }
    }
    
    col
}

/// Build a row group from thead/tbody/tfoot element
fn build_row_group(element: &crate::html::Element) -> TableRowGroup {
    let mut group = TableRowGroup::default();
    
    for child in element.children() {
        if let Some(el) = child.as_element() {
            if el.tag_name().eq_ignore_ascii_case("tr") {
                let row = build_row(el);
                group.rows.push(row);
            }
        }
    }
    
    group
}

/// Build a row from a tr element
fn build_row(element: &crate::html::Element) -> TableRowBox {
    let mut row = TableRowBox::new();
    
    for child in element.children() {
        if let Some(el) = child.as_element() {
            let tag_name = el.tag_name().to_ascii_lowercase();
            
            if tag_name == "td" || tag_name == "th" {
                let mut cell = build_cell(el);
                cell.is_header = tag_name == "th";
                row.cells.push(cell);
            }
        }
    }
    
    row
}

/// Build a cell from td/th element
fn build_cell(element: &crate::html::Element) -> TableCellBox {
    let mut cell = TableCellBox::default();
    
    // Parse colspan
    if let Some(colspan_attr) = element.attr("colspan") {
        if let Ok(colspan) = colspan_attr.parse::<usize>() {
            cell.colspan = colspan.max(1);
        }
    }
    
    // Parse rowspan
    if let Some(rowspan_attr) = element.attr("rowspan") {
        if let Ok(rowspan) = rowspan_attr.parse::<usize>() {
            cell.rowspan = rowspan.max(1);
        }
    }
    
    // Parse width
    if let Some(width_attr) = element.attr("width") {
        if let Ok(w) = width_attr.parse::<f32>() {
            cell.width = w;
        }
    }
    
    // Parse height
    if let Some(height_attr) = element.attr("height") {
        if let Ok(h) = height_attr.parse::<f32>() {
            cell.height = h;
        }
    }
    
    // Parse align
    if let Some(align_attr) = element.attr("align") {
        cell.horizontal_align = match align_attr.to_ascii_lowercase().as_str() {
            "center" => CellAlign::Center,
            "right" => CellAlign::Right,
            "justify" => CellAlign::Justify,
            "char" => CellAlign::Char,
            _ => CellAlign::Left,
        };
    }
    
    // Parse valign
    if let Some(valign_attr) = element.attr("valign") {
        cell.vertical_align = match valign_attr.to_ascii_lowercase().as_str() {
            "top" => CellVerticalAlign::Top,
            "middle" => CellVerticalAlign::Middle,
            "bottom" => TableVerticalAlign::Bottom,
            _ => CellVerticalAlign::Middle,
        };
    }
    
    cell
}

/// Calculate the number of columns in the table
fn calculate_column_count(table: &TableBox) -> usize {
    let mut max_cols = 0;
    
    // Check column definitions
    let col_def_count: usize = table.colgroups.iter()
        .map(|cg| cg.span)
        .sum();
    max_cols = max_cols.max(col_def_count);
    max_cols = max_cols.max(table.columns.len());
    
    // Check all rows
    for row in table.all_rows() {
        let row_cols: usize = row.cells.iter()
            .map(|c| c.colspan)
            .sum();
        max_cols = max_cols.max(row_cols);
    }
    
    max_cols.max(1)
}

/// Calculate the number of rows in the table
fn calculate_row_count(table: &TableBox) -> usize {
    let mut row_count = 0;
    
    // Count rows in groups
    if let Some(ref header) = table.header {
        row_count += header.rows.len();
    }
    
    for body in &table.bodies {
        row_count += body.rows.len();
    }
    
    if let Some(ref footer) = table.footer {
        row_count += footer.rows.len();
    }
    
    // Count direct rows
    row_count += table.rows.len();
    
    row_count.max(1)
}

/// Parse table length value (pixels or percentage)
fn parse_table_length(value: &str) -> Option<Length> {
    let value = value.trim();
    
    if value.ends_with('%') {
        let num = value.trim_end_matches('%').trim();
        if let Ok(p) = num.parse::<f32>() {
            return Some(Length::Percent(p));
        }
    } else if value.ends_with("px") {
        let num = value.trim_end_matches("px").trim();
        if let Ok(px) = num.parse::<f32>() {
            return Some(Length::Px(px));
        }
    } else {
        // Try parsing as plain number (pixels)
        if let Ok(px) = value.parse::<f32>() {
            return Some(Length::Px(px));
        }
    }
    
    None
}

/// Layout a table container
pub fn layout_table(
    box_: &mut LayoutBox,
    table_data: &mut TableBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let available_width = box_.dimensions.content.width;
    let mut context = TableLayoutContext::new(available_width, base_font_size);
    
    // Step 1: Build cell content boxes and calculate intrinsic widths
    build_cell_content_boxes(table_data, style_resolver);
    
    // Step 2: Calculate column widths
    match table_data.layout {
        TableLayout::Fixed => {
            layout_fixed_table_width(table_data, available_width, base_font_size);
        }
        TableLayout::Auto => {
            layout_auto_table_width(table_data, available_width, base_font_size);
        }
    }
    
    // Step 3: Calculate row heights
    calculate_row_heights(table_data, base_font_size);
    
    // Step 4: Position cells
    position_table_cells(table_data, box_, bfc, style_resolver, base_font_size);
    
    // Step 5: Set final table dimensions
    let table_width = if table_data.border_collapse == BorderCollapse::Collapse {
        table_data.column_widths.iter().sum()
    } else {
        table_data.total_width()
    };
    
    let table_height = if table_data.border_collapse == BorderCollapse::Collapse {
        table_data.row_heights.iter().sum()
    } else {
        table_data.total_height()
    };
    
    box_.dimensions.content.width = table_width;
    box_.dimensions.content.height = table_height;
    box_.is_laid_out = true;
}

/// Build content boxes for table cells
fn build_cell_content_boxes(
    table: &mut TableBox,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
) {
    // This would recursively build layout boxes for cell content
    // For now, we set up placeholder dimensions based on content
    
    let all_rows = table.all_rows_mut();
    
    for (row_idx, row) in all_rows.iter_mut().enumerate() {
        for (cell_idx, cell) in row.cells.iter_mut().enumerate() {
            cell.row_index = row_idx;
            cell.col_index = cell_idx;
            
            // Estimate min/max widths based on content
            // In full implementation, this would measure actual content
            cell.min_width = 20.0; // Minimum cell width
            cell.max_width = 200.0; // Maximum cell width before wrapping
            cell.min_height = 20.0; // Minimum cell height
            
            // Use explicit dimensions if specified
            if cell.width > 0.0 {
                cell.min_width = cell.width;
                cell.max_width = cell.width;
            }
            
            if cell.height > 0.0 {
                cell.min_height = cell.height;
            }
        }
    }
}

/// Layout table with fixed layout algorithm
fn layout_fixed_table_width(
    table: &mut TableBox,
    available_width: f32,
    base_font_size: f32,
) {
    let col_count = table.col_count;
    if col_count == 0 {
        return;
    }
    
    // First pass: collect specified widths
    let mut specified_widths: Vec<Option<f32>> = vec![None; col_count];
    let mut total_specified: f32 = 0.0;
    let mut specified_count = 0;
    
    // Check column definitions
    let mut col_idx = 0;
    for colgroup in &table.colgroups {
        for col in &colgroup.columns {
            if col_idx >= col_count {
                break;
            }
            if let Some(width) = col.width {
                let w = width.to_pt_with_container(base_font_size, available_width);
                if w > 0.0 {
                    specified_widths[col_idx] = Some(w);
                    total_specified += w;
                    specified_count += 1;
                }
            }
            col_idx += 1;
        }
    }
    
    // Check direct columns
    for (i, col) in table.columns.iter().enumerate() {
        if i >= col_count {
            break;
        }
        if let Some(width) = col.width {
            let w = width.to_pt_with_container(base_font_size, available_width);
            if w > 0.0 {
                specified_widths[i] = Some(w);
                total_specified += w;
                specified_count += 1;
            }
        }
    }
    
    // Check cell widths (first row)
    let all_rows = table.all_rows();
    if let Some(first_row) = all_rows.first() {
        for (i, cell) in first_row.cells.iter().enumerate() {
            if i >= col_count {
                break;
            }
            if cell.width > 0.0 && specified_widths[i].is_none() {
                specified_widths[i] = Some(cell.width);
                total_specified += cell.width;
                specified_count += 1;
            }
        }
    }
    
    // Distribute remaining width
    let remaining_cols = col_count - specified_count;
    let border_spacing_total = if table.border_collapse == BorderCollapse::Separate {
        table.border_spacing.0 * (col_count + 1) as f32
    } else {
        0.0
    };
    
    let remaining_width = (available_width - total_specified - border_spacing_total).max(0.0);
    let default_width = if remaining_cols > 0 {
        remaining_width / remaining_cols as f32
    } else {
        0.0
    };
    
    // Assign final widths
    for (i, width) in specified_widths.iter().enumerate() {
        table.column_widths[i] = width.unwrap_or(default_width).max(10.0);
    }
}

/// Layout table with auto layout algorithm
fn layout_auto_table_width(
    table: &mut TableBox,
    available_width: f32,
    _base_font_size: f32,
) {
    let col_count = table.col_count;
    if col_count == 0 {
        return;
    }
    
    // Initialize min/max widths for each column
    let mut col_min_widths: Vec<f32> = vec![0.0; col_count];
    let mut col_max_widths: Vec<f32> = vec![0.0; col_count];
    
    // Calculate min/max widths from cells
    let all_rows = table.all_rows();
    
    for row in &all_rows {
        let mut col_idx = 0;
        
        for cell in &row.cells {
            // Skip cells that span multiple columns for now (simplified)
            if cell.colspan == 1 {
                col_min_widths[col_idx] = col_min_widths[col_idx].max(cell.min_width);
                col_max_widths[col_idx] = col_max_widths[col_idx].max(cell.max_width);
            }
            
            col_idx += cell.colspan;
        }
    }
    
    // Calculate total min/max widths
    let total_min: f32 = col_min_widths.iter().sum();
    let total_max: f32 = col_max_widths.iter().sum();
    
    let border_spacing_total = if table.border_collapse == BorderCollapse::Separate {
        table.border_spacing.0 * (col_count + 1) as f32
    } else {
        0.0
    };
    
    let content_width = available_width - border_spacing_total;
    
    // Distribute available width
    if total_max <= content_width {
        // Use max widths and distribute extra
        let extra = (content_width - total_max) / col_count as f32;
        for i in 0..col_count {
            table.column_widths[i] = col_max_widths[i] + extra;
        }
    } else if total_min <= content_width {
        // Distribute between min and max
        let ratio = (content_width - total_min) / (total_max - total_min);
        for i in 0..col_count {
            let range = col_max_widths[i] - col_min_widths[i];
            table.column_widths[i] = col_min_widths[i] + range * ratio;
        }
    } else {
        // Use min widths (table overflows)
        for i in 0..col_count {
            table.column_widths[i] = col_min_widths[i];
        }
    }
}

/// Calculate row heights
fn calculate_row_heights(table: &mut TableBox, _base_font_size: f32) {
    let row_count = table.row_count;
    if row_count == 0 {
        return;
    }
    
    let all_rows = table.all_rows();
    
    for (row_idx, row) in all_rows.iter().enumerate() {
        let mut max_height: f32 = 0.0;
        
        for cell in &row.cells {
            // For cells spanning single row, use cell min height
            if cell.rowspan == 1 {
                max_height = max_height.max(cell.min_height);
            }
        }
        
        // Ensure minimum row height
        table.row_heights[row_idx] = max_height.max(20.0);
    }
}

/// Position table cells
fn position_table_cells(
    table: &TableBox,
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let table_x = box_.dimensions.content.x;
    let table_y = bfc.current_y;
    
    let mut current_y = table_y;
    
    // Handle caption first
    if let Some(ref caption) = table.caption {
        // Position caption based on caption-side
        match table.caption_side {
            CaptionSide::Top => {
                // Caption goes above table
                // For now, skip detailed caption layout
                current_y += caption.dimensions.content.height;
            }
            CaptionSide::Bottom => {
                // Caption goes below table (handled after table)
            }
            _ => {}
        }
    }
    
    // Add initial border spacing
    if table.border_collapse == BorderCollapse::Separate {
        current_y += table.border_spacing.1;
    }
    
    // Position rows and cells
    let all_rows = table.all_rows();
    
    for (row_idx, row) in all_rows.iter().enumerate() {
        let row_height = table.row_heights.get(row_idx).copied().unwrap_or(20.0);
        let mut current_x = table_x;
        
        // Add initial border spacing
        if table.border_collapse == BorderCollapse::Separate {
            current_x += table.border_spacing.0;
        }
        
        for (cell_idx, cell) in row.cells.iter().enumerate() {
            if cell_idx >= table.col_count {
                break;
            }
            
            // Calculate cell width (sum of spanned columns)
            let cell_width: f32 = table.column_widths[cell_idx..(cell_idx + cell.colspan).min(table.col_count)]
                .iter()
                .sum();
            
            // Add spacing between columns
            let spacing_width = if table.border_collapse == BorderCollapse::Separate && cell.colspan > 1 {
                table.border_spacing.0 * (cell.colspan - 1) as f32
            } else {
                0.0
            };
            
            let total_cell_width = cell_width + spacing_width;
            
            // Calculate cell height (sum of spanned rows)
            let cell_height: f32 = table.row_heights[row_idx..(row_idx + cell.rowspan).min(row_count)]
                .iter()
                .sum();
            
            let spacing_height = if table.border_collapse == BorderCollapse::Separate && cell.rowspan > 1 {
                table.border_spacing.1 * (cell.rowspan - 1) as f32
            } else {
                0.0
            };
            
            let total_cell_height = cell_height + spacing_height;
            
            // Position cell content if it exists
            if let Some(ref mut content) = cell.content {
                content.dimensions.content.x = current_x;
                content.dimensions.content.y = current_y;
                content.dimensions.content.width = total_cell_width;
                content.dimensions.content.height = total_cell_height;
                content.is_laid_out = true;
            }
            
            current_x += total_cell_width;
            
            // Add border spacing between columns
            if table.border_collapse == BorderCollapse::Separate {
                current_x += table.border_spacing.0;
            }
        }
        
        current_y += row_height;
        
        // Add border spacing between rows
        if table.border_collapse == BorderCollapse::Separate {
            current_y += table.border_spacing.1;
        }
    }
    
    // Update BFC position
    bfc.current_y = current_y;
}

/// Parse table-layout CSS property
pub fn parse_table_layout(value: &crate::css::CssValue) -> TableLayout {
    match value {
        crate::css::CssValue::Ident(s) => match s.as_str() {
            "fixed" => TableLayout::Fixed,
            "auto" => TableLayout::Auto,
            _ => TableLayout::Auto,
        },
        _ => TableLayout::Auto,
    }
}

/// Parse border-collapse CSS property
pub fn parse_border_collapse(value: &crate::css::CssValue) -> BorderCollapse {
    match value {
        crate::css::CssValue::Ident(s) => match s.as_str() {
            "collapse" => BorderCollapse::Collapse,
            "separate" => BorderCollapse::Separate,
            _ => BorderCollapse::Separate,
        },
        _ => BorderCollapse::Separate,
    }
}

/// Parse caption-side CSS property
pub fn parse_caption_side(value: &crate::css::CssValue) -> CaptionSide {
    match value {
        crate::css::CssValue::Ident(s) => match s.as_str() {
            "top" => CaptionSide::Top,
            "bottom" => CaptionSide::Bottom,
            "left" => CaptionSide::Left,
            "right" => CaptionSide::Right,
            _ => CaptionSide::Top,
        },
        _ => CaptionSide::Top,
    }
}

/// Parse border-spacing CSS property
pub fn parse_border_spacing(value: &crate::css::CssValue) -> (f32, f32) {
    match value {
        crate::css::CssValue::List(values) if values.len() >= 2 => {
            let h = parse_spacing_value(&values[0]);
            let v = parse_spacing_value(&values[1]);
            (h, v)
        }
        _ => {
            let v = parse_spacing_value(value);
            (v, v)
        }
    }
}

fn parse_spacing_value(value: &crate::css::CssValue) -> f32 {
    match value {
        crate::css::CssValue::Length(n, _) => *n,
        crate::css::CssValue::Number(n) => *n,
        _ => 2.0,
    }
}

/// Parse empty-cells CSS property
pub fn parse_empty_cells(value: &crate::css::CssValue) -> EmptyCells {
    match value {
        crate::css::CssValue::Ident(s) => match s.as_str() {
            "show" => EmptyCells::Show,
            "hide" => EmptyCells::Hide,
            _ => EmptyCells::Show,
        },
        _ => EmptyCells::Show,
    }
}

/// Check if element is a table element
pub fn is_table_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_ascii_lowercase().as_str(),
        "table" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" | "caption" | "colgroup" | "col"
    )
}

/// Get the display type for a table element
pub fn table_element_display(tag_name: &str) -> Option<crate::layout::style::Display> {
    use crate::layout::style::Display;
    
    match tag_name.to_ascii_lowercase().as_str() {
        "table" => Some(Display::Table),
        "thead" | "tbody" | "tfoot" => Some(Display::TableRowGroup),
        "tr" => Some(Display::TableRow),
        "td" | "th" => Some(Display::TableCell),
        "caption" => Some(Display::Block), // Caption is block-level
        "colgroup" => Some(Display::TableColumnGroup),
        "col" => Some(Display::TableColumn),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::Element;
    use crate::layout::box_model::LayoutBox;

    #[test]
    fn test_table_box_creation() {
        let table = TableBox::new();
        assert_eq!(table.layout, TableLayout::Auto);
        assert_eq!(table.border_collapse, BorderCollapse::Separate);
        assert_eq!(table.border_spacing, (2.0, 2.0));
    }

    #[test]
    fn test_table_box_from_style() {
        let style = ComputedStyle::default();
        let table = TableBox::from_style(&style);
        assert_eq!(table.caption_side, CaptionSide::Top);
        assert_eq!(table.empty_cells, EmptyCells::Show);
    }

    #[test]
    fn test_table_row() {
        let mut row = TableRowBox::new();
        
        let cell1 = TableCellBox {
            colspan: 1,
            min_width: 50.0,
            max_width: 100.0,
            ..Default::default()
        };
        
        let cell2 = TableCellBox {
            colspan: 2,
            min_width: 75.0,
            max_width: 150.0,
            ..Default::default()
        };
        
        row.cells.push(cell1);
        row.cells.push(cell2);
        
        assert_eq!(row.col_span(), 3);
        assert_eq!(row.min_width(), 125.0);
        assert_eq!(row.max_width(), 250.0);
    }

    #[test]
    fn test_table_cell_defaults() {
        let cell = TableCellBox::default();
        assert_eq!(cell.colspan, 0);
        assert_eq!(cell.rowspan, 0);
        assert!(!cell.is_header);
    }

    #[test]
    fn test_caption_side_variants() {
        assert_eq!(CaptionSide::Top, CaptionSide::default());
        
        let top = parse_caption_side(&crate::css::CssValue::Ident("top".to_string()));
        let bottom = parse_caption_side(&crate::css::CssValue::Ident("bottom".to_string()));
        let left = parse_caption_side(&crate::css::CssValue::Ident("left".to_string()));
        let right = parse_caption_side(&crate::css::CssValue::Ident("right".to_string()));
        
        assert_eq!(top, CaptionSide::Top);
        assert_eq!(bottom, CaptionSide::Bottom);
        assert_eq!(left, CaptionSide::Left);
        assert_eq!(right, CaptionSide::Right);
    }

    #[test]
    fn test_border_collapse_variants() {
        let collapse = parse_border_collapse(&crate::css::CssValue::Ident("collapse".to_string()));
        let separate = parse_border_collapse(&crate::css::CssValue::Ident("separate".to_string()));
        
        assert_eq!(collapse, BorderCollapse::Collapse);
        assert_eq!(separate, BorderCollapse::Separate);
    }

    #[test]
    fn test_table_layout_variants() {
        let fixed = parse_table_layout(&crate::css::CssValue::Ident("fixed".to_string()));
        let auto = parse_table_layout(&crate::css::CssValue::Ident("auto".to_string()));
        
        assert_eq!(fixed, TableLayout::Fixed);
        assert_eq!(auto, TableLayout::Auto);
    }

    #[test]
    fn test_empty_cells_variants() {
        let show = parse_empty_cells(&crate::css::CssValue::Ident("show".to_string()));
        let hide = parse_empty_cells(&crate::css::CssValue::Ident("hide".to_string()));
        
        assert_eq!(show, EmptyCells::Show);
        assert_eq!(hide, EmptyCells::Hide);
    }

    #[test]
    fn test_border_spacing_parsing() {
        // Single value
        let single = parse_border_spacing(&crate::css::CssValue::Length(10.0, crate::css::Unit::Px));
        assert_eq!(single, (10.0, 10.0));
        
        // Two values
        let two = parse_border_spacing(&crate::css::CssValue::List(vec![
            crate::css::CssValue::Length(5.0, crate::css::Unit::Px),
            crate::css::CssValue::Length(10.0, crate::css::Unit::Px),
        ]));
        assert_eq!(two, (5.0, 10.0));
    }

    #[test]
    fn test_table_column() {
        let col = TableColumn {
            width: Some(Length::Px(100.0)),
            min_width: 50.0,
            max_width: 200.0,
            computed_width: 100.0,
        };
        
        assert_eq!(col.width, Some(Length::Px(100.0)));
        assert_eq!(col.min_width, 50.0);
        assert_eq!(col.max_width, 200.0);
    }

    #[test]
    fn test_table_colgroup() {
        let mut colgroup = TableColGroup::default();
        colgroup.span = 3;
        
        let col1 = TableColumn {
            width: Some(Length::Px(50.0)),
            ..Default::default()
        };
        
        colgroup.columns.push(col1);
        
        assert_eq!(colgroup.span, 3);
        assert_eq!(colgroup.columns.len(), 1);
    }

    #[test]
    fn test_table_row_group() {
        let mut group = TableRowGroup::default();
        
        let row1 = TableRowBox::new();
        let row2 = TableRowBox::new();
        
        group.rows.push(row1);
        group.rows.push(row2);
        
        assert_eq!(group.rows.len(), 2);
    }

    #[test]
    fn test_is_table_element() {
        assert!(is_table_element("table"));
        assert!(is_table_element("thead"));
        assert!(is_table_element("tbody"));
        assert!(is_table_element("tfoot"));
        assert!(is_table_element("tr"));
        assert!(is_table_element("td"));
        assert!(is_table_element("th"));
        assert!(is_table_element("caption"));
        assert!(is_table_element("colgroup"));
        assert!(is_table_element("col"));
        
        assert!(!is_table_element("div"));
        assert!(!is_table_element("span"));
        assert!(!is_table_element("p"));
    }

    #[test]
    fn test_table_element_display() {
        use crate::layout::style::Display;
        
        assert_eq!(table_element_display("table"), Some(Display::Table));
        assert_eq!(table_element_display("tr"), Some(Display::TableRow));
        assert_eq!(table_element_display("td"), Some(Display::TableCell));
        assert_eq!(table_element_display("th"), Some(Display::TableCell));
        assert_eq!(table_element_display("caption"), Some(Display::Block));
        
        assert_eq!(table_element_display("div"), None);
    }

    #[test]
    fn test_table_layout_context() {
        let mut ctx = TableLayoutContext::new(500.0, 12.0);
        
        assert_eq!(ctx.available_width, 500.0);
        assert_eq!(ctx.base_font_size, 12.0);
        
        // Mark cells as occupied
        ctx.mark_occupied(0, 0, 2, 2);
        
        assert!(ctx.is_occupied(0, 0)); // Starting cell
        assert!(ctx.is_occupied(0, 1)); // Same row, next col
        assert!(ctx.is_occupied(1, 0)); // Next row, same col
        assert!(ctx.is_occupied(1, 1)); // Next row, next col
        assert!(!ctx.is_occupied(0, 2)); // Outside span
        assert!(!ctx.is_occupied(2, 0)); // Outside span
    }

    #[test]
    fn test_fixed_table_layout() {
        let mut table = TableBox::new();
        table.layout = TableLayout::Fixed;
        table.col_count = 3;
        table.column_widths = vec![0.0; 3];
        
        // Add columns with widths
        table.columns.push(TableColumn {
            width: Some(Length::Px(100.0)),
            ..Default::default()
        });
        table.columns.push(TableColumn {
            width: Some(Length::Px(150.0)),
            ..Default::default()
        });
        
        layout_fixed_table_width(&mut table, 500.0, 12.0);
        
        // First two columns should use specified widths
        assert!(table.column_widths[0] > 0.0);
        assert!(table.column_widths[1] > 0.0);
        assert!(table.column_widths[2] > 0.0); // Third column gets remaining
    }

    #[test]
    fn test_auto_table_layout() {
        let mut table = TableBox::new();
        table.layout = TableLayout::Auto;
        table.col_count = 2;
        table.column_widths = vec![0.0; 2];
        
        // Create a row with cells
        let mut row = TableRowBox::new();
        row.cells.push(TableCellBox {
            min_width: 50.0,
            max_width: 100.0,
            colspan: 1,
            ..Default::default()
        });
        row.cells.push(TableCellBox {
            min_width: 75.0,
            max_width: 150.0,
            colspan: 1,
            ..Default::default()
        });
        table.rows.push(row);
        
        layout_auto_table_width(&mut table, 300.0, 12.0);
        
        // Both columns should have positive widths
        assert!(table.column_widths[0] > 0.0);
        assert!(table.column_widths[1] > 0.0);
    }

    #[test]
    fn test_calculate_row_heights() {
        let mut table = TableBox::new();
        table.row_count = 2;
        table.row_heights = vec![0.0; 2];
        
        let mut row1 = TableRowBox::new();
        row1.cells.push(TableCellBox {
            min_height: 30.0,
            rowspan: 1,
            ..Default::default()
        });
        
        let mut row2 = TableRowBox::new();
        row2.cells.push(TableCellBox {
            min_height: 40.0,
            rowspan: 1,
            ..Default::default()
        });
        
        table.rows.push(row1);
        table.rows.push(row2);
        
        calculate_row_heights(&mut table, 12.0);
        
        assert!(table.row_heights[0] >= 30.0);
        assert!(table.row_heights[1] >= 40.0);
    }

    #[test]
    fn test_table_total_width() {
        let mut table = TableBox::new();
        table.border_collapse = BorderCollapse::Separate;
        table.border_spacing = (2.0, 2.0);
        table.col_count = 2;
        table.column_widths = vec![100.0, 150.0];
        
        // Width = 100 + 150 + spacing * (2+1) = 250 + 6 = 256
        let total = table.total_width();
        assert_eq!(total, 256.0);
    }

    #[test]
    fn test_table_total_height() {
        let mut table = TableBox::new();
        table.border_collapse = BorderCollapse::Separate;
        table.border_spacing = (2.0, 2.0);
        table.row_count = 2;
        table.row_heights = vec![30.0, 40.0];
        
        // Height = 30 + 40 + spacing * (2+1) = 70 + 6 = 76
        let total = table.total_height();
        assert_eq!(total, 76.0);
    }

    #[test]
    fn test_cell_align_variants() {
        assert_eq!(CellAlign::Left, CellAlign::default());
        
        let left = CellAlign::Left;
        let center = CellAlign::Center;
        let right = CellAlign::Right;
        let justify = CellAlign::Justify;
        let char_align = CellAlign::Char;
        
        assert_ne!(left, center);
        assert_ne!(center, right);
        assert_ne!(right, justify);
    }

    #[test]
    fn test_cell_vertical_align_variants() {
        assert_eq!(CellVerticalAlign::Middle, CellVerticalAlign::default());
        
        let top = CellVerticalAlign::Top;
        let middle = CellVerticalAlign::Middle;
        let bottom = CellVerticalAlign::Bottom;
        let baseline = CellVerticalAlign::Baseline;
        
        assert_ne!(top, middle);
        assert_ne!(middle, bottom);
        assert_ne!(bottom, baseline);
    }
}

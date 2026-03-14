//! CSS Grid Layout
//!
//! Implements CSS Grid Layout Module Level 1
//! Supports: grid-template-columns, grid-template-rows, grid-template-areas,
//!           grid-auto-columns, grid-auto-rows, grid-auto-flow, gap,
//!           grid-column/row-start/end, grid-area

use crate::types::{Rect, Length};
use crate::layout::box_model::{LayoutBox, BoxType};
use crate::layout::style::ComputedStyle;
use crate::layout::flow::BlockFormattingContext;

/// Grid track sizing function
#[derive(Debug, Clone, PartialEq)]
pub enum TrackSizingFunction {
    /// Fixed length (px, em, etc.)
    Length(f32),
    /// Percentage of container
    Percentage(f32),
    /// Flexible length (fr unit)
    Flex(f32),
    /// minmax(min, max) function
    MinMax(Box<TrackSizingFunction>, Box<TrackSizingFunction>),
    /// Auto track
    Auto,
    /// min-content
    MinContent,
    /// max-content
    MaxContent,
    /// fit-content(length)
    FitContent(f32),
}

impl Default for TrackSizingFunction {
    fn default() -> Self {
        TrackSizingFunction::Auto
    }
}

impl TrackSizingFunction {
    /// Check if this is a fixed size (not flexible)
    pub fn is_fixed(&self) -> bool {
        matches!(self, TrackSizingFunction::Length(_) | TrackSizingFunction::Percentage(_))
    }

    /// Check if this is an intrinsic size (min-content, max-content, auto)
    pub fn is_intrinsic(&self) -> bool {
        matches!(self, 
            TrackSizingFunction::MinContent | 
            TrackSizingFunction::MaxContent | 
            TrackSizingFunction::Auto |
            TrackSizingFunction::FitContent(_)
        )
    }

    /// Check if this is a flexible track (fr unit)
    pub fn is_flexible(&self) -> bool {
        matches!(self, TrackSizingFunction::Flex(_))
    }

    /// Get the flex factor if this is a flexible track
    pub fn flex_factor(&self) -> Option<f32> {
        match self {
            TrackSizingFunction::Flex(f) => Some(*f),
            _ => None,
        }
    }
}

/// Grid auto-flow direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GridAutoFlow {
    /// Place items in row order (default)
    #[default]
    Row,
    /// Place items in column order
    Column,
    /// Place items in row order, filling in dense
    RowDense,
    /// Place items in column order, filling in dense
    ColumnDense,
}

impl GridAutoFlow {
    /// Check if flow is in row direction
    pub fn is_row(&self) -> bool {
        matches!(self, GridAutoFlow::Row | GridAutoFlow::RowDense)
    }

    /// Check if flow is dense (backfill empty cells)
    pub fn is_dense(&self) -> bool {
        matches!(self, GridAutoFlow::RowDense | GridAutoFlow::ColumnDense)
    }
}

/// Grid line definition (start or end)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GridLine {
    /// Line number (1-indexed, 0 = auto)
    pub line_number: i32,
    /// Named grid area (for grid-area property)
    pub area_name: Option<String>,
    /// Span count (for span N syntax)
    pub span: Option<i32>,
}

impl GridLine {
    /// Create an automatic grid line
    pub fn auto() -> Self {
        Self {
            line_number: 0,
            area_name: None,
            span: None,
        }
    }

    /// Create a grid line with specific number
    pub fn numbered(n: i32) -> Self {
        Self {
            line_number: n,
            area_name: None,
            span: None,
        }
    }

    /// Create a spanning grid line
    pub fn span(n: i32) -> Self {
        Self {
            line_number: 0,
            area_name: None,
            span: Some(n),
        }
    }

    /// Check if this is auto
    pub fn is_auto(&self) -> bool {
        self.line_number == 0 && self.area_name.is_none() && self.span.is_none()
    }
}

/// Grid item placement
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GridPlacement {
    /// Column start line
    pub column_start: GridLine,
    /// Column end line
    pub column_end: GridLine,
    /// Row start line
    pub row_start: GridLine,
    /// Row end line
    pub row_end: GridLine,
}

impl GridPlacement {
    /// Create a placement for a named grid area
    pub fn from_area_name(name: &str) -> Self {
        Self {
            column_start: GridLine { area_name: Some(name.to_string()), ..Default::default() },
            column_end: GridLine::auto(),
            row_start: GridLine::auto(),
            row_end: GridLine::auto(),
        }
    }

    /// Check if this placement is fully automatic
    pub fn is_auto(&self) -> bool {
        self.column_start.is_auto() && 
        self.column_end.is_auto() && 
        self.row_start.is_auto() && 
        self.row_end.is_auto()
    }

    /// Get the column span (number of tracks)
    pub fn column_span(&self) -> i32 {
        if let Some(span) = self.column_start.span {
            span
        } else if let Some(span) = self.column_end.span {
            span
        } else if self.column_start.line_number > 0 && self.column_end.line_number > 0 {
            (self.column_end.line_number - self.column_start.line_number).max(1)
        } else {
            1
        }
    }

    /// Get the row span (number of tracks)
    pub fn row_span(&self) -> i32 {
        if let Some(span) = self.row_start.span {
            span
        } else if let Some(span) = self.row_end.span {
            span
        } else if self.row_start.line_number > 0 && self.row_end.line_number > 0 {
            (self.row_end.line_number - self.row_start.line_number).max(1)
        } else {
            1
        }
    }
}

/// Grid container properties
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridContainer {
    /// Explicit column track definitions
    pub template_columns: Vec<TrackSizingFunction>,
    /// Explicit row track definitions
    pub template_rows: Vec<TrackSizingFunction>,
    /// Named grid areas definition
    pub template_areas: GridTemplateAreas,
    /// Auto column track size (for implicitly created columns)
    pub auto_columns: TrackSizingFunction,
    /// Auto row track size (for implicitly created rows)
    pub auto_rows: TrackSizingFunction,
    /// Auto-placement algorithm direction
    pub auto_flow: GridAutoFlow,
    /// Gap between columns
    pub column_gap: f32,
    /// Gap between rows
    pub row_gap: f32,
}

/// Grid template areas definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplateAreas {
    /// Named areas as rows of strings (e.g., ["header header", "sidebar main"])
    pub rows: Vec<Vec<String>>,
    /// Map from area name to (row_start, column_start, row_end, column_end)
    pub named_areas: std::collections::HashMap<String, (usize, usize, usize, usize)>,
}

impl GridTemplateAreas {
    /// Create from CSS grid-template-areas value
    pub fn parse(value: &str) -> Self {
        let mut areas = Self::default();
        
        // Parse the string like: "header header" "sidebar main"
        let cleaned = value.replace('"', "");
        for line in cleaned.lines() {
            let row: Vec<String> = line
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            if !row.is_empty() {
                areas.rows.push(row);
            }
        }
        
        // Build named_areas map
        areas.build_named_areas();
        areas
    }

    /// Build the named_areas map from rows
    fn build_named_areas(&mut self) {
        self.named_areas.clear();
        
        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col_idx, name) in row.iter().enumerate() {
                if name == "." {
                    continue; // Empty cell
                }
                
                // Find the extent of this area
                let entry = self.named_areas
                    .entry(name.clone())
                    .or_insert((row_idx, col_idx, row_idx + 1, col_idx + 1));
                
                // Update bounds
                entry.2 = entry.2.max(row_idx + 1);
                entry.3 = entry.3.max(col_idx + 1);
            }
        }
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    /// Get the number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get area bounds by name
    pub fn get_area(&self, name: &str) -> Option<(usize, usize, usize, usize)> {
        self.named_areas.get(name).copied()
    }
}

/// A grid track (row or column)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridTrack {
    /// Track sizing function
    pub sizing: TrackSizingFunction,
    /// Base size (resolved minimum size)
    pub base_size: f32,
    /// Growth limit (maximum size for auto tracks)
    pub growth_limit: f32,
    /// Final calculated size
    pub final_size: f32,
    /// Whether this is an implicit track
    pub is_implicit: bool,
}

impl GridTrack {
    pub fn new(sizing: TrackSizingFunction, is_implicit: bool) -> Self {
        Self {
            sizing,
            base_size: 0.0,
            growth_limit: f32::INFINITY,
            final_size: 0.0,
            is_implicit,
        }
    }

    pub fn with_size(sizing: TrackSizingFunction, size: f32, is_implicit: bool) -> Self {
        Self {
            sizing,
            base_size: size,
            growth_limit: size,
            final_size: size,
            is_implicit,
        }
    }
}

/// Grid item with placement info
#[derive(Debug, Clone)]
pub struct GridItem {
    /// Index of the child box in the container
    pub child_index: usize,
    /// Placement info (from style or auto-placed)
    pub placement: GridPlacement,
    /// Resolved column start (0-indexed)
    pub resolved_col_start: usize,
    /// Resolved column end (exclusive)
    pub resolved_col_end: usize,
    /// Resolved row start (0-indexed)
    pub resolved_row_start: usize,
    /// Resolved row end (exclusive)
    pub resolved_row_end: usize,
    /// Minimum content width
    pub min_content_width: f32,
    /// Minimum content height
    pub min_content_height: f32,
    /// Maximum content width
    pub max_content_width: f32,
    /// Maximum content height
    pub max_content_height: f32,
}

impl GridItem {
    pub fn new(child_index: usize, placement: GridPlacement) -> Self {
        Self {
            child_index,
            placement,
            resolved_col_start: 0,
            resolved_col_end: 1,
            resolved_row_start: 0,
            resolved_row_end: 1,
            min_content_width: 0.0,
            min_content_height: 0.0,
            max_content_width: f32::INFINITY,
            max_content_height: f32::INFINITY,
        }
    }

    /// Get the column span
    pub fn column_span(&self) -> usize {
        self.resolved_col_end - self.resolved_col_start
    }

    /// Get the row span
    pub fn row_span(&self) -> usize {
        self.resolved_row_end - self.resolved_row_start
    }
}

/// Grid layout context
#[derive(Debug)]
pub struct GridContext {
    /// Container properties
    pub container: GridContainer,
    /// Column tracks
    pub columns: Vec<GridTrack>,
    /// Row tracks
    pub rows: Vec<GridTrack>,
    /// Grid items with placement
    pub items: Vec<GridItem>,
    /// Available width
    pub available_width: f32,
    /// Available height
    pub available_height: f32,
}

impl GridContext {
    pub fn new(container: GridContainer, available_size: crate::types::Size) -> Self {
        Self {
            available_width: available_size.width,
            available_height: available_size.height,
            container,
            columns: Vec::new(),
            rows: Vec::new(),
            items: Vec::new(),
        }
    }

    /// Get total width of all columns
    pub fn total_column_width(&self) -> f32 {
        self.columns.iter().map(|c| c.final_size).sum::<f32>()
    }

    /// Get total height of all rows
    pub fn total_row_height(&self) -> f32 {
        self.rows.iter().map(|r| r.final_size).sum::<f32>()
    }

    /// Get the position and size for an item
    pub fn get_item_rect(&self, item: &GridItem) -> Rect {
        let x: f32 = self.columns[..item.resolved_col_start]
            .iter()
            .map(|c| c.final_size)
            .sum::<f32>()
            + item.resolved_col_start as f32 * self.container.column_gap;
        
        let y: f32 = self.rows[..item.resolved_row_start]
            .iter()
            .map(|r| r.final_size)
            .sum::<f32>()
            + item.resolved_row_start as f32 * self.container.row_gap;
        
        let width: f32 = self.columns[item.resolved_col_start..item.resolved_col_end]
            .iter()
            .map(|c| c.final_size)
            .sum::<f32>()
            + (item.column_span() as f32 - 1.0) * self.container.column_gap;
        
        let height: f32 = self.rows[item.resolved_row_start..item.resolved_row_end]
            .iter()
            .map(|r| r.final_size)
            .sum::<f32>()
            + (item.row_span() as f32 - 1.0) * self.container.row_gap;
        
        Rect::new(x, y, width, height)
    }
}

/// Layout a grid container and its items
pub fn layout_grid_container(
    box_: &mut LayoutBox,
    bfc: &mut BlockFormattingContext,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    let style = box_.element()
        .map(|el| style_resolver(el))
        .unwrap_or_default();

    let container = GridContainer::from_style(&style, base_font_size, box_.dimensions.content.width);
    
    // Get container dimensions
    let content_width = box_.dimensions.content.width;
    let content_height = box_.dimensions.content.height;
    
    let available_size = crate::types::Size::new(content_width, content_height);
    let mut context = GridContext::new(container, available_size);

    // Collect grid items
    let grid_items: Vec<(usize, GridPlacement)> = box_.children
        .iter()
        .enumerate()
        .map(|(i, child)| {
            let item_style = child.element()
                .map(|el| style_resolver(el))
                .unwrap_or_default();
            let placement = GridPlacement::from_style(&item_style);
            (i, placement)
        })
        .collect();

    if grid_items.is_empty() {
        box_.is_laid_out = true;
        return;
    }

    // Initialize grid tracks based on template definitions
    initialize_tracks(&mut context, &grid_items);

    // Place items on the grid (resolve explicit placements and auto-place)
    place_items(&mut context, &grid_items);

    // Calculate track sizes (the grid sizing algorithm)
    calculate_track_sizes(&mut context, box_, style_resolver, base_font_size);

    // Position items within the grid
    position_grid_items(box_, &context, bfc);

    // Calculate final container size
    let total_width = context.total_column_width() 
        + (context.columns.len().saturating_sub(1)) as f32 * context.container.column_gap;
    let total_height = context.total_row_height()
        + (context.rows.len().saturating_sub(1)) as f32 * context.container.row_gap;
    
    box_.dimensions.content.width = total_width;
    box_.dimensions.content.height = total_height;
    box_.is_laid_out = true;
}

/// Initialize column and row tracks
fn initialize_tracks(context: &mut GridContext, items: &[(usize, GridPlacement)]) {
    // Create explicit column tracks
    for sizing in &context.container.template_columns {
        context.columns.push(GridTrack::new(sizing.clone(), false));
    }

    // Create explicit row tracks
    for sizing in &context.container.template_rows {
        context.rows.push(GridTrack::new(sizing.clone(), false));
    }

    // If template areas are defined, ensure we have enough tracks
    let area_cols = context.container.template_areas.column_count();
    let area_rows = context.container.template_areas.row_count();

    // Ensure minimum tracks based on template areas
    while context.columns.len() < area_cols {
        context.columns.push(GridTrack::new(context.container.auto_columns.clone(), false));
    }
    while context.rows.len() < area_rows {
        context.rows.push(GridTrack::new(context.container.auto_rows.clone(), false));
    }

    // Calculate minimum tracks needed based on explicit placements
    let mut max_explicit_col: usize = context.columns.len();
    let mut max_explicit_row: usize = context.rows.len();

    for (_, placement) in items {
        if placement.column_start.line_number > 0 {
            max_explicit_col = max_explicit_col.max(placement.column_start.line_number as usize);
        }
        if placement.column_end.line_number > 0 {
            max_explicit_col = max_explicit_col.max(placement.column_end.line_number as usize);
        }
        if placement.row_start.line_number > 0 {
            max_explicit_row = max_explicit_row.max(placement.row_start.line_number as usize);
        }
        if placement.row_end.line_number > 0 {
            max_explicit_row = max_explicit_row.max(placement.row_end.line_number as usize);
        }
    }

    // Add implicit tracks as needed
    while context.columns.len() < max_explicit_col {
        context.columns.push(GridTrack::new(context.container.auto_columns.clone(), true));
    }
    while context.rows.len() < max_explicit_row {
        context.rows.push(GridTrack::new(context.container.auto_rows.clone(), true));
    }

    // Ensure at least one track in each direction
    if context.columns.is_empty() {
        context.columns.push(GridTrack::new(context.container.auto_columns.clone(), true));
    }
    if context.rows.is_empty() {
        context.rows.push(GridTrack::new(context.container.auto_rows.clone(), true));
    }
}

/// Place items on the grid
fn place_items(context: &mut GridContext, items: &[(usize, GridPlacement)]) {
    // Track occupied cells
    let mut occupied: Vec<Vec<bool>> = vec![vec![false; 10]; 10]; // Will grow as needed

    for (child_index, placement) in items {
        let mut item = GridItem::new(*child_index, placement.clone());

        // Try to resolve named areas first
        let mut placement = placement.clone();
        
        if let Some(ref area_name) = placement.column_start.area_name {
            if let Some((row_start, col_start, row_end, col_end)) = 
                context.container.template_areas.get_area(area_name) {
                placement.row_start = GridLine::numbered(*row_start as i32 + 1);
                placement.column_start = GridLine::numbered(*col_start as i32 + 1);
                placement.row_end = GridLine::numbered(*row_end as i32 + 1);
                placement.column_end = GridLine::numbered(*col_end as i32 + 1);
            }
        }

        // Resolve explicit placements
        if placement.column_start.line_number > 0 {
            item.resolved_col_start = (placement.column_start.line_number - 1) as usize;
        }
        if placement.column_end.line_number > 0 {
            item.resolved_col_end = (placement.column_end.line_number - 1) as usize;
        } else if let Some(span) = placement.column_end.span {
            item.resolved_col_end = item.resolved_col_start + span as usize;
        }

        if placement.row_start.line_number > 0 {
            item.resolved_row_start = (placement.row_start.line_number - 1) as usize;
        }
        if placement.row_end.line_number > 0 {
            item.resolved_row_end = (placement.row_end.line_number - 1) as usize;
        } else if let Some(span) = placement.row_end.span {
            item.resolved_row_end = item.resolved_row_start + span as usize;
        }

        // Ensure we have enough tracks
        ensure_tracks(context, item.resolved_col_end, item.resolved_row_end);

        // Auto-placement for items without explicit placement
        if placement.is_auto() {
            auto_place_item(context, &mut item, &mut occupied);
        } else {
            // Mark cells as occupied
            mark_occupied(&mut occupied, &item);
        }

        context.items.push(item);
    }
}

/// Ensure we have enough tracks for the given indices
fn ensure_tracks(context: &mut GridContext, min_cols: usize, min_rows: usize) {
    while context.columns.len() < min_cols {
        context.columns.push(GridTrack::new(context.container.auto_columns.clone(), true));
    }
    while context.rows.len() < min_rows {
        context.rows.push(GridTrack::new(context.container.auto_rows.clone(), true));
    }

    // Grow occupied matrix if needed
}

/// Mark grid cells as occupied by an item
fn mark_occupied(occupied: &mut Vec<Vec<bool>>, item: &GridItem) {
    // Ensure the matrix is large enough
    while occupied.len() < item.resolved_row_end {
        occupied.push(vec![false; occupied.first().map(|r| r.len()).unwrap_or(10)]);
    }
    for row in occupied.iter_mut() {
        while row.len() < item.resolved_col_end {
            row.push(false);
        }
    }

    for row in item.resolved_row_start..item.resolved_row_end {
        for col in item.resolved_col_start..item.resolved_col_end {
            if row < occupied.len() && col < occupied[row].len() {
                occupied[row][col] = true;
            }
        }
    }
}

/// Auto-place an item using the auto-placement algorithm
fn auto_place_item(context: &mut GridContext, item: &mut GridItem, occupied: &mut Vec<Vec<bool>>) {
    let col_span = item.placement.column_span() as usize;
    let row_span = item.placement.row_span() as usize;

    if context.container.auto_flow.is_row() {
        // Search row by row
        for row in 0.. {
            for col in 0..context.columns.len() {
                if can_fit(occupied, col, row, col_span, row_span) {
                    item.resolved_col_start = col;
                    item.resolved_col_end = col + col_span;
                    item.resolved_row_start = row;
                    item.resolved_row_end = row + row_span;
                    ensure_tracks(context, col + col_span, row + row_span);
                    mark_occupied(occupied, item);
                    return;
                }
            }
        }
    } else {
        // Search column by column
        for col in 0.. {
            for row in 0..context.rows.len() {
                if can_fit(occupied, col, row, col_span, row_span) {
                    item.resolved_col_start = col;
                    item.resolved_col_end = col + col_span;
                    item.resolved_row_start = row;
                    item.resolved_row_end = row + row_span;
                    ensure_tracks(context, col + col_span, row + row_span);
                    mark_occupied(occupied, item);
                    return;
                }
            }
        }
    }
}

/// Check if an item can fit at the given position
fn can_fit(occupied: &[Vec<bool>], col: usize, row: usize, col_span: usize, row_span: usize) -> bool {
    for r in row..row + row_span {
        if r >= occupied.len() {
            continue; // Treat out-of-bounds as available
        }
        for c in col..col + col_span {
            if c < occupied[r].len() && occupied[r][c] {
                return false;
            }
        }
    }
    true
}

/// Calculate track sizes using the grid sizing algorithm
fn calculate_track_sizes(
    context: &mut GridContext,
    box_: &LayoutBox,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) {
    // Step 1: Initialize base sizes
    for track in &mut context.columns {
        track.base_size = match &track.sizing {
            TrackSizingFunction::Length(l) => *l,
            TrackSizingFunction::Percentage(p) => context.available_width * p / 100.0,
            _ => 0.0,
        };
        track.final_size = track.base_size;
    }

    for track in &mut context.rows {
        track.base_size = match &track.sizing {
            TrackSizingFunction::Length(l) => *l,
            TrackSizingFunction::Percentage(p) => {
                if context.available_height > 0.0 {
                    context.available_height * p / 100.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        track.final_size = track.base_size;
    }

    // Step 2: Calculate min/max content contributions from items
    for item in &context.items {
        if let Some(child) = box_.children.get(item.child_index) {
            let (min_width, max_width, min_height, max_height) = 
                calculate_content_sizes(child, style_resolver, base_font_size);
            
            // Distribute contributions to tracks
            distribute_size_to_tracks(&mut context.columns, item.resolved_col_start, item.resolved_col_end, min_width, max_width);
            distribute_size_to_tracks(&mut context.rows, item.resolved_row_start, item.resolved_row_end, min_height, max_height);
        }
    }

    // Step 3: Resolve flexible tracks (fr units)
    resolve_flexible_tracks(&mut context.columns, context.available_width, context.container.column_gap);
    resolve_flexible_tracks(&mut context.rows, context.available_height, context.container.row_gap);

    // Step 4: Apply minmax constraints
    apply_minmax_constraints(context, box_, style_resolver, base_font_size);
}

/// Calculate content sizes for a grid item
fn calculate_content_sizes(
    child: &LayoutBox,
    style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    base_font_size: f32,
) -> (f32, f32, f32, f32) {
    let style = child.element()
        .map(|el| style_resolver(el))
        .unwrap_or_default();

    // Estimate content sizes
    let min_width = if style.min_width.is_auto() {
        0.0
    } else {
        style.min_width.to_pt(base_font_size)
    };

    let max_width = if style.max_width.is_auto() {
        f32::INFINITY
    } else {
        style.max_width.to_pt(base_font_size)
    };

    let min_height = if style.min_height.is_auto() {
        0.0
    } else {
        style.min_height.to_pt(base_font_size)
    };

    let max_height = if style.max_height.is_auto() {
        f32::INFINITY
    } else {
        style.max_height.to_pt(base_font_size)
    };

    // Estimate content size based on children
    let content_width = if child.children.is_empty() {
        child.text_content.as_ref()
            .map(|text| text.len() as f32 * 6.0) // Approximate char width
            .unwrap_or(50.0)
    } else {
        child.children.iter()
            .map(|c| c.dimensions.content.width)
            .fold(0.0, f32::max)
    };

    let content_height = if child.children.is_empty() {
        style.font_size.to_pt(base_font_size)
    } else {
        child.children.iter()
            .map(|c| c.dimensions.content.height)
            .sum()
    };

    (min_width.max(content_width * 0.5), max_width.max(content_width), 
     min_height.max(content_height * 0.5), max_height.max(content_height))
}

/// Distribute size contributions to tracks
fn distribute_size_to_tracks(
    tracks: &mut [GridTrack],
    start: usize,
    end: usize,
    min_size: f32,
    _max_size: f32,
) {
    let span = end - start;
    if span == 0 {
        return;
    }

    let min_per_track = min_size / span as f32;

    for i in start..end {
        if i < tracks.len() {
            tracks[i].base_size = tracks[i].base_size.max(min_per_track);
            tracks[i].final_size = tracks[i].base_size;
        }
    }
}

/// Resolve flexible tracks (fr units)
fn resolve_flexible_tracks(tracks: &mut [GridTrack], available_size: f32, gap: f32) {
    // Calculate total gap space
    let gap_space = if tracks.len() > 1 {
        (tracks.len() - 1) as f32 * gap
    } else {
        0.0
    };

    let available = (available_size - gap_space).max(0.0);
    
    // Calculate fixed space used
    let fixed_used: f32 = tracks.iter()
        .filter(|t| !t.sizing.is_flexible())
        .map(|t| t.final_size)
        .sum();

    let flexible_space = (available - fixed_used).max(0.0);

    // Calculate total flex factor
    let total_flex: f32 = tracks.iter()
        .filter_map(|t| t.sizing.flex_factor())
        .sum();

    if total_flex > 0.0 {
        let flex_unit = flexible_space / total_flex;

        for track in tracks.iter_mut() {
            if let Some(flex) = track.sizing.flex_factor() {
                track.final_size = (flex_unit * flex).max(track.base_size);
            }
        }
    }
}

/// Apply minmax constraints
fn apply_minmax_constraints(
    context: &mut GridContext,
    _box_: &LayoutBox,
    _style_resolver: &dyn Fn(&crate::html::Element) -> ComputedStyle,
    _base_font_size: f32,
) {
    // Apply minmax constraints to columns
    for track in &mut context.columns {
        if let TrackSizingFunction::MinMax(min, max) = &track.sizing {
            let min_val = match min.as_ref() {
                TrackSizingFunction::Length(l) => *l,
                TrackSizingFunction::Percentage(p) => context.available_width * p / 100.0,
                TrackSizingFunction::MinContent => track.base_size * 0.5,
                TrackSizingFunction::MaxContent => track.base_size,
                _ => track.base_size,
            };

            let max_val = match max.as_ref() {
                TrackSizingFunction::Length(l) => *l,
                TrackSizingFunction::Percentage(p) => context.available_width * p / 100.0,
                TrackSizingFunction::MinContent => track.base_size,
                TrackSizingFunction::MaxContent => f32::INFINITY,
                TrackSizingFunction::Auto => f32::INFINITY,
                _ => f32::INFINITY,
            };

            track.final_size = track.final_size.max(min_val).min(max_val);
        }
    }

    // Apply minmax constraints to rows
    for track in &mut context.rows {
        if let TrackSizingFunction::MinMax(min, max) = &track.sizing {
            let min_val = match min.as_ref() {
                TrackSizingFunction::Length(l) => *l,
                TrackSizingFunction::Percentage(p) => {
                    if context.available_height > 0.0 {
                        context.available_height * p / 100.0
                    } else {
                        0.0
                    }
                }
                TrackSizingFunction::MinContent => track.base_size * 0.5,
                TrackSizingFunction::MaxContent => track.base_size,
                _ => track.base_size,
            };

            let max_val = match max.as_ref() {
                TrackSizingFunction::Length(l) => *l,
                TrackSizingFunction::Percentage(p) => {
                    if context.available_height > 0.0 {
                        context.available_height * p / 100.0
                    } else {
                        f32::INFINITY
                    }
                }
                TrackSizingFunction::MinContent => track.base_size,
                TrackSizingFunction::MaxContent => f32::INFINITY,
                TrackSizingFunction::Auto => f32::INFINITY,
                _ => f32::INFINITY,
            };

            track.final_size = track.final_size.max(min_val).min(max_val);
        }
    }
}

/// Position grid items within the container
fn position_grid_items(
    box_: &mut LayoutBox,
    context: &GridContext,
    bfc: &BlockFormattingContext,
) {
    for item in &context.items {
        if let Some(child) = box_.children.get_mut(item.child_index) {
            let rect = context.get_item_rect(item);
            
            // Position relative to containing block
            child.dimensions.content.x = bfc.containing_block.x + rect.x;
            child.dimensions.content.y = bfc.containing_block.y + rect.y;
            child.dimensions.content.width = rect.width;
            child.dimensions.content.height = rect.height;
            
            child.is_laid_out = true;
        }
    }
}

impl GridContainer {
    /// Create from computed style
    pub fn from_style(style: &ComputedStyle, base_font_size: f32, container_width: f32) -> Self {
        let mut container = Self::default();

        // Parse grid-template-columns
        if !style.grid_template_columns.is_empty() {
            container.template_columns = parse_track_list(&style.grid_template_columns, base_font_size, container_width);
        }

        // Parse grid-template-rows
        if !style.grid_template_rows.is_empty() {
            container.template_rows = parse_track_list(&style.grid_template_rows, base_font_size, container_width);
        }

        // Parse grid-template-areas
        if !style.grid_template_areas.is_empty() {
            container.template_areas = GridTemplateAreas::parse(&style.grid_template_areas);
        }

        // Parse grid-auto-columns
        if !style.grid_auto_columns.is_empty() {
            container.auto_columns = parse_track_size(&style.grid_auto_columns, base_font_size, container_width);
        }

        // Parse grid-auto-rows
        if !style.grid_auto_rows.is_empty() {
            container.auto_rows = parse_track_size(&style.grid_auto_rows, base_font_size, container_width);
        }

        // Parse grid-auto-flow
        container.auto_flow = style.grid_auto_flow;

        // Parse gap properties
        container.column_gap = style.column_gap.to_pt_with_container(base_font_size, container_width);
        container.row_gap = style.row_gap.to_pt_with_container(base_font_size, container_width);

        container
    }
}

impl GridPlacement {
    /// Create from computed style
    pub fn from_style(style: &ComputedStyle) -> Self {
        Self {
            column_start: style.grid_column_start.clone(),
            column_end: style.grid_column_end.clone(),
            row_start: style.grid_row_start.clone(),
            row_end: style.grid_row_end.clone(),
        }
    }
}

/// Parse a track list (e.g., "100px 1fr auto")
fn parse_track_list(value: &str, base_font_size: f32, container_width: f32) -> Vec<TrackSizingFunction> {
    let mut tracks = Vec::new();
    
    for part in value.split_whitespace() {
        tracks.push(parse_track_size(part, base_font_size, container_width));
    }

    tracks
}

/// Parse a single track size
fn parse_track_size(value: &str, base_font_size: f32, container_width: f32) -> TrackSizingFunction {
    let value = value.trim().to_lowercase();

    // Check for minmax()
    if value.starts_with("minmax(") && value.ends_with(')') {
        let inner = &value[7..value.len()-1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let min = parse_track_size(parts[0], base_font_size, container_width);
            let max = parse_track_size(parts[1], base_font_size, container_width);
            return TrackSizingFunction::MinMax(Box::new(min), Box::new(max));
        }
    }

    // Check for fit-content()
    if value.starts_with("fit-content(") && value.ends_with(')') {
        let inner = &value[12..value.len()-1];
        if let Ok(len) = parse_length_value(inner, base_font_size, container_width) {
            return TrackSizingFunction::FitContent(len);
        }
    }

    // Check for fr unit
    if let Some(num_str) = value.strip_suffix("fr") {
        if let Ok(num) = num_str.trim().parse::<f32>() {
            return TrackSizingFunction::Flex(num);
        }
    }

    // Check for percentage
    if value.ends_with('%') {
        if let Ok(pct) = value.trim_end_matches('%').trim().parse::<f32>() {
            return TrackSizingFunction::Percentage(pct);
        }
    }

    // Check for keywords
    match value.as_str() {
        "auto" => return TrackSizingFunction::Auto,
        "min-content" => return TrackSizingFunction::MinContent,
        "max-content" => return TrackSizingFunction::MaxContent,
        _ => {}
    }

    // Try to parse as length
    if let Ok(len) = parse_length_value(&value, base_font_size, container_width) {
        return TrackSizingFunction::Length(len);
    }

    TrackSizingFunction::Auto
}

/// Parse a length value string to points
fn parse_length_value(value: &str, base_font_size: f32, container_width: f32) -> Result<f32, ()> {
    let value = value.trim();

    // Handle pixels
    if let Some(num_str) = value.strip_suffix("px") {
        return num_str.trim().parse::<f32>().map(|v| v * 0.75).map_err(|_| ());
    }

    // Handle points
    if let Some(num_str) = value.strip_suffix("pt") {
        return num_str.trim().parse::<f32>().map_err(|_| ());
    }

    // Handle em
    if let Some(num_str) = value.strip_suffix("em") {
        return num_str.trim().parse::<f32>().map(|v| v * base_font_size).map_err(|_| ());
    }

    // Handle rem (simplified - same as em for now)
    if let Some(num_str) = value.strip_suffix("rem") {
        return num_str.trim().parse::<f32>().map(|v| v * base_font_size).map_err(|_| ());
    }

    // Handle percentage
    if value.ends_with('%') {
        return value.trim_end_matches('%').trim()
            .parse::<f32>()
            .map(|v| v * container_width / 100.0)
            .map_err(|_| ());
    }

    // Try plain number as pixels
    value.parse::<f32>().map(|v| v * 0.75).map_err(|_| ())
}

/// Parse a grid line value
fn parse_grid_line(value: &str) -> GridLine {
    let value = value.trim();

    // Check for span
    if value.starts_with("span") {
        let num_str = value[4..].trim();
        if let Ok(n) = num_str.parse::<i32>() {
            return GridLine::span(n);
        }
        return GridLine::span(1);
    }

    // Check for number
    if let Ok(n) = value.parse::<i32>() {
        if n > 0 {
            return GridLine::numbered(n);
        }
    }

    // Check for auto
    if value.eq_ignore_ascii_case("auto") {
        return GridLine::auto();
    }

    // Treat as named area
    GridLine {
        line_number: 0,
        area_name: Some(value.to_string()),
        span: None,
    }
}

/// Parse grid-auto-flow value
pub fn parse_grid_auto_flow(value: &str) -> GridAutoFlow {
    let value = value.trim().to_lowercase();
    
    if value.contains("column") {
        if value.contains("dense") {
            GridAutoFlow::ColumnDense
        } else {
            GridAutoFlow::Column
        }
    } else {
        if value.contains("dense") {
            GridAutoFlow::RowDense
        } else {
            GridAutoFlow::Row
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_sizing_function() {
        assert!(TrackSizingFunction::Length(100.0).is_fixed());
        assert!(TrackSizingFunction::Percentage(50.0).is_fixed());
        assert!(!TrackSizingFunction::Flex(1.0).is_fixed());
        assert!(TrackSizingFunction::Flex(1.0).is_flexible());
        assert_eq!(TrackSizingFunction::Flex(2.0).flex_factor(), Some(2.0));
    }

    #[test]
    fn test_grid_auto_flow() {
        assert!(GridAutoFlow::Row.is_row());
        assert!(!GridAutoFlow::Column.is_row());
        assert!(GridAutoFlow::RowDense.is_dense());
        assert!(!GridAutoFlow::Column.is_dense());
    }

    #[test]
    fn test_grid_line() {
        let auto = GridLine::auto();
        assert!(auto.is_auto());

        let numbered = GridLine::numbered(3);
        assert_eq!(numbered.line_number, 3);

        let span = GridLine::span(2);
        assert_eq!(span.span, Some(2));
    }

    #[test]
    fn test_grid_placement() {
        let placement = GridPlacement {
            column_start: GridLine::numbered(1),
            column_end: GridLine::numbered(3),
            row_start: GridLine::numbered(1),
            row_end: GridLine::span(2),
        };

        assert_eq!(placement.column_span(), 2);
        assert_eq!(placement.row_span(), 2);
    }

    #[test]
    fn test_grid_template_areas_parse() {
        let areas = GridTemplateAreas::parse(r#""header header" "sidebar main""#);
        
        assert_eq!(areas.row_count(), 2);
        assert_eq!(areas.column_count(), 2);
        
        let header = areas.get_area("header");
        assert!(header.is_some());
        let (r1, c1, r2, c2) = header.unwrap();
        assert_eq!(r1, 0);
        assert_eq!(c1, 0);
        assert_eq!(r2, 1);
        assert_eq!(c2, 2);
    }

    #[test]
    fn test_parse_track_size() {
        assert!(matches!(parse_track_size("100px", 12.0, 500.0), TrackSizingFunction::Length(_)));
        assert!(matches!(parse_track_size("1fr", 12.0, 500.0), TrackSizingFunction::Flex(_)));
        assert!(matches!(parse_track_size("auto", 12.0, 500.0), TrackSizingFunction::Auto));
        assert!(matches!(parse_track_size("min-content", 12.0, 500.0), TrackSizingFunction::MinContent));
        assert!(matches!(parse_track_size("max-content", 12.0, 500.0), TrackSizingFunction::MaxContent));
        
        let minmax = parse_track_size("minmax(100px, 1fr)", 12.0, 500.0);
        assert!(matches!(minmax, TrackSizingFunction::MinMax(_, _)));
    }

    #[test]
    fn test_parse_grid_line() {
        let line = parse_grid_line("3");
        assert_eq!(line.line_number, 3);

        let span = parse_grid_line("span 2");
        assert_eq!(span.span, Some(2));

        let auto = parse_grid_line("auto");
        assert!(auto.is_auto());

        let named = parse_grid_line("header");
        assert_eq!(named.area_name, Some("header".to_string()));
    }

    #[test]
    fn test_parse_grid_auto_flow() {
        assert_eq!(parse_grid_auto_flow("row"), GridAutoFlow::Row);
        assert_eq!(parse_grid_auto_flow("column"), GridAutoFlow::Column);
        assert_eq!(parse_grid_auto_flow("row dense"), GridAutoFlow::RowDense);
        assert_eq!(parse_grid_auto_flow("column dense"), GridAutoFlow::ColumnDense);
    }

    #[test]
    fn test_grid_context() {
        let container = GridContainer::default();
        let context = GridContext::new(container, crate::types::Size::new(500.0, 400.0));
        
        assert_eq!(context.available_width, 500.0);
        assert_eq!(context.available_height, 400.0);
    }

    #[test]
    fn test_grid_track() {
        let track = GridTrack::new(TrackSizingFunction::Length(100.0), false);
        assert_eq!(track.base_size, 0.0);
        assert!(!track.is_implicit);

        let sized = GridTrack::with_size(TrackSizingFunction::Auto, 50.0, true);
        assert_eq!(sized.final_size, 50.0);
        assert!(sized.is_implicit);
    }

    #[test]
    fn test_can_fit() {
        let occupied = vec![
            vec![false, false, false],
            vec![false, true, false],
            vec![false, false, false],
        ];

        assert!(can_fit(&occupied, 0, 0, 1, 1));
        assert!(!can_fit(&occupied, 1, 1, 1, 1));
        assert!(can_fit(&occupied, 2, 0, 2, 1));
    }
}

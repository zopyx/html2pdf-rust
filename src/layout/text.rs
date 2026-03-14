//! Text Layout
//!
//! Handles text shaping, line breaking, and white space processing.
//! Implements Unicode line breaking and basic text metrics.

use crate::types::Rect;
use crate::layout::box_model::{LayoutBox, Dimensions};
use crate::layout::style::{ComputedStyle, WhiteSpace, TextTransform};
use unicode_width::UnicodeWidthChar;

/// Text metrics for a font at a given size
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    /// Font size in points
    pub font_size: f32,
    /// Ascent (distance from baseline to top)
    pub ascent: f32,
    /// Descent (distance from baseline to bottom)
    pub descent: f32,
    /// Line height
    pub line_height: f32,
    /// Average character width (approximate)
    pub avg_char_width: f32,
    /// Space width
    pub space_width: f32,
}

impl TextMetrics {
    pub fn new(font_size: f32) -> Self {
        // Default metrics based on font size
        // In a full implementation, these would come from font tables
        let ascent = font_size * 0.8;
        let descent = font_size * 0.2;
        let line_height = font_size * 1.2;
        let avg_char_width = font_size * 0.5;
        let space_width = font_size * 0.25;

        Self {
            font_size,
            ascent,
            descent,
            line_height,
            avg_char_width,
            space_width,
        }
    }

    /// Get the height of a line
    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    /// Get the baseline offset from the top of the line
    pub fn baseline_offset(&self) -> f32 {
        self.ascent
    }
}

/// A line of text containing one or more text fragments
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    /// Text fragments in this line
    pub fragments: Vec<TextFragment>,
    /// Line width
    pub width: f32,
    /// Line height
    pub height: f32,
    /// Baseline offset from line top
    pub baseline: f32,
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Add a fragment to this line
    pub fn add_fragment(&mut self, fragment: TextFragment, metrics: &TextMetrics) {
        self.width += fragment.width;
        self.height = self.height.max(metrics.line_height);
        self.baseline = self.baseline.max(metrics.baseline_offset());
        self.fragments.push(fragment);
    }

    /// Get the remaining width in this line
    pub fn remaining_width(&self, max_width: f32) -> f32 {
        (max_width - self.width).max(0.0)
    }
}

/// A fragment of text with uniform styling
#[derive(Debug, Clone, PartialEq)]
pub struct TextFragment {
    /// The text content
    pub text: String,
    /// Width of the text
    pub width: f32,
    /// Height of the text
    pub height: f32,
    /// Position within the line
    pub x: f32,
    /// Font size
    pub font_size: f32,
    /// Whether this fragment ends with a soft wrap opportunity
    pub has_wrap_opportunity: bool,
}

/// Line breaking algorithm
pub struct LineBreaker {
    max_width: f32,
    current_line: Line,
    lines: Vec<Line>,
    metrics: TextMetrics,
}

impl LineBreaker {
    pub fn new(max_width: f32, font_size: f32) -> Self {
        Self {
            max_width,
            current_line: Line::new(),
            lines: Vec::new(),
            metrics: TextMetrics::new(font_size),
        }
    }

    /// Add text to be laid out
    pub fn add_text(&mut self, text: &str, white_space: WhiteSpace) {
        match white_space {
            WhiteSpace::Pre => self.add_preformatted_text(text),
            WhiteSpace::PreWrap => self.add_pre_wrap_text(text),
            WhiteSpace::PreLine => self.add_pre_line_text(text),
            WhiteSpace::Nowrap => self.add_no_wrap_text(text),
            WhiteSpace::Normal => self.add_normal_text(text),
        }
    }

    /// Add text with normal white space handling
    fn add_normal_text(&mut self, text: &str) {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            let word_width = self.measure_text(word);
            
            // Check if we need to break before this word
            if !self.current_line.is_empty() {
                let space_width = self.metrics.space_width;
                if self.current_line.width + space_width + word_width > self.max_width {
                    self.finish_line();
                } else {
                    // Add space before word (except for first word)
                    self.add_fragment(" ", space_width);
                }
            } else if word_width > self.max_width {
                // Word is too long, need to break it
                self.break_word(word);
                continue;
            }
            
            self.add_fragment(word, word_width);
        }
    }

    /// Add preformatted text (preserves all whitespace)
    fn add_preformatted_text(&mut self, text: &str) {
        for line in text.lines() {
            let line_width = self.measure_text(line);
            
            if line_width > self.max_width {
                // Break long lines in preformatted text
                self.break_word(line);
            } else {
                self.add_fragment(line, line_width);
            }
            
            // Force line break after each line
            self.finish_line();
        }
    }

    /// Add pre-wrap text (preserves whitespace but wraps)
    fn add_pre_wrap_text(&mut self, text: &str) {
        // Similar to pre but wraps at max_width
        let mut current = String::new();
        let mut current_width = 0.0;
        
        for ch in text.chars() {
            if ch == '\n' {
                if !current.is_empty() {
                    self.add_fragment(&current, current_width);
                }
                self.finish_line();
                current.clear();
                current_width = 0.0;
            } else {
                let ch_width = self.measure_char(ch);
                
                if current_width + ch_width > self.max_width && !current.is_empty() {
                    self.add_fragment(&current, current_width);
                    self.finish_line();
                    current.clear();
                    current_width = 0.0;
                }
                
                current.push(ch);
                current_width += ch_width;
            }
        }
        
        if !current.is_empty() {
            self.add_fragment(&current, current_width);
        }
    }

    /// Add pre-line text (collapses spaces, preserves newlines)
    fn add_pre_line_text(&mut self, text: &str) {
        let mut in_whitespace = false;
        let mut current = String::new();
        
        for ch in text.chars() {
            if ch == '\n' {
                if !current.is_empty() {
                    let width = self.measure_text(&current);
                    self.add_fragment(&current, width);
                    current.clear();
                }
                self.finish_line();
                in_whitespace = false;
            } else if ch.is_whitespace() {
                if !in_whitespace && !current.is_empty() {
                    // Collapse multiple spaces
                    current.push(' ');
                }
                in_whitespace = true;
            } else {
                in_whitespace = false;
                current.push(ch);
            }
        }
        
        if !current.is_empty() {
            let width = self.measure_text(&current);
            self.add_fragment(&current, width);
        }
    }

    /// Add no-wrap text (all on one line)
    fn add_no_wrap_text(&mut self, text: &str) {
        let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let width = self.measure_text(&collapsed);
        self.add_fragment(&collapsed, width);
    }

    /// Break a long word at character boundaries
    fn break_word(&mut self, word: &str) {
        let mut current = String::new();
        let mut current_width = 0.0;
        
        for ch in word.chars() {
            let ch_width = self.measure_char(ch);
            
            if current_width + ch_width > self.max_width && !current.is_empty() {
                self.add_fragment(&current, current_width);
                self.finish_line();
                current.clear();
                current_width = 0.0;
            }
            
            current.push(ch);
            current_width += ch_width;
        }
        
        if !current.is_empty() {
            self.add_fragment(&current, current_width);
        }
    }

    /// Add a fragment to the current line
    fn add_fragment(&mut self, text: &str, width: f32) {
        let fragment = TextFragment {
            text: text.to_string(),
            width,
            height: self.metrics.line_height,
            x: self.current_line.width,
            font_size: self.metrics.font_size,
            has_wrap_opportunity: text.ends_with(' ') || text.ends_with('-'),
        };
        
        self.current_line.add_fragment(fragment, &self.metrics);
    }

    /// Finish the current line and start a new one
    fn finish_line(&mut self) {
        if !self.current_line.is_empty() {
            // Calculate final positions
            let line_height = self.current_line.height;
            for fragment in &mut self.current_line.fragments {
                fragment.height = line_height;
            }
            
            self.lines.push(self.current_line.clone());
            self.current_line = Line::new();
        }
    }

    /// Measure text width
    fn measure_text(&self, text: &str) -> f32 {
        text.chars().map(|ch| self.measure_char(ch)).sum()
    }

    /// Measure a single character
    fn measure_char(&self, ch: char) -> f32 {
        if ch == ' ' {
            self.metrics.space_width
        } else {
            // Use Unicode width for character width estimation
            let width = ch.width().unwrap_or(1) as f32;
            width * self.metrics.avg_char_width
        }
    }

    /// Complete layout and return the lines
    pub fn finish(mut self) -> Vec<Line> {
        self.finish_line();
        self.lines
    }

    /// Get the current metrics
    pub fn metrics(&self) -> &TextMetrics {
        &self.metrics
    }
}

/// Text layout engine
pub struct TextLayout {
    metrics: TextMetrics,
}

impl TextLayout {
    pub fn new(font_size: f32) -> Self {
        Self {
            metrics: TextMetrics::new(font_size),
        }
    }

    /// Layout text within a given width
    pub fn layout_text(
        &self,
        text: &str,
        max_width: f32,
        style: &ComputedStyle,
    ) -> Vec<Line> {
        let processed_text = self.apply_text_transform(text, style.text_transform);
        
        let mut breaker = LineBreaker::new(max_width, self.metrics.font_size);
        breaker.add_text(&processed_text, style.white_space);
        breaker.finish()
    }

    /// Measure text without breaking lines
    pub fn measure_text(&self, text: &str) -> (f32, f32) {
        let width: f32 = text.chars()
            .map(|ch| {
                if ch == ' ' {
                    self.metrics.space_width
                } else {
                    ch.width().unwrap_or(1) as f32 * self.metrics.avg_char_width
                }
            })
            .sum();
        
        (width, self.metrics.line_height)
    }

    /// Apply text transformation
    fn apply_text_transform(&self, text: &str, transform: TextTransform) -> String {
        match transform {
            TextTransform::None => text.to_string(),
            TextTransform::Uppercase => text.to_uppercase(),
            TextTransform::Lowercase => text.to_lowercase(),
            TextTransform::Capitalize => {
                text.split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }

    /// Collapse white space according to CSS rules
    pub fn collapse_whitespace(&self, text: &str, white_space: WhiteSpace) -> String {
        match white_space {
            WhiteSpace::Pre | WhiteSpace::PreWrap => text.to_string(),
            WhiteSpace::Normal | WhiteSpace::PreLine => {
                // Collapse sequences of whitespace to a single space
                text.split_whitespace().collect::<Vec<_>>().join(" ")
            }
            WhiteSpace::Nowrap => {
                text.split_whitespace().collect::<Vec<_>>().join(" ")
            }
        }
    }

    /// Get text metrics
    pub fn metrics(&self) -> &TextMetrics {
        &self.metrics
    }
}

/// Position text fragments within a line based on text-align
pub fn align_line(line: &mut Line, max_width: f32, text_align: crate::layout::style::TextAlign) {
    match text_align {
        crate::layout::style::TextAlign::Left => {
            // Left-aligned: fragments already at correct position
        }
        crate::layout::style::TextAlign::Center => {
            let offset = (max_width - line.width) / 2.0;
            for fragment in &mut line.fragments {
                fragment.x += offset;
            }
        }
        crate::layout::style::TextAlign::Right => {
            let offset = max_width - line.width;
            for fragment in &mut line.fragments {
                fragment.x += offset;
            }
        }
        crate::layout::style::TextAlign::Justify => {
            // Simple justification: distribute space evenly
            let fragment_count = line.fragments.len();
            if fragment_count > 1 {
                let extra_space = max_width - line.width;
                let gaps = (fragment_count - 1) as f32;
                let space_per_gap = extra_space / gaps;
                
                let mut x_offset = 0.0;
                for (i, fragment) in line.fragments.iter_mut().enumerate() {
                    fragment.x += x_offset;
                    x_offset += space_per_gap;
                    // Don't add extra space after the last fragment
                    if i >= fragment_count - 1 {
                        break;
                    }
                }
            }
        }
    }
}

/// Calculate the bounding box for laid out text
pub fn calculate_text_bounds(lines: &[Line], container_x: f32, container_y: f32) -> Rect {
    if lines.is_empty() {
        return Rect::new(container_x, container_y, 0.0, 0.0);
    }

    let max_width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
    let total_height: f32 = lines.iter().map(|l| l.height).sum();

    Rect::new(container_x, container_y, max_width, total_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_metrics() {
        let metrics = TextMetrics::new(12.0);
        assert_eq!(metrics.font_size, 12.0);
        assert!(metrics.line_height > metrics.font_size);
    }

    #[test]
    fn test_line_breaker_normal() {
        let mut breaker = LineBreaker::new(100.0, 12.0);
        breaker.add_text("Hello world this is a test", WhiteSpace::Normal);
        let lines = breaker.finish();
        
        // Should break into multiple lines based on width
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_line_breaker_nowrap() {
        let mut breaker = LineBreaker::new(50.0, 12.0);
        breaker.add_text("Hello world", WhiteSpace::Nowrap);
        let lines = breaker.finish();
        
        // Should be a single line
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_text_layout() {
        let layout = TextLayout::new(12.0);
        let style = ComputedStyle::default();
        
        let lines = layout.layout_text("Hello world", 100.0, &style);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_measure_text() {
        let layout = TextLayout::new(12.0);
        let (width, height) = layout.measure_text("Hello");
        
        assert!(width > 0.0);
        assert!(height > 0.0);
    }

    #[test]
    fn test_collapse_whitespace() {
        let layout = TextLayout::new(12.0);
        
        let result = layout.collapse_whitespace("Hello   world", WhiteSpace::Normal);
        assert_eq!(result, "Hello world");
        
        let result = layout.collapse_whitespace("Hello   world", WhiteSpace::Pre);
        assert_eq!(result, "Hello   world");
    }
}

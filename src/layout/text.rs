//! Text Layout
//!
//! Handles text shaping, line breaking, and white space processing.
//! Implements Unicode line breaking and basic text metrics.

use crate::types::Rect;
use crate::layout::style::{
    ComputedStyle, WhiteSpace, TextTransform, TextAlign, WordWrap
};
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
    /// X-height (height of lowercase 'x')
    pub x_height: f32,
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
        let x_height = font_size * 0.5;

        Self {
            font_size,
            ascent,
            descent,
            line_height,
            avg_char_width,
            space_width,
            x_height,
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

/// Word breaking behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WordBreak {
    Normal,
    BreakAll,
    KeepAll,
}

/// Overflow wrap behavior  
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverflowWrap {
    Normal,
    BreakWord,
    Anywhere,
}

/// Vertical alignment for inline elements
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VerticalAlign {
    Baseline,
    Top,
    Bottom,
    Middle,
    Sub,
    Super,
    TextTop,
    TextBottom,
    Length(f32),
    Percent(f32),
}

impl Default for VerticalAlign {
    fn default() -> Self {
        VerticalAlign::Baseline
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
    /// Available width (for alignment)
    pub available_width: f32,
}

impl Line {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_available_width(width: f32) -> Self {
        Self {
            available_width: width,
            ..Default::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty()
    }

    /// Add a fragment to this line
    pub fn add_fragment(&mut self, fragment: TextFragment, metrics: &TextMetrics) {
        self.width += fragment.width;
        self.height = self.height.max(fragment.height);
        self.baseline = self.baseline.max(metrics.baseline_offset());
        self.fragments.push(fragment);
    }

    /// Get the remaining width in this line
    pub fn remaining_width(&self, max_width: f32) -> f32 {
        (max_width - self.width).max(0.0)
    }

    /// Apply text alignment
    pub fn apply_alignment(&mut self, align: TextAlign) {
        if self.fragments.is_empty() {
            return;
        }

        let remaining_space = self.available_width - self.width;
        
        match align {
            TextAlign::Left => {
                // Already left-aligned
            }
            TextAlign::Center => {
                let offset = remaining_space / 2.0;
                for fragment in &mut self.fragments {
                    fragment.x += offset;
                }
            }
            TextAlign::Right => {
                let offset = remaining_space;
                for fragment in &mut self.fragments {
                    fragment.x += offset;
                }
            }
            TextAlign::Justify => {
                // Only justify if there's space and more than one fragment
                if remaining_space > 0.0 && self.fragments.len() > 1 {
                    // Count word gaps (simplified - assumes each fragment is a word)
                    let gaps = self.fragments.len().saturating_sub(1) as f32;
                    let fragment_count = self.fragments.len();
                    if gaps > 0.0 {
                        let space_per_gap = remaining_space / gaps;
                        let mut x_offset = 0.0;
                        for (i, fragment) in self.fragments.iter_mut().enumerate() {
                            fragment.x += x_offset;
                            if i < fragment_count - 1 {
                                x_offset += space_per_gap;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Apply vertical alignment to fragments
    pub fn apply_vertical_align(&mut self, line_height: f32, metrics: &TextMetrics) {
        let line_baseline = self.baseline;
        
        for fragment in &mut self.fragments {
            let offset = match fragment.vertical_align {
                VerticalAlign::Baseline => 0.0,
                VerticalAlign::Top => -(line_baseline - fragment.height),
                VerticalAlign::Bottom => -(line_height - line_baseline),
                VerticalAlign::Middle => -(line_height / 2.0 - fragment.height / 2.0),
                VerticalAlign::Sub => metrics.x_height * 0.5,
                VerticalAlign::Super => -(metrics.x_height * 0.5),
                VerticalAlign::TextTop => -(line_baseline - metrics.ascent),
                VerticalAlign::TextBottom => -(metrics.descent),
                VerticalAlign::Length(l) => -l,
                VerticalAlign::Percent(p) => -(line_height * p / 100.0),
            };
            fragment.y_offset = offset;
            // Adjust height to accommodate offset
            let new_top = offset.min(0.0);
            let new_bottom = (fragment.height + offset).max(fragment.height);
            self.height = self.height.max(new_bottom - new_top);
        }
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
    /// Vertical offset for alignment
    pub y_offset: f32,
    /// Font size
    pub font_size: f32,
    /// Whether this fragment ends with a soft wrap opportunity
    pub has_wrap_opportunity: bool,
    /// Vertical alignment
    pub vertical_align: VerticalAlign,
    /// Whether this is a word break opportunity
    pub is_break_opportunity: bool,
}

impl TextFragment {
    pub fn new(text: String, width: f32, height: f32, font_size: f32) -> Self {
        Self {
            text,
            width,
            height,
            x: 0.0,
            y_offset: 0.0,
            font_size,
            has_wrap_opportunity: false,
            vertical_align: VerticalAlign::Baseline,
            is_break_opportunity: false,
        }
    }
}

/// Word breaking state
#[derive(Debug, Clone)]
struct WordBreaker {
    word_break: WordBreak,
    overflow_wrap: OverflowWrap,
}

impl WordBreaker {
    fn new(word_break: WordBreak, overflow_wrap: OverflowWrap) -> Self {
        Self { word_break, overflow_wrap }
    }

    fn can_break_at(&self, ch: char, prev_ch: Option<char>) -> bool {
        match self.word_break {
            WordBreak::BreakAll => {
                // Can break between any two characters
                true
            }
            WordBreak::KeepAll => {
                // Only break at explicit break opportunities
                ch.is_whitespace() || ch == '-'
            }
            WordBreak::Normal => {
                // Standard word breaking
                if ch.is_whitespace() {
                    true
                } else if ch == '-' {
                    true
                } else if let Some(prev) = prev_ch {
                    // Allow break between CJK characters
                    let is_cjk = |c: char| {
                        matches!(c as u32, 
                            0x4E00..=0x9FFF |   // CJK Unified Ideographs
                            0x3040..=0x309F |   // Hiragana
                            0x30A0..=0x30FF |   // Katakana
                            0xAC00..=0xD7AF     // Hangul
                        )
                    };
                    is_cjk(prev) && is_cjk(ch)
                } else {
                    false
                }
            }
        }
    }

    fn should_break_word(&self, _word: &str, width: f32, max_width: f32) -> bool {
        match self.overflow_wrap {
            OverflowWrap::BreakWord | OverflowWrap::Anywhere => {
                width > max_width
            }
            OverflowWrap::Normal => false,
        }
    }
}

/// Line breaking algorithm
pub struct LineBreaker {
    max_width: f32,
    current_line: Line,
    lines: Vec<Line>,
    metrics: TextMetrics,
    word_breaker: WordBreaker,
    text_align: TextAlign,
    hanging_punctuation: bool,
}

impl LineBreaker {
    pub fn new(max_width: f32, font_size: f32) -> Self {
        Self {
            max_width,
            current_line: Line::with_available_width(max_width),
            lines: Vec::new(),
            metrics: TextMetrics::new(font_size),
            word_breaker: WordBreaker::new(WordBreak::Normal, OverflowWrap::Normal),
            text_align: TextAlign::Left,
            hanging_punctuation: false,
        }
    }

    pub fn with_style(mut self, style: &ComputedStyle) -> Self {
        self.word_breaker = WordBreaker::new(
            parse_word_break(style),
            parse_overflow_wrap(style),
        );
        self.text_align = style.text_align;
        self
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
        let words = self.split_text_into_words(text);
        
        for (_i, word) in words.iter().enumerate() {
            let word_width = self.measure_text(&word.text);
            
            // Check if we need to break before this word
            if !self.current_line.is_empty() {
                let space_width = self.metrics.space_width;
                if self.current_line.width + space_width + word_width > self.max_width {
                    // Check if we can break the word
                    if self.word_breaker.should_break_word(&word.text, word_width, self.max_width) {
                        self.break_and_add_word(&word.text, word_width);
                    } else {
                        self.finish_line();
                        self.add_word(word.text.clone(), word_width, word.is_break_opportunity);
                    }
                } else {
                    // Add space before word (except for first word)
                    self.add_fragment(" ", space_width, true);
                    self.add_word(word.text.clone(), word_width, word.is_break_opportunity);
                }
            } else if word_width > self.max_width {
                // Word is too long, need to break it
                if self.word_breaker.overflow_wrap != OverflowWrap::Normal {
                    self.break_and_add_word(&word.text, word_width);
                } else {
                    self.add_word(word.text.clone(), word_width, word.is_break_opportunity);
                }
            } else {
                self.add_word(word.text.clone(), word_width, word.is_break_opportunity);
            }
        }
    }

    /// Split text into words, respecting word breaking rules
    fn split_text_into_words(&self, text: &str) -> Vec<WordInfo> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut prev_ch: Option<char> = None;
        
        for ch in text.chars() {
            if ch.is_whitespace() {
                if !current_word.is_empty() {
                    words.push(WordInfo {
                        text: current_word.clone(),
                        is_break_opportunity: true,
                    });
                    current_word.clear();
                }
            } else if self.word_breaker.can_break_at(ch, prev_ch) && !current_word.is_empty() {
                // Potential break point within word
                words.push(WordInfo {
                    text: current_word.clone(),
                    is_break_opportunity: true,
                });
                current_word.clear();
                current_word.push(ch);
            } else {
                current_word.push(ch);
            }
            prev_ch = Some(ch);
        }
        
        if !current_word.is_empty() {
            words.push(WordInfo {
                text: current_word,
                is_break_opportunity: true,
            });
        }
        
        words
    }

    /// Add preformatted text (preserves all whitespace)
    fn add_preformatted_text(&mut self, text: &str) {
        for line in text.lines() {
            let line_width = self.measure_text(line);
            
            if line_width > self.max_width {
                // Break long lines in preformatted text
                self.break_and_add_word(line, line_width);
            } else {
                self.add_fragment(line, line_width, false);
            }
            
            // Force line break after each line
            self.finish_line();
        }
    }

    /// Add pre-wrap text (preserves whitespace but wraps)
    fn add_pre_wrap_text(&mut self, text: &str) {
        let mut current = String::new();
        let mut current_width = 0.0;
        
        for ch in text.chars() {
            if ch == '\n' {
                if !current.is_empty() {
                    self.add_fragment(&current, current_width, false);
                }
                self.finish_line();
                current.clear();
                current_width = 0.0;
            } else {
                let ch_width = self.measure_char(ch);
                
                if current_width + ch_width > self.max_width && !current.is_empty() {
                    self.add_fragment(&current, current_width, false);
                    self.finish_line();
                    current.clear();
                    current_width = 0.0;
                }
                
                current.push(ch);
                current_width += ch_width;
            }
        }
        
        if !current.is_empty() {
            self.add_fragment(&current, current_width, false);
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
                    self.add_fragment(&current, width, true);
                    current.clear();
                }
                self.finish_line();
                in_whitespace = false;
            } else if ch.is_whitespace() {
                if !in_whitespace && !current.is_empty() {
                    // Collapse multiple spaces
                    let width = self.measure_text(&current);
                    self.add_fragment(&current, width, true);
                    current.clear();
                }
                in_whitespace = true;
            } else {
                in_whitespace = false;
                current.push(ch);
            }
        }
        
        if !current.is_empty() {
            let width = self.measure_text(&current);
            self.add_fragment(&current, width, true);
        }
    }

    /// Add no-wrap text (all on one line)
    fn add_no_wrap_text(&mut self, text: &str) {
        let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let width = self.measure_text(&collapsed);
        self.add_fragment(&collapsed, width, false);
    }

    /// Add a word to the current line
    fn add_word(&mut self, text: String, width: f32, is_break_opportunity: bool) {
        self.add_fragment(&text, width, is_break_opportunity);
    }

    /// Break a long word and add it across multiple lines
    fn break_and_add_word(&mut self, word: &str, _word_width: f32) {
        let mut current = String::new();
        let mut current_width = 0.0;
        
        for ch in word.chars() {
            let ch_width = self.measure_char(ch);
            
            if current_width + ch_width > self.max_width && !current.is_empty() {
                self.add_fragment(&current, current_width, false);
                self.finish_line();
                current.clear();
                current_width = 0.0;
            }
            
            current.push(ch);
            current_width += ch_width;
        }
        
        if !current.is_empty() {
            self.add_fragment(&current, current_width, false);
        }
    }

    /// Break a long word at character boundaries
    fn break_word(&mut self, word: &str) {
        self.break_and_add_word(word, self.measure_text(word));
    }

    /// Add a fragment to the current line
    fn add_fragment(&mut self, text: &str, width: f32, has_wrap_opportunity: bool) {
        let fragment = TextFragment {
            text: text.to_string(),
            width,
            height: self.metrics.line_height,
            x: self.current_line.width,
            y_offset: 0.0,
            font_size: self.metrics.font_size,
            has_wrap_opportunity,
            vertical_align: VerticalAlign::Baseline,
            is_break_opportunity: has_wrap_opportunity,
        };
        
        self.current_line.add_fragment(fragment, &self.metrics);
    }

    /// Finish the current line and start a new one
    fn finish_line(&mut self) {
        if !self.current_line.is_empty() {
            // Apply alignment
            self.current_line.apply_alignment(self.text_align);
            
            // Calculate final positions
            let line_height = self.current_line.height;
            for fragment in &mut self.current_line.fragments {
                fragment.height = line_height;
            }
            
            self.lines.push(self.current_line.clone());
            self.current_line = Line::with_available_width(self.max_width);
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

/// Word information for layout
#[derive(Debug, Clone)]
struct WordInfo {
    text: String,
    is_break_opportunity: bool,
}

/// Parse word-break property
fn parse_word_break(style: &ComputedStyle) -> WordBreak {
    // For now, use word_wrap as a proxy
    match style.word_wrap {
        WordWrap::BreakWord => WordBreak::BreakAll,
        WordWrap::Anywhere => WordBreak::BreakAll,
        WordWrap::Normal => WordBreak::Normal,
    }
}

/// Parse overflow-wrap property
fn parse_overflow_wrap(style: &ComputedStyle) -> OverflowWrap {
    match style.word_wrap {
        WordWrap::BreakWord => OverflowWrap::BreakWord,
        WordWrap::Anywhere => OverflowWrap::Anywhere,
        WordWrap::Normal => OverflowWrap::Normal,
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
        
        let _breaker = LineBreaker::new(max_width, self.metrics.font_size)
            .with_style(style);
        
        // This is a workaround - we'd need to refactor LineBreaker to not consume self
        let mut breaker = LineBreaker::new(max_width, self.metrics.font_size);
        breaker.word_breaker = WordBreaker::new(parse_word_break(style), parse_overflow_wrap(style));
        breaker.text_align = style.text_align;
        
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
pub fn align_line(line: &mut Line, max_width: f32, text_align: TextAlign) {
    line.available_width = max_width;
    line.apply_alignment(text_align);
}

/// Calculate vertical alignment offset
pub fn calculate_vertical_align(
    align: VerticalAlign,
    line_height: f32,
    element_height: f32,
    metrics: &TextMetrics,
) -> f32 {
    match align {
        VerticalAlign::Baseline => 0.0,
        VerticalAlign::Top => -(line_height - element_height),
        VerticalAlign::Bottom => 0.0,
        VerticalAlign::Middle => (line_height - element_height) / 2.0,
        VerticalAlign::Sub => metrics.x_height * 0.5,
        VerticalAlign::Super => -(metrics.x_height * 0.5),
        VerticalAlign::TextTop => -(metrics.ascent - element_height),
        VerticalAlign::TextBottom => -(line_height - metrics.descent - element_height),
        VerticalAlign::Length(l) => -l,
        VerticalAlign::Percent(p) => -(line_height * p / 100.0),
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

/// Calculate optimal line breaking with Knuth-Plass algorithm (simplified)
/// Returns the best break points for justified text
pub fn calculate_optimal_breaks(
    words: &[TextFragment],
    max_width: f32,
    is_last_line: bool,
) -> Vec<usize> {
    if words.is_empty() {
        return Vec::new();
    }

    if is_last_line {
        // Last line doesn't justify, just break when needed
        let mut breaks = Vec::new();
        let mut current_width = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            if current_width + word.width > max_width && i > 0 {
                breaks.push(i - 1);
                current_width = word.width;
            } else {
                current_width += word.width;
            }
        }
        
        breaks
    } else {
        // For justified text, we'd use a more sophisticated algorithm
        // For now, use simple greedy breaking
        let mut breaks = Vec::new();
        let mut current_width = 0.0;
        
        for (i, word) in words.iter().enumerate() {
            // Add space width between words
            let space_width = if i > 0 { 3.0 } else { 0.0 }; // Approximate space
            
            if current_width + word.width + space_width > max_width && i > 0 {
                breaks.push(i - 1);
                current_width = word.width;
            } else {
                current_width += word.width + space_width;
            }
        }
        
        breaks
    }
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

    #[test]
    fn test_text_alignment() {
        let mut line = Line {
            fragments: vec![
                TextFragment::new("Hello".to_string(), 30.0, 12.0, 12.0),
                TextFragment::new("world".to_string(), 35.0, 12.0, 12.0),
            ],
            width: 65.0,
            height: 12.0,
            baseline: 9.6,
            available_width: 100.0,
        };
        
        // Test left alignment (default)
        line.apply_alignment(TextAlign::Left);
        assert_eq!(line.fragments[0].x, 0.0);
        
        // Test center alignment
        line.fragments[0].x = 0.0;
        line.apply_alignment(TextAlign::Center);
        assert!((line.fragments[0].x - 17.5).abs() < 0.1);
        
        // Test right alignment
        line.fragments[0].x = 0.0;
        line.apply_alignment(TextAlign::Right);
        assert!((line.fragments[0].x - 35.0).abs() < 0.1);
    }

    #[test]
    fn test_vertical_align() {
        let metrics = TextMetrics::new(12.0);
        
        // Baseline alignment
        let offset = calculate_vertical_align(VerticalAlign::Baseline, 20.0, 12.0, &metrics);
        assert_eq!(offset, 0.0);
        
        // Middle alignment
        let offset = calculate_vertical_align(VerticalAlign::Middle, 20.0, 12.0, &metrics);
        assert_eq!(offset, 4.0);
        
        // Top alignment
        let offset = calculate_vertical_align(VerticalAlign::Top, 20.0, 12.0, &metrics);
        assert_eq!(offset, -8.0);
    }

    #[test]
    fn test_word_breaking() {
        let mut style = ComputedStyle::default();
        style.word_wrap = WordWrap::BreakWord;
        
        let layout = TextLayout::new(12.0);
        let lines = layout.layout_text("supercalifragilisticexpialidocious", 50.0, &style);
        
        // Should break the long word into multiple lines
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_overflow_wrap() {
        let wrap = OverflowWrap::BreakWord;
        let breaker = WordBreaker::new(WordBreak::Normal, wrap);
        
        assert!(breaker.should_break_word("verylongword", 100.0, 50.0));
        assert!(!breaker.should_break_word("short", 30.0, 50.0));
    }

    #[test]
    fn test_line_justify() {
        let mut line = Line {
            fragments: vec![
                TextFragment::new("Hello".to_string(), 30.0, 12.0, 12.0),
                TextFragment::new("world".to_string(), 35.0, 12.0, 12.0),
            ],
            width: 65.0,
            height: 12.0,
            baseline: 9.6,
            available_width: 100.0,
        };
        
        line.apply_alignment(TextAlign::Justify);
        
        // First fragment should be at 0
        assert_eq!(line.fragments[0].x, 0.0);
        // Second fragment should be offset to distribute space
        assert!(line.fragments[1].x > 35.0);
    }
}

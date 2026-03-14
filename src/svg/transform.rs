//! SVG coordinate transforms
//!
//! Implements SVG transform attribute parsing and matrix operations.
//! Supports: matrix, translate, scale, rotate, skewX, skewY

use crate::types::Point;

/// A 3x3 transformation matrix for 2D graphics
/// 
/// Represented as:
/// | a  c  e |
/// | b  d  f |
/// | 0  0  1 |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub a: f32, // scale x
    pub b: f32, // skew y
    pub c: f32, // skew x
    pub d: f32, // scale y
    pub e: f32, // translate x
    pub f: f32, // translate y
}

impl Transform {
    /// Identity transform (no transformation)
    pub const fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a new transform from matrix values
    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    /// Create a translation transform
    pub const fn translate(x: f32, y: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    /// Create a scaling transform
    pub const fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a uniform scaling transform
    pub const fn scale_uniform(s: f32) -> Self {
        Self::scale(s, s)
    }

    /// Create a rotation transform (angle in degrees)
    pub fn rotate(angle_deg: f32) -> Self {
        Self::rotate_around(angle_deg, 0.0, 0.0)
    }

    /// Create a rotation transform around a point (angle in degrees)
    pub fn rotate_around(angle_deg: f32, cx: f32, cy: f32) -> Self {
        let angle_rad = angle_deg.to_radians();
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        // translate(cx, cy) * rotate(angle) * translate(-cx, -cy)
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: cx - cx * cos + cy * sin,
            f: cy - cx * sin - cy * cos,
        }
    }

    /// Create a skewX transform (angle in degrees)
    pub fn skew_x(angle_deg: f32) -> Self {
        let tan = angle_deg.to_radians().tan();
        Self {
            a: 1.0,
            b: 0.0,
            c: tan,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a skewY transform (angle in degrees)
    pub fn skew_y(angle_deg: f32) -> Self {
        let tan = angle_deg.to_radians().tan();
        Self {
            a: 1.0,
            b: tan,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Multiply two transforms (self * other)
    /// 
    /// The result is the transform that applies other first, then self.
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Apply this transform to a point
    pub fn apply(&self, point: Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y + self.e,
            self.b * point.x + self.d * point.y + self.f,
        )
    }

    /// Apply this transform to a point without translation
    /// (useful for transforming vectors/directions)
    pub fn apply_vector(&self, point: Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y,
            self.b * point.x + self.d * point.y,
        )
    }

    /// Get the inverse of this transform
    /// 
    /// Returns None if the transform is not invertible (determinant is 0)
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        
        if det.abs() < 1e-10 {
            return None;
        }

        let inv_det = 1.0 / det;

        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
    }

    /// Check if this is the identity transform
    pub fn is_identity(&self) -> bool {
        (self.a - 1.0).abs() < 1e-6
            && (self.b).abs() < 1e-6
            && (self.c).abs() < 1e-6
            && (self.d - 1.0).abs() < 1e-6
            && (self.e).abs() < 1e-6
            && (self.f).abs() < 1e-6
    }

    /// Convert to PDF format: [a b c d e f]
    /// 
    /// PDF uses a different matrix format, this returns the values
    /// in the order expected by PDF's cm operator
    pub fn to_pdf_array(&self) -> [f32; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    /// Format as PDF cm operator argument string
    pub fn to_pdf_string(&self) -> String {
        format!(
            "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6}",
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Parse an SVG transform attribute string
/// 
/// Supports the following functions:
/// - matrix(a b c d e f)
/// - translate(x [y])
/// - scale(x [y])
/// - rotate(angle [cx cy])
/// - skewX(angle)
/// - skewY(angle)
pub fn parse_transform(transform_str: &str) -> Result<Transform, String> {
    let mut result = Transform::identity();
    let parser = TransformParser::new(transform_str);
    
    for parsed in parser {
        let transform = parsed?;
        result = result.multiply(&transform);
    }
    
    Ok(result)
}

/// Parser for transform attribute
struct TransformParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> TransformParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let c = self.peek();
            if c.is_ascii_whitespace() || c == ',' {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn parse_function(&mut self) -> Option<Result<Transform, String>> {
        self.skip_whitespace();

        if self.is_eof() {
            return None;
        }

        // Parse function name
        let name_start = self.pos;
        while !self.is_eof() && self.peek().is_ascii_alphabetic() {
            self.advance(1);
        }

        if self.pos == name_start {
            return Some(Err(format!("Expected function name at position {}", self.pos)));
        }

        let name = &self.input[name_start..self.pos];
        
        self.skip_whitespace();
        
        if !self.peek_str("(") {
            return Some(Err(format!("Expected '(' after {}", name)));
        }
        self.advance(1);
        self.skip_whitespace();

        // Parse arguments based on function type
        let result = match name {
            "matrix" => self.parse_matrix_args(),
            "translate" => self.parse_translate_args(),
            "scale" => self.parse_scale_args(),
            "rotate" => self.parse_rotate_args(),
            "skewX" => self.parse_skew_x_args(),
            "skewY" => self.parse_skew_y_args(),
            _ => Err(format!("Unknown transform function: {}", name)),
        };

        self.skip_whitespace();
        
        if !self.peek_str(")") {
            return Some(Err("Expected ')'".to_string()));
        }
        self.advance(1);

        Some(result)
    }

    fn parse_matrix_args(&mut self) -> Result<Transform, String> {
        let a = self.parse_number()?;
        let b = self.parse_number()?;
        let c = self.parse_number()?;
        let d = self.parse_number()?;
        let e = self.parse_number()?;
        let f = self.parse_number()?;
        
        Ok(Transform::new(a, b, c, d, e, f))
    }

    fn parse_translate_args(&mut self) -> Result<Transform, String> {
        let x = self.parse_number()?;
        let y = self.parse_number().unwrap_or(0.0);
        
        Ok(Transform::translate(x, y))
    }

    fn parse_scale_args(&mut self) -> Result<Transform, String> {
        let x = self.parse_number()?;
        let y = self.parse_number().unwrap_or(x);
        
        Ok(Transform::scale(x, y))
    }

    fn parse_rotate_args(&mut self) -> Result<Transform, String> {
        let angle = self.parse_number()?;
        
        // Check if rotation center is specified
        if let Some(cx) = self.parse_number() {
            let cy = self.parse_number().unwrap_or(0.0);
            Ok(Transform::rotate_around(angle, cx, cy))
        } else {
            Ok(Transform::rotate(angle))
        }
    }

    fn parse_skew_x_args(&mut self) -> Result<Transform, String> {
        let angle = self.parse_number()?;
        Ok(Transform::skew_x(angle))
    }

    fn parse_skew_y_args(&mut self) -> Result<Transform, String> {
        let angle = self.parse_number()?;
        Ok(Transform::skew_y(angle))
    }

    fn parse_number(&mut self) -> Result<f32, String> {
        self.skip_whitespace();

        let start = self.pos;
        let mut has_digits = false;

        // Optional sign
        if self.peek() == '-' || self.peek() == '+' {
            self.advance(1);
        }

        // Integer part
        while self.peek().is_ascii_digit() {
            self.advance(1);
            has_digits = true;
        }

        // Fractional part
        if self.peek() == '.' {
            self.advance(1);
            while self.peek().is_ascii_digit() {
                self.advance(1);
                has_digits = true;
            }
        }

        // Exponent
        if self.peek() == 'e' || self.peek() == 'E' {
            self.advance(1);
            if self.peek() == '-' || self.peek() == '+' {
                self.advance(1);
            }
            while self.peek().is_ascii_digit() {
                self.advance(1);
            }
        }

        if !has_digits {
            return Err(format!("Expected number at position {}", start));
        }

        self.input[start..self.pos]
            .parse()
            .map_err(|e| format!("Invalid number: {}", e))
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn peek_str(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

impl<'a> Iterator for TransformParser<'a> {
    type Item = Result<Transform, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_function()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        let p = Point::new(10.0, 20.0);
        assert_eq!(t.apply(p), p);
    }

    #[test]
    fn test_translate() {
        let t = Transform::translate(10.0, 20.0);
        let p = Point::new(5.0, 5.0);
        assert_eq!(t.apply(p), Point::new(15.0, 25.0));
    }

    #[test]
    fn test_scale() {
        let t = Transform::scale(2.0, 3.0);
        let p = Point::new(10.0, 10.0);
        assert_eq!(t.apply(p), Point::new(20.0, 30.0));
    }

    #[test]
    fn test_rotate() {
        let t = Transform::rotate(90.0);
        let p = Point::new(1.0, 0.0);
        let result = t.apply(p);
        assert!((result.x - 0.0).abs() < 1e-6);
        assert!((result.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_rotate_around() {
        let t = Transform::rotate_around(90.0, 10.0, 10.0);
        // Point at center of rotation should stay fixed
        let center = Point::new(10.0, 10.0);
        let result = t.apply(center);
        assert!((result.x - 10.0).abs() < 1e-6);
        assert!((result.y - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiply() {
        let t1 = Transform::translate(10.0, 0.0);
        let t2 = Transform::scale(2.0, 2.0);
        let combined = t1.multiply(&t2);
        
        let p = Point::new(5.0, 5.0);
        let result = combined.apply(p);
        // scale then translate: (5*2 + 10, 5*2 + 0) = (20, 10)
        assert_eq!(result, Point::new(20.0, 10.0));
    }

    #[test]
    fn test_parse_translate() {
        let t = parse_transform("translate(10, 20)").unwrap();
        let p = Point::new(5.0, 5.0);
        assert_eq!(t.apply(p), Point::new(15.0, 25.0));
    }

    #[test]
    fn test_parse_translate_single_arg() {
        let t = parse_transform("translate(10)").unwrap();
        let p = Point::new(5.0, 5.0);
        assert_eq!(t.apply(p), Point::new(15.0, 5.0));
    }

    #[test]
    fn test_parse_scale() {
        let t = parse_transform("scale(2, 3)").unwrap();
        let p = Point::new(10.0, 10.0);
        assert_eq!(t.apply(p), Point::new(20.0, 30.0));
    }

    #[test]
    fn test_parse_rotate() {
        let t = parse_transform("rotate(90)").unwrap();
        let p = Point::new(1.0, 0.0);
        let result = t.apply(p);
        assert!((result.x - 0.0).abs() < 1e-5);
        assert!((result.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_parse_matrix() {
        let t = parse_transform("matrix(1 0 0 1 10 20)").unwrap();
        let p = Point::new(5.0, 5.0);
        assert_eq!(t.apply(p), Point::new(15.0, 25.0));
    }

    #[test]
    fn test_parse_multiple() {
        let t = parse_transform("translate(10, 20) scale(2)").unwrap();
        let p = Point::new(5.0, 5.0);
        let result = t.apply(p);
        // scale then translate
        assert_eq!(result, Point::new(20.0, 30.0));
    }

    #[test]
    fn test_parse_skew() {
        let t = parse_transform("skewX(45)").unwrap();
        let p = Point::new(1.0, 1.0);
        let result = t.apply(p);
        // After 45 degree skewX: x' = x + tan(45) * y = 1 + 1 = 2
        assert!((result.x - 2.0).abs() < 1e-5);
        assert!((result.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_pdf_string() {
        let t = Transform::translate(10.0, 20.0);
        let s = t.to_pdf_string();
        assert!(s.contains("1.000000")); // a
        assert!(s.contains("10.000000") || s.contains("20.000000")); // e or f
    }

    #[test]
    fn test_inverse() {
        let t = Transform::translate(10.0, 20.0);
        let inv = t.inverse().unwrap();
        let p = Point::new(15.0, 25.0);
        let back = inv.apply(p);
        assert!((back.x - 5.0).abs() < 1e-6);
        assert!((back.y - 5.0).abs() < 1e-6);
    }
}

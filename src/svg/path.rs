//! SVG path parsing and commands
//!
//! Implements SVG path data parsing according to the SVG specification.
//! Supports all path commands: M, L, H, V, C, S, Q, T, A, Z

use crate::types::Point;

/// SVG path command
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    /// Move to (absolute)
    MoveTo(Point),
    /// Move to (relative)
    MoveToRel(Point),
    /// Line to (absolute)
    LineTo(Point),
    /// Line to (relative)
    LineToRel(Point),
    /// Horizontal line (absolute)
    HorizontalTo(f32),
    /// Horizontal line (relative)
    HorizontalToRel(f32),
    /// Vertical line (absolute)
    VerticalTo(f32),
    /// Vertical line (relative)
    VerticalToRel(f32),
    /// Cubic Bezier curve (absolute): C x1 y1, x2 y2, x y
    CubicTo(Point, Point, Point),
    /// Cubic Bezier curve (relative)
    CubicToRel(Point, Point, Point),
    /// Smooth cubic Bezier (absolute): S x2 y2, x y
    SmoothCubicTo(Point, Point),
    /// Smooth cubic Bezier (relative)
    SmoothCubicToRel(Point, Point),
    /// Quadratic Bezier curve (absolute): Q x1 y1, x y
    QuadraticTo(Point, Point),
    /// Quadratic Bezier curve (relative)
    QuadraticToRel(Point, Point),
    /// Smooth quadratic Bezier (absolute): T x y
    SmoothQuadraticTo(Point),
    /// Smooth quadratic Bezier (relative)
    SmoothQuadraticToRel(Point),
    /// Arc (absolute): A rx ry x-axis-rotation large-arc-flag sweep-flag x y
    ArcTo(f32, f32, f32, bool, bool, Point),
    /// Arc (relative)
    ArcToRel(f32, f32, f32, bool, bool, Point),
    /// Close path
    ClosePath,
}

/// Parser for SVG path data
pub struct PathParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> PathParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<PathCommand>, String> {
        let mut commands = Vec::new();
        let mut current_pos = Point::new(0.0, 0.0);
        let mut subpath_start = Point::new(0.0, 0.0);
        let mut last_cubic_control: Option<Point> = None;
        let mut last_quad_control: Option<Point> = None;

        while !self.is_eof() {
            self.skip_whitespace_and_separators();
            
            if self.is_eof() {
                break;
            }

            let cmd_char = self.peek();
            if !cmd_char.is_ascii_alphabetic() {
                return Err(format!("Expected command, found '{}' at position {}", cmd_char, self.pos));
            }

            self.advance(1);
            let is_relative = cmd_char.is_ascii_lowercase();
            let cmd_upper = cmd_char.to_ascii_uppercase();

            match cmd_upper {
                'M' => {
                    // Move to - can be followed by multiple coordinate pairs (treated as line-to)
                    let mut first = true;
                    while let Some((x, y)) = self.parse_coordinate_pair() {
                        let point = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        if first {
                            commands.push(PathCommand::MoveTo(point));
                            subpath_start = point;
                            first = false;
                        } else {
                            commands.push(PathCommand::LineTo(point));
                        }
                        current_pos = point;
                    }
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                'L' => {
                    while let Some((x, y)) = self.parse_coordinate_pair() {
                        let point = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };
                        commands.push(PathCommand::LineTo(point));
                        current_pos = point;
                    }
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                'H' => {
                    while let Some(x) = self.parse_number() {
                        let x_abs = if is_relative { current_pos.x + x } else { x };
                        commands.push(PathCommand::HorizontalTo(x_abs));
                        current_pos.x = x_abs;
                    }
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                'V' => {
                    while let Some(y) = self.parse_number() {
                        let y_abs = if is_relative { current_pos.y + y } else { y };
                        commands.push(PathCommand::VerticalTo(y_abs));
                        current_pos.y = y_abs;
                    }
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                'C' => {
                    while let Some((x1, y1)) = self.parse_coordinate_pair() {
                        let (x2, y2) = self.parse_coordinate_pair()
                            .ok_or_else(|| "Expected second control point for cubic".to_string())?;
                        let (x, y) = self.parse_coordinate_pair()
                            .ok_or_else(|| "Expected end point for cubic".to_string())?;

                        let p1 = if is_relative {
                            Point::new(current_pos.x + x1, current_pos.y + y1)
                        } else {
                            Point::new(x1, y1)
                        };
                        let p2 = if is_relative {
                            Point::new(current_pos.x + x2, current_pos.y + y2)
                        } else {
                            Point::new(x2, y2)
                        };
                        let end = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        commands.push(PathCommand::CubicTo(p1, p2, end));
                        last_cubic_control = Some(p2);
                        current_pos = end;
                    }
                    last_quad_control = None;
                }
                'S' => {
                    while let Some((x2, y2)) = self.parse_coordinate_pair() {
                        let (x, y) = self.parse_coordinate_pair()
                            .ok_or_else(|| "Expected end point for smooth cubic".to_string())?;

                        // Calculate first control point as reflection of last control point
                        let p1 = last_cubic_control.map(|cp| {
                            Point::new(2.0 * current_pos.x - cp.x, 2.0 * current_pos.y - cp.y)
                        }).unwrap_or(current_pos);

                        let p2 = if is_relative {
                            Point::new(current_pos.x + x2, current_pos.y + y2)
                        } else {
                            Point::new(x2, y2)
                        };
                        let end = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        commands.push(PathCommand::SmoothCubicTo(p2, end));
                        last_cubic_control = Some(p2);
                        current_pos = end;
                    }
                    last_quad_control = None;
                }
                'Q' => {
                    while let Some((x1, y1)) = self.parse_coordinate_pair() {
                        let (x, y) = self.parse_coordinate_pair()
                            .ok_or_else(|| "Expected end point for quadratic".to_string())?;

                        let control = if is_relative {
                            Point::new(current_pos.x + x1, current_pos.y + y1)
                        } else {
                            Point::new(x1, y1)
                        };
                        let end = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        commands.push(PathCommand::QuadraticTo(control, end));
                        last_quad_control = Some(control);
                        current_pos = end;
                    }
                    last_cubic_control = None;
                }
                'T' => {
                    while let Some((x, y)) = self.parse_coordinate_pair() {
                        // Calculate control point as reflection of last control point
                        let control = last_quad_control.map(|cp| {
                            Point::new(2.0 * current_pos.x - cp.x, 2.0 * current_pos.y - cp.y)
                        }).unwrap_or(current_pos);

                        let end = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        commands.push(PathCommand::SmoothQuadraticTo(end));
                        last_quad_control = Some(control);
                        current_pos = end;
                    }
                    last_cubic_control = None;
                }
                'A' => {
                    while let Some(rx) = self.parse_number() {
                        let ry = self.parse_number()
                            .ok_or_else(|| "Expected ry for arc".to_string())?;
                        let x_axis_rotation = self.parse_number()
                            .ok_or_else(|| "Expected x-axis-rotation for arc".to_string())?;
                        let large_arc_flag = self.parse_flag()
                            .ok_or_else(|| "Expected large-arc-flag for arc".to_string())?;
                        let sweep_flag = self.parse_flag()
                            .ok_or_else(|| "Expected sweep-flag for arc".to_string())?;
                        let (x, y) = self.parse_coordinate_pair()
                            .ok_or_else(|| "Expected end point for arc".to_string())?;

                        let end = if is_relative {
                            Point::new(current_pos.x + x, current_pos.y + y)
                        } else {
                            Point::new(x, y)
                        };

                        commands.push(PathCommand::ArcTo(rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end));
                        current_pos = end;
                    }
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                'Z' => {
                    commands.push(PathCommand::ClosePath);
                    current_pos = subpath_start;
                    last_cubic_control = None;
                    last_quad_control = None;
                }
                _ => {
                    return Err(format!("Unknown path command: {}", cmd_char));
                }
            }
        }

        Ok(commands)
    }

    fn parse_coordinate_pair(&mut self) -> Option<(f32, f32)> {
        self.skip_whitespace_and_separators();
        let x = self.parse_number()?;
        self.skip_whitespace_and_separators();
        let y = self.parse_number()?;
        Some((x, y))
    }

    fn parse_number(&mut self) -> Option<f32> {
        self.skip_whitespace_and_separators();

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

        if !has_digits || self.pos == start {
            self.pos = start;
            return None;
        }

        self.input[start..self.pos].parse().ok()
    }

    fn parse_flag(&mut self) -> Option<bool> {
        self.skip_whitespace_and_separators();
        let c = self.peek();
        if c == '0' || c == '1' {
            self.advance(1);
            Some(c == '1')
        } else {
            None
        }
    }

    fn skip_whitespace_and_separators(&mut self) {
        while !self.is_eof() {
            let c = self.peek();
            if c.is_ascii_whitespace() || c == ',' {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

/// Parse SVG path data string into commands
pub fn parse_path_data(data: &str) -> Result<Vec<PathCommand>, String> {
    let mut parser = PathParser::new(data);
    parser.parse()
}

/// Convert path commands to PDF path operations
/// 
/// Returns a string containing PDF path operators
pub fn path_commands_to_pdf(commands: &[PathCommand]) -> String {
    let mut result = String::new();
    let mut current_pos = Point::new(0.0, 0.0);

    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(p) | PathCommand::MoveToRel(p) => {
                result.push_str(&format!("{:.3} {:.3} m\n", p.x, p.y));
                current_pos = *p;
            }
            PathCommand::LineTo(p) | PathCommand::LineToRel(p) => {
                result.push_str(&format!("{:.3} {:.3} l\n", p.x, p.y));
                current_pos = *p;
            }
            PathCommand::HorizontalTo(x) => {
                result.push_str(&format!("{:.3} {:.3} l\n", x, current_pos.y));
                current_pos.x = *x;
            }
            PathCommand::HorizontalToRel(x) => {
                let new_x = current_pos.x + x;
                result.push_str(&format!("{:.3} {:.3} l\n", new_x, current_pos.y));
                current_pos.x = new_x;
            }
            PathCommand::VerticalTo(y) => {
                result.push_str(&format!("{:.3} {:.3} l\n", current_pos.x, y));
                current_pos.y = *y;
            }
            PathCommand::VerticalToRel(y) => {
                let new_y = current_pos.y + y;
                result.push_str(&format!("{:.3} {:.3} l\n", current_pos.x, new_y));
                current_pos.y = new_y;
            }
            PathCommand::CubicTo(cp1, cp2, end) |
            PathCommand::CubicToRel(cp1, cp2, end) => {
                result.push_str(&format!(
                    "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n",
                    cp1.x, cp1.y, cp2.x, cp2.y, end.x, end.y
                ));
                current_pos = *end;
            }
            PathCommand::SmoothCubicTo(cp2, end) |
            PathCommand::SmoothCubicToRel(cp2, end) => {
                result.push_str(&format!(
                    "{:.3} {:.3} {:.3} {:.3} c\n",
                    cp2.x, cp2.y, end.x, end.y
                ));
                current_pos = *end;
            }
            PathCommand::QuadraticTo(cp, end) |
            PathCommand::QuadraticToRel(cp, end) => {
                // Convert quadratic to cubic for PDF
                let cp1 = Point::new(
                    current_pos.x + 2.0/3.0 * (cp.x - current_pos.x),
                    current_pos.y + 2.0/3.0 * (cp.y - current_pos.y)
                );
                let cp2 = Point::new(
                    end.x + 2.0/3.0 * (cp.x - end.x),
                    end.y + 2.0/3.0 * (cp.y - end.y)
                );
                result.push_str(&format!(
                    "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n",
                    cp1.x, cp1.y, cp2.x, cp2.y, end.x, end.y
                ));
                current_pos = *end;
            }
            PathCommand::SmoothQuadraticTo(end) |
            PathCommand::SmoothQuadraticToRel(end) => {
                // This needs the previous control point to calculate
                // For now, treat as line
                result.push_str(&format!("{:.3} {:.3} l\n", end.x, end.y));
                current_pos = *end;
            }
            PathCommand::ArcTo(rx, ry, rotation, large_arc, sweep, end) |
            PathCommand::ArcToRel(rx, ry, rotation, large_arc, sweep, end) => {
                // Convert arc to cubic bezier curves
                // This is a simplified implementation
                // A full implementation would use endpoint parameterization
                let arc_commands = arc_to_beziers(
                    current_pos, *rx, *ry, *rotation, *large_arc, *sweep, *end
                );
                for cmd in arc_commands {
                    result.push_str(&cmd);
                }
                current_pos = *end;
            }
            PathCommand::ClosePath => {
                result.push_str("h\n");
            }
        }
    }

    result
}

/// Convert an SVG arc to cubic bezier curves
/// Based on the SVG specification algorithm
fn arc_to_beziers(
    start: Point,
    rx: f32,
    ry: f32,
    phi: f32,
    large_arc: bool,
    sweep: bool,
    end: Point,
) -> Vec<String> {
    let mut result = Vec::new();

    // If the start and end points are the same, return empty
    if (start.x - end.x).abs() < 0.0001 && (start.y - end.y).abs() < 0.0001 {
        return result;
    }

    // Ensure radii are positive
    let rx = rx.abs();
    let ry = ry.abs();

    // If either radius is zero, treat as line
    if rx < 0.0001 || ry < 0.0001 {
        result.push(format!("{:.3} {:.3} l\n", end.x, end.y));
        return result;
    }

    // Convert angle to radians
    let phi = phi.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    // Step 1: Compute (x1', y1')
    let dx2 = (start.x - end.x) / 2.0;
    let dy2 = (start.y - end.y) / 2.0;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    // Step 2: Compute (cx', cy')
    let mut lambda = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    let (rx, ry) = if lambda > 1.0 {
        let scale = lambda.sqrt();
        (rx * scale, ry * scale)
    } else {
        (rx, ry)
    };

    let num = (rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p)
        .max(0.0);
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    
    let factor = if den == 0.0 { 0.0 } else { (num / den).sqrt() };
    let factor = if large_arc == sweep { -factor } else { factor };

    let cxp = factor * rx * y1p / ry;
    let cyp = -factor * ry * x1p / rx;

    // Step 3: Compute (cx, cy)
    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    // Step 4: Compute start and sweep angles
    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let mut delta_theta = (((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx)) - theta1;

    if sweep && delta_theta < 0.0 {
        delta_theta += 2.0 * std::f32::consts::PI;
    } else if !sweep && delta_theta > 0.0 {
        delta_theta -= 2.0 * std::f32::consts::PI;
    }

    // Approximate arc with cubic beziers
    // Use 4 segments for full circle, fewer for smaller arcs
    let num_segments = ((delta_theta.abs() / (std::f32::consts::PI / 2.0)).ceil() as usize).max(1).min(4);
    let eta1 = theta1;
    let eta_delta = delta_theta / num_segments as f32;

    for i in 0..num_segments {
        let eta2 = eta1 + eta_delta * (i as f32 + 1.0);
        
        // Calculate bezier control points for this segment
        let ep = eta1 + eta_delta * i as f32;
        let e2 = eta2;
        
        let cos_ep = ep.cos();
        let sin_ep = ep.sin();
        let cos_e2 = e2.cos();
        let sin_e2 = e2.sin();

        // Start point of this segment
        let p1x = cx + rx * cos_phi * cos_ep - ry * sin_phi * sin_ep;
        let p1y = cy + rx * sin_phi * cos_ep + ry * cos_phi * sin_ep;

        // End point of this segment
        let p2x = cx + rx * cos_phi * cos_e2 - ry * sin_phi * sin_e2;
        let p2y = cy + rx * sin_phi * cos_e2 + ry * cos_phi * sin_e2;

        // Control points (approximation)
        let alpha = (eta_delta.tan() * 4.0 / 3.0).sin();
        
        let cp1x = p1x + alpha * (-rx * cos_phi * sin_ep - ry * sin_phi * cos_ep);
        let cp1y = p1y + alpha * (-rx * sin_phi * sin_ep + ry * cos_phi * cos_ep);
        
        let cp2x = p2x - alpha * (-rx * cos_phi * sin_e2 - ry * sin_phi * cos_e2);
        let cp2y = p2y - alpha * (-rx * sin_phi * sin_e2 + ry * cos_phi * cos_e2);

        result.push(format!(
            "{:.3} {:.3} {:.3} {:.3} {:.3} {:.3} c\n",
            cp1x, cp1y, cp2x, cp2y, p2x, p2y
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_move_to() {
        let path = "M 100 200";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], PathCommand::MoveTo(Point::new(100.0, 200.0)));
    }

    #[test]
    fn test_parse_line_to() {
        let path = "M 0 0 L 100 100";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[1], PathCommand::LineTo(Point::new(100.0, 100.0)));
    }

    #[test]
    fn test_parse_relative() {
        let path = "M 10 10 l 20 30";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands[1], PathCommand::LineToRel(Point::new(20.0, 30.0)));
    }

    #[test]
    fn test_parse_cubic_bezier() {
        let path = "M 0 0 C 10 10 20 10 30 0";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 2);
        match &commands[1] {
            PathCommand::CubicTo(cp1, cp2, end) => {
                assert_eq!(*cp1, Point::new(10.0, 10.0));
                assert_eq!(*cp2, Point::new(20.0, 10.0));
                assert_eq!(*end, Point::new(30.0, 0.0));
            }
            _ => panic!("Expected CubicTo"),
        }
    }

    #[test]
    fn test_parse_close_path() {
        let path = "M 0 0 L 100 0 L 100 100 Z";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 4);
        assert_eq!(commands[3], PathCommand::ClosePath);
    }

    #[test]
    fn test_parse_complex_path() {
        let path = "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z";
        let commands = parse_path_data(path).unwrap();
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_parse_arc() {
        let path = "M 100 100 A 50 50 0 0 1 150 150";
        let commands = parse_path_data(path).unwrap();
        assert_eq!(commands.len(), 2);
        match &commands[1] {
            PathCommand::ArcTo(rx, ry, rot, large, sweep, end) => {
                assert_eq!(*rx, 50.0);
                assert_eq!(*ry, 50.0);
                assert_eq!(*rot, 0.0);
                assert_eq!(*large, false);
                assert_eq!(*sweep, true);
                assert_eq!(*end, Point::new(150.0, 150.0));
            }
            _ => panic!("Expected ArcTo"),
        }
    }
}

//! CSS Tokenizer (CSS Syntax Module Level 3)

use std::iter::Peekable;
use std::str::Chars;

/// CSS Token types
#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    /// Identifier token
    Ident(String),
    /// Function token (identifier followed by ()
    Function(String),
    /// At-keyword token (@identifier)
    AtKeyword(String),
    /// Hash token (used for IDs and colors)
    Hash(String, HashType),
    /// String token
    String(String),
    /// Bad string (unclosed)
    BadString(String),
    /// URL token
    Url(String),
    /// Bad URL (malformed)
    BadUrl(String),
    /// Delimiter character
    Delim(char),
    /// Number token
    Number(f64, NumberType),
    /// Percentage token
    Percentage(f64),
    /// Dimension token (number with unit)
    Dimension(f64, String, NumberType),
    /// Whitespace
    Whitespace,
    /// CDO comment start (<!--)
    CDO,
    /// CDC comment end (--!)
    CDC,
    /// Colon :
    Colon,
    /// Semicolon ;
    Semicolon,
    /// Comma ,
    Comma,
    /// Opening square bracket [
    OpenBracket,
    /// Closing square bracket ]
    CloseBracket,
    /// Opening parenthesis (
    OpenParen,
    /// Closing parenthesis )
    CloseParen,
    /// Opening curly brace {
    OpenBrace,
    /// Closing curly brace }
    CloseBrace,
    /// Comment
    Comment(String),
    /// End of file
    EOF,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HashType {
    Id,
    Unrestricted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberType {
    Integer,
    Number,
}

/// CSS Tokenizer
pub struct CssTokenizer<'a> {
    input: &'a str,
    chars: Peekable<Chars<'a>>,
    position: usize,
}

impl<'a> CssTokenizer<'a> {
    /// Create a new CSS tokenizer
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            position: 0,
        }
    }

    /// Consume and return next character
    fn consume(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Peek at next character without consuming
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    /// Consume while predicate is true
    fn consume_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if pred(ch) {
                result.push(self.consume().unwrap());
            } else {
                break;
            }
        }
        result
    }

    /// Consume next token
    pub fn next_token(&mut self) -> CssToken {
        self.consume_token()
    }

    fn consume_token(&mut self) -> CssToken {
        match self.peek() {
            None => CssToken::EOF,
            Some(ch) => match ch {
                ' ' | '\t' | '\n' | '\r' | '\x0C' => {
                    self.consume_whitespace()
                }
                '"' | '\'' => {
                    self.consume_string()
                }
                '#' => {
                    self.consume_hash()
                }
                '$' | '+' | ',' | '.' | '/' | ':' | ';' | '<' | '>' | '?' | '@' => {
                    self.consume_at_delim()
                }
                '(' => {
                    self.consume();
                    CssToken::OpenParen
                }
                ')' => {
                    self.consume();
                    CssToken::CloseParen
                }
                '[' => {
                    self.consume();
                    CssToken::OpenBracket
                }
                ']' => {
                    self.consume();
                    CssToken::CloseBracket
                }
                '{' => {
                    self.consume();
                    CssToken::OpenBrace
                }
                '}' => {
                    self.consume();
                    CssToken::CloseBrace
                }
                '\\' => {
                    // Escape
                    self.consume_ident_like()
                }
                '0'..='9' => {
                    self.consume_numeric()
                }
                '-' => {
                    self.consume_minus()
                }
                'a'..='z' | 'A'..='Z' | '_' | '\u{80}'..=char::MAX => {
                    self.consume_ident_like()
                }
        
                '*' => {
                    self.consume();
                    if self.peek() == Some('=') {
                        self.consume();
                        CssToken::Delim('*') // Should be *=
                    } else {
                        CssToken::Delim('*')
                    }
                }
                '^' | '~' | '|' => {
                    self.consume();
                    if self.peek() == Some('=') {
                        self.consume();
                    }
                    CssToken::Delim(ch)
                }
                _ => {
                    self.consume();
                    CssToken::Delim(ch)
                }
            }
        }
    }

    fn consume_whitespace(&mut self) -> CssToken {
        self.consume_while(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\x0C');
        CssToken::Whitespace
    }

    fn consume_string(&mut self) -> CssToken {
        let quote = self.consume().unwrap();
        let mut result = String::new();

        loop {
            match self.peek() {
                None => return CssToken::BadString(result),
                Some(c) if c == quote => {
                    self.consume();
                    return CssToken::String(result);
                }
                Some('\n') | Some('\r') | Some('\x0C') => {
                    return CssToken::BadString(result);
                }
                Some('\\') => {
                    self.consume();
                    if let Some(escaped) = self.consume_escape() {
                        result.push(escaped);
                    }
                }
                Some(c) => {
                    self.consume();
                    result.push(c);
                }
            }
        }
    }

    fn consume_hash(&mut self) -> CssToken {
        self.consume(); // #
        
        if self.would_start_ident() {
            let name = self.consume_ident_sequence();
            let hash_type = if self.is_ident_start(self.input.as_bytes().get(self.position.saturating_sub(name.len() + 1)).copied().map(|b| b as char).unwrap_or('\0')) {
                HashType::Id
            } else {
                HashType::Unrestricted
            };
            CssToken::Hash(name, hash_type)
        } else {
            CssToken::Delim('#')
        }
    }

    fn consume_at_delim(&mut self) -> CssToken {
        let ch = self.consume().unwrap();
        
        if ch == '@' && self.would_start_ident() {
            let name = self.consume_ident_sequence();
            CssToken::AtKeyword(name)
        } else if ch == '<' && self.peek() == Some('!') {
            self.consume();
            if self.peek() == Some('-') {
                self.consume();
                if self.peek() == Some('-') {
                    self.consume();
                    return CssToken::CDO;
                }
            }
            // Revert
            CssToken::Delim('<')
        } else if ch == '-' && self.peek() == Some('-') {
            self.consume();
            if self.peek() == Some('>') {
                self.consume();
                return CssToken::CDC;
            }
            CssToken::Delim('-')
        } else {
            CssToken::Delim(ch)
        }
    }

    fn consume_numeric(&mut self) -> CssToken {
        let number = self.consume_number();
        
        if self.would_start_ident() {
            let unit = self.consume_ident_sequence();
            CssToken::Dimension(number.value, unit, number.number_type)
        } else if self.peek() == Some('%') {
            self.consume();
            CssToken::Percentage(number.value)
        } else {
            CssToken::Number(number.value, number.number_type)
        }
    }

    fn consume_minus(&mut self) -> CssToken {
        self.consume(); // -
        
        if self.would_start_ident() {
            // Negative number with ident, or ident starting with -
            self.consume_ident_like_prefixed("-")
        } else if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            // Negative number
            let number = self.consume_number();
            let sign = -1.0;
            
            if self.would_start_ident() {
                let unit = self.consume_ident_sequence();
                CssToken::Dimension(sign * number.value, unit, number.number_type)
            } else if self.peek() == Some('%') {
                self.consume();
                CssToken::Percentage(sign * number.value)
            } else {
                CssToken::Number(sign * number.value, number.number_type)
            }
        } else if self.peek() == Some('-') && self.input[self.position..].starts_with("-->") {
            // CDC
            self.consume();
            self.consume();
            CssToken::CDC
        } else if self.peek() == Some('.') && self.input[self.position..].chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
            // Negative decimal
            self.consume(); // .
            let number = self.consume_number();
            CssToken::Number(-number.value, number.number_type)
        } else {
            CssToken::Delim('-')
        }
    }

    fn consume_ident_like(&mut self) -> CssToken {
        let name = self.consume_ident_sequence();
        
        if self.peek() == Some('(') {
            self.consume();
            if name.eq_ignore_ascii_case("url") {
                self.consume_url()
            } else {
                CssToken::Function(name)
            }
        } else {
            CssToken::Ident(name)
        }
    }

    fn consume_ident_like_prefixed(&mut self, prefix: &str) -> CssToken {
        let name = prefix.to_string() + &self.consume_ident_sequence();
        
        if self.peek() == Some('(') {
            self.consume();
            CssToken::Function(name)
        } else {
            CssToken::Ident(name)
        }
    }

    fn consume_url(&mut self) -> CssToken {
        // Consume URL token
        self.consume_whitespace();
        
        match self.peek() {
            Some('"') | Some('\'') => {
                // URL as string
                let str_token = self.consume_string();
                match str_token {
                    CssToken::String(s) => {
                        self.consume_whitespace();
                        if self.peek() == Some(')') {
                            self.consume();
                            CssToken::Url(s)
                        } else {
                            CssToken::BadUrl(s)
                        }
                    }
                    CssToken::BadString(s) => {
                        self.consume_bad_url_remnants();
                        CssToken::BadUrl(s)
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let mut url = String::new();
                loop {
                    match self.peek() {
                        None | Some(')') => break,
                        Some(' ') | Some('\t') | Some('\n') | Some('\r') | Some('\x0C') => break,
                        Some('"') | Some('\'') | Some('(') => {
                            self.consume_bad_url_remnants();
                            return CssToken::BadUrl(url);
                        }
                        Some('\\') => {
                            self.consume();
                            if let Some(escaped) = self.consume_escape() {
                                url.push(escaped);
                            }
                        }
                        Some(c) => {
                            self.consume();
                            url.push(c);
                        }
                    }
                }
                
                self.consume_whitespace();
                if self.peek() == Some(')') || self.peek().is_none() {
                    self.consume();
                    CssToken::Url(url)
                } else {
                    self.consume_bad_url_remnants();
                    CssToken::BadUrl(url)
                }
            }
        }
    }

    fn consume_bad_url_remnants(&mut self) {
        loop {
            match self.peek() {
                None | Some(')') => {
                    self.consume();
                    break;
                }
                Some('\\') => {
                    self.consume();
                    if self.peek() != Some('\n') {
                        // Skip escaped char
                        self.consume();
                    }
                }
                _ => {
                    self.consume();
                }
            }
        }
    }

    #[allow(dead_code)]
    fn consume_comment(&mut self) -> CssToken {
        // Already consumed /*
        self.consume(); // *
        
        let mut content = String::new();
        loop {
            match self.peek() {
                None => break,
                Some('*') => {
                    self.consume();
                    if self.peek() == Some('/') {
                        self.consume();
                        break;
                    } else {
                        content.push('*');
                    }
                }
                Some(c) => {
                    self.consume();
                    content.push(c);
                }
            }
        }
        
        CssToken::Comment(content)
    }

    fn consume_ident_sequence(&mut self) -> String {
        let mut result = String::new();
        
        loop {
            match self.peek() {
                Some('a'..='z') | Some('A'..='Z') | Some('0'..='9') | 
                Some('_') | Some('-') | Some('\u{80}'..=char::MAX) => {
                    result.push(self.consume().unwrap());
                }
                Some('\\') => {
                    self.consume();
                    if let Some(escaped) = self.consume_escape() {
                        result.push(escaped);
                    }
                }
                _ => break,
            }
        }
        
        result
    }

    fn consume_number(&mut self) -> NumberResult {
        let mut value_str = String::new();
        let mut is_integer = true;
        
        // Optional sign already handled by caller for -
        if self.peek() == Some('+') {
            self.consume();
        }
        
        // Integer part
        value_str.push_str(&self.consume_while(|c| c.is_ascii_digit()));
        
        // Decimal part
        if self.peek() == Some('.') && self.input[self.position..].chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
            value_str.push(self.consume().unwrap());
            value_str.push_str(&self.consume_while(|c| c.is_ascii_digit()));
            is_integer = false;
        }
        
        // Exponent part
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' {
                let exp_start = self.position;
                self.consume();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.consume();
                    }
                }
                let exp_digits = self.consume_while(|c| c.is_ascii_digit());
                if !exp_digits.is_empty() {
                    value_str.push('e');
                    value_str.push_str(&exp_digits);
                    is_integer = false;
                } else {
                    // Back up
                    self.position = exp_start;
                }
            }
        }
        
        let value: f64 = value_str.parse().unwrap_or(0.0);
        
        NumberResult {
            value,
            number_type: if is_integer { NumberType::Integer } else { NumberType::Number },
        }
    }

    fn consume_escape(&mut self) -> Option<char> {
        match self.peek() {
            None | Some('\n') | Some('\x0C') => None,
            Some('0'..='9') | Some('a'..='f') | Some('A'..='F') => {
                // Hex escape
                let hex = self.consume_while(|c| c.is_ascii_hexdigit());
                if hex.len() > 6 {
                    // Limit to 6 hex digits
                }
                let code = u32::from_str_radix(&hex, 16).ok()?;
                if code == 0 || (0xD800..=0xDFFF).contains(&code) || code > 0x10FFFF {
                    Some('\u{FFFD}') // Replacement character
                } else {
                    char::from_u32(code)
                }
            }
            Some(c) => {
                self.consume();
                Some(c)
            }
        }
    }

    fn would_start_ident(&mut self) -> bool {
        if let Some(c) = self.peek() {
            if c == '-' {
                // Check for -- or -ident
                let mut iter = self.input[self.position..].chars();
                iter.next(); // skip -
                match iter.next() {
                    Some('-') => return true,
                    Some(c) if Self::is_ident_start_char(c) => return true,
                    _ => {}
                }
            } else if Self::is_ident_start_char(c) || c == '\\' {
                return true;
            }
        }
        false
    }

    fn is_ident_start(&self, ch: char) -> bool {
        Self::is_ident_start_char(ch)
    }

    fn is_ident_start_char(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch >= '\u{80}'
    }
}

struct NumberResult {
    value: f64,
    number_type: NumberType,
}

/// Collect all tokens from input
#[allow(dead_code)]
pub fn tokenize(input: &str) -> Vec<CssToken> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut tokens = Vec::new();
    
    loop {
        let token = tokenizer.next_token();
        let is_eof = matches!(token, CssToken::EOF);
        tokens.push(token);
        if is_eof {
            break;
        }
    }
    
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident_token() {
        let tokens = tokenize("hello");
        assert_eq!(tokens[0], CssToken::Ident("hello".to_string()));
    }

    #[test]
    fn test_number_tokens() {
        let tokens = tokenize("42 3.14 -5 +10");
        assert_eq!(tokens[0], CssToken::Number(42.0, NumberType::Integer));
        assert_eq!(tokens[2], CssToken::Number(3.14, NumberType::Number));
    }

    #[test]
    fn test_dimension_token() {
        let tokens = tokenize("100px 50% 1.5em");
        assert_eq!(tokens[0], CssToken::Dimension(100.0, "px".to_string(), NumberType::Integer));
        assert_eq!(tokens[2], CssToken::Percentage(50.0));
        assert_eq!(tokens[4], CssToken::Dimension(1.5, "em".to_string(), NumberType::Number));
    }

    #[test]
    fn test_at_keyword() {
        let tokens = tokenize("@media @import @page");
        assert_eq!(tokens[0], CssToken::AtKeyword("media".to_string()));
        assert_eq!(tokens[2], CssToken::AtKeyword("import".to_string()));
        assert_eq!(tokens[4], CssToken::AtKeyword("page".to_string()));
    }

    #[test]
    fn test_hash_token() {
        let tokens = tokenize("#id #ff0000");
        assert!(matches!(tokens[0], CssToken::Hash(ref s, _) if s == "id"));
        assert!(matches!(tokens[2], CssToken::Hash(ref s, _) if s == "ff0000"));
    }

    #[test]
    fn test_string_token() {
        let tokens = tokenize(r#""hello" 'world'"#);
        assert_eq!(tokens[0], CssToken::String("hello".to_string()));
        assert_eq!(tokens[2], CssToken::String("world".to_string()));
    }

    #[test]
    fn test_function_token() {
        let tokens = tokenize("rgb( calc(");
        assert_eq!(tokens[0], CssToken::Function("rgb".to_string()));
        assert_eq!(tokens[2], CssToken::Function("calc".to_string()));
    }

    #[test]
    fn test_delimiters() {
        let tokens = tokenize("{}():;,>+~=");
        assert_eq!(tokens[0], CssToken::OpenBrace);
        assert_eq!(tokens[1], CssToken::CloseBrace);
        assert_eq!(tokens[2], CssToken::OpenParen);
        assert_eq!(tokens[3], CssToken::CloseParen);
        assert_eq!(tokens[4], CssToken::Colon);
        assert_eq!(tokens[5], CssToken::Semicolon);
        assert_eq!(tokens[6], CssToken::Comma);
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize("/* this is a comment */ ident");
        assert!(matches!(tokens[0], CssToken::Comment(_)));
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::Ident("ident".to_string()));
    }

    #[test]
    fn test_simple_rule() {
        let css = "body { color: red; }";
        let tokens = tokenize(css);
        
        assert_eq!(tokens[0], CssToken::Ident("body".to_string()));
        assert_eq!(tokens[1], CssToken::Whitespace);
        assert_eq!(tokens[2], CssToken::OpenBrace);
    }

    #[test]
    fn test_complex_selector() {
        let css = "div.container > p:first-child";
        let tokens = tokenize(css);
        
        assert_eq!(tokens[0], CssToken::Ident("div".to_string()));
        assert!(matches!(tokens[1], CssToken::Hash(_, _)));
    }
}

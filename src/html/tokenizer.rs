//! HTML5 Tokenizer
//!
//! Implements the tokenization rules from the WHATWG HTML5 specification

/// HTML Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_identifier: Option<String>,
        system_identifier: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attributes: Vec<super::Attribute>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Text(String),
    EndOfFile,
}

/// Tokenizer state machine states
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    // These are part of full HTML5 spec but not fully implemented
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    RawText,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
}

/// HTML Tokenizer
pub struct HtmlTokenizer<'a> {
    input: &'a str,
    position: usize,
    state: State,
    #[allow(dead_code)]
    return_state: Option<State>,
    #[allow(dead_code)]
    current_token: Option<Token>,
    current_tag_name: String,
    current_tag_self_closing: bool,
    current_attribute_name: String,
    current_attribute_value: String,
    attributes: Vec<super::Attribute>,
    temporary_buffer: String,
    current_comment: String,
    #[allow(dead_code)]
    character_reference_code: u32,
    last_emitted_start_tag: String,
}

impl<'a> HtmlTokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            state: State::Data,
            return_state: None,
            current_token: None,
            current_tag_name: String::new(),
            current_tag_self_closing: false,
            current_attribute_name: String::new(),
            current_attribute_value: String::new(),
            attributes: Vec::new(),
            temporary_buffer: String::new(),
            current_comment: String::new(),
            character_reference_code: 0,
            last_emitted_start_tag: String::new(),
        }
    }

    /// Peek at current character without consuming
    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    /// Consume and return current character
    fn consume(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    /// Check if next characters match a string
    fn peek_matches(&self, s: &str) -> bool {
        self.input[self.position..].starts_with(s)
    }

    /// Consume expected string
    #[allow(dead_code)]
    fn consume_str(&mut self, s: &str) -> bool {
        if self.peek_matches(s) {
            self.position += s.len();
            true
        } else {
            false
        }
    }

    /// Emit a token
    fn emit_token(&mut self, token: Token) -> Token {
        if let Token::StartTag { ref name, .. } = token {
            self.last_emitted_start_tag = name.clone();
        }
        token
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        loop {
            if let Some(token) = self.step() {
                return token;
            }
        }
    }

    /// Process one step of tokenization
    fn step(&mut self) -> Option<Token> {
        let ch = self.peek();

        match self.state {
            State::Data => {
                match ch {
                    Some('<') => {
                        self.consume();
                        self.state = State::TagOpen;
                        None
                    }
                    Some('&') => {
                        // Handle character references
                        self.consume();
                        self.state = State::Data; // Simplified
                        Some(Token::Text("&".to_string()))
                    }
                    None => Some(Token::EndOfFile),
                    Some(c) => {
                        self.consume();
                        Some(Token::Text(c.to_string()))
                    }
                }
            }

            State::TagOpen => {
                match ch {
                    Some('!') => {
                        self.consume();
                        self.state = State::MarkupDeclarationOpen;
                        None
                    }
                    Some('/') => {
                        self.consume();
                        self.state = State::EndTagOpen;
                        None
                    }
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.current_tag_name.clear();
                        self.attributes.clear();
                        self.current_tag_self_closing = false;
                        self.state = State::TagName;
                        None
                    }
                    Some('?') => {
                        self.consume();
                        self.state = State::BogusComment;
                        self.current_comment.clear();
                        None
                    }
                    _ => {
                        // Parse error, emit '<' as text
                        self.state = State::Data;
                        Some(Token::Text("<".to_string()))
                    }
                }
            }

            State::EndTagOpen => {
                match ch {
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.current_tag_name.clear();
                        self.state = State::TagName;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    _ => {
                        self.state = State::BogusComment;
                        self.current_comment.clear();
                        None
                    }
                }
            }

            State::TagName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('/') => {
                        self.consume();
                        self.state = State::SelfClosingStartTag;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: self.current_tag_self_closing,
                        };
                        Some(self.emit_token(token))
                    }
                    Some(c) if c.is_ascii_uppercase() => {
                        self.consume();
                        self.current_tag_name.push(c.to_ascii_lowercase());
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_tag_name.push(c);
                        None
                    }
                }
            }

            State::BeforeAttributeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        None
                    }
                    Some('/') | Some('>') | None => {
                        self.state = State::AfterAttributeName;
                        None
                    }
                    Some('=') => {
                        self.consume();
                        // Parse error
                        self.current_attribute_name.clear();
                        self.current_attribute_value.clear();
                        self.state = State::AttributeName;
                        None
                    }
                    _ => {
                        self.current_attribute_name.clear();
                        self.current_attribute_value.clear();
                        self.state = State::AttributeName;
                        None
                    }
                }
            }

            State::AttributeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('/') | Some('>') | None => {
                        self.state = State::AfterAttributeName;
                        None
                    }
                    Some('=') => {
                        self.consume();
                        self.state = State::BeforeAttributeValue;
                        None
                    }
                    Some(c) if c.is_ascii_uppercase() => {
                        self.consume();
                        self.current_attribute_name.push(c.to_ascii_lowercase());
                        None
                    }
                    Some(c) => {
                        self.consume();
                        self.current_attribute_name.push(c);
                        None
                    }
                }
            }

            State::AfterAttributeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        None
                    }
                    Some('=') => {
                        self.consume();
                        self.state = State::BeforeAttributeValue;
                        None
                    }
                    Some('/') => {
                        self.consume();
                        self.state = State::SelfClosingStartTag;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: self.current_tag_self_closing,
                        };
                        Some(self.emit_token(token))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    _ => {
                        self.current_attribute_name.clear();
                        self.current_attribute_value.clear();
                        self.state = State::AttributeName;
                        None
                    }
                }
            }

            State::BeforeAttributeValue => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        None
                    }
                    Some('"') => {
                        self.consume();
                        self.state = State::AttributeValueDoubleQuoted;
                        None
                    }
                    Some('\'') => {
                        self.consume();
                        self.state = State::AttributeValueSingleQuoted;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: self.current_tag_self_closing,
                        };
                        Some(self.emit_token(token))
                    }
                    _ => {
                        self.state = State::AttributeValueUnquoted;
                        None
                    }
                }
            }

            State::AttributeValueDoubleQuoted => {
                match ch {
                    Some('"') => {
                        self.consume();
                        self.state = State::AfterAttributeValueQuoted;
                        None
                    }
                    Some('&') => {
                        self.consume();
                        // Handle character reference
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_attribute_value.push(c);
                        None
                    }
                }
            }

            State::AttributeValueSingleQuoted => {
                match ch {
                    Some('\'') => {
                        self.consume();
                        self.state = State::AfterAttributeValueQuoted;
                        None
                    }
                    Some('&') => {
                        self.consume();
                        // Handle character reference
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_attribute_value.push(c);
                        None
                    }
                }
            }

            State::AttributeValueUnquoted => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('&') => {
                        self.consume();
                        // Handle character reference
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: self.current_tag_self_closing,
                        };
                        Some(self.emit_token(token))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_attribute_value.push(c);
                        None
                    }
                }
            }

            State::AfterAttributeValueQuoted => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('/') => {
                        self.consume();
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::SelfClosingStartTag;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: self.current_tag_self_closing,
                        };
                        Some(self.emit_token(token))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    _ => {
                        self.attributes.push(super::Attribute::new(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        ));
                        self.state = State::BeforeAttributeName;
                        None
                    }
                }
            }

            State::SelfClosingStartTag => {
                match ch {
                    Some('>') => {
                        self.consume();
                        self.current_tag_self_closing = true;
                        self.state = State::Data;
                        let token = Token::StartTag {
                            name: self.current_tag_name.clone(),
                            attributes: self.attributes.clone(),
                            self_closing: true,
                        };
                        Some(self.emit_token(token))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    _ => {
                        self.state = State::BeforeAttributeName;
                        None
                    }
                }
            }

            State::BogusComment => {
                match ch {
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let comment = std::mem::take(&mut self.current_comment);
                        Some(Token::Comment(comment))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push(c);
                        None
                    }
                }
            }

            State::MarkupDeclarationOpen => {
                if self.peek_matches("--") {
                    self.position += 2;
                    self.state = State::CommentStart;
                    self.current_comment.clear();
                    None
                } else if self.peek_matches("doctype") || self.peek_matches("DOCTYPE") {
                    self.position += 7;
                    self.state = State::Doctype;
                    None
                } else {
                    self.state = State::BogusComment;
                    self.current_comment.clear();
                    None
                }
            }

            State::CommentStart => {
                match ch {
                    Some('-') => {
                        self.consume();
                        self.state = State::CommentStartDash;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let comment = std::mem::take(&mut self.current_comment);
                        Some(Token::Comment(comment))
                    }
                    _ => {
                        self.state = State::Comment;
                        None
                    }
                }
            }

            State::CommentStartDash => {
                match ch {
                    Some('-') => {
                        self.consume();
                        self.state = State::CommentEnd;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let comment = std::mem::take(&mut self.current_comment);
                        Some(Token::Comment(comment))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push('-');
                        self.current_comment.push(c);
                        self.state = State::Comment;
                        None
                    }
                }
            }

            State::Comment => {
                match ch {
                    Some('<') => {
                        self.consume();
                        self.current_comment.push('<');
                        None
                    }
                    Some('-') => {
                        self.consume();
                        self.state = State::CommentEndDash;
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push(c);
                        None
                    }
                }
            }

            State::CommentEndDash => {
                match ch {
                    Some('-') => {
                        self.consume();
                        self.state = State::CommentEnd;
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push('-');
                        self.current_comment.push(c);
                        self.state = State::Comment;
                        None
                    }
                }
            }

            State::CommentEnd => {
                match ch {
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let comment = std::mem::take(&mut self.current_comment);
                        Some(Token::Comment(comment))
                    }
                    Some('!') => {
                        self.consume();
                        self.state = State::CommentEndBang;
                        None
                    }
                    Some('-') => {
                        self.consume();
                        self.current_comment.push('-');
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push('-');
                        self.current_comment.push('-');
                        self.current_comment.push(c);
                        self.state = State::Comment;
                        None
                    }
                }
            }

            State::CommentEndBang => {
                match ch {
                    Some('-') => {
                        self.consume();
                        self.current_comment.push('-');
                        self.current_comment.push('!');
                        self.state = State::CommentEndDash;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let comment = std::mem::take(&mut self.current_comment);
                        Some(Token::Comment(comment))
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        self.current_comment.push('-');
                        self.current_comment.push('!');
                        self.current_comment.push(c);
                        self.state = State::Comment;
                        None
                    }
                }
            }

            State::Doctype => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = State::BeforeDoctypeName;
                        None
                    }
                    Some('>') => {
                        self.state = State::BeforeDoctypeName;
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    _ => {
                        self.state = State::BeforeDoctypeName;
                        None
                    }
                }
            }

            State::BeforeDoctypeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        Some(Token::Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::Doctype {
                            name: None,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    Some(c) if c.is_ascii_uppercase() => {
                        self.consume();
                        self.current_tag_name.clear();
                        self.current_tag_name.push(c.to_ascii_lowercase());
                        self.state = State::DoctypeName;
                        None
                    }
                    Some(c) => {
                        self.consume();
                        self.current_tag_name.clear();
                        self.current_tag_name.push(c);
                        self.state = State::DoctypeName;
                        None
                    }
                }
            }

            State::DoctypeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        self.state = State::AfterDoctypeName;
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let name = std::mem::take(&mut self.current_tag_name);
                        Some(Token::Doctype {
                            name: Some(name),
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: false,
                        })
                    }
                    None => {
                        self.state = State::Data;
                        let name = std::mem::take(&mut self.current_tag_name);
                        Some(Token::Doctype {
                            name: Some(name),
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    Some(c) if c.is_ascii_uppercase() => {
                        self.consume();
                        self.current_tag_name.push(c.to_ascii_lowercase());
                        None
                    }
                    Some(c) => {
                        self.consume();
                        self.current_tag_name.push(c);
                        None
                    }
                }
            }

            State::AfterDoctypeName => {
                match ch {
                    Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                        self.consume();
                        None
                    }
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let name = std::mem::take(&mut self.current_tag_name);
                        Some(Token::Doctype {
                            name: Some(name),
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: false,
                        })
                    }
                    None => {
                        self.state = State::Data;
                        let name = std::mem::take(&mut self.current_tag_name);
                        Some(Token::Doctype {
                            name: Some(name),
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    _ => {
                        // Simplified - check for PUBLIC or SYSTEM
                        self.state = State::BogusDoctype;
                        None
                    }
                }
            }

            State::BogusDoctype => {
                match ch {
                    Some('>') => {
                        self.consume();
                        self.state = State::Data;
                        let name = if self.current_tag_name.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut self.current_tag_name))
                        };
                        Some(Token::Doctype {
                            name,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    None => {
                        self.state = State::Data;
                        let name = if self.current_tag_name.is_empty() {
                            None
                        } else {
                            Some(std::mem::take(&mut self.current_tag_name))
                        };
                        Some(Token::Doctype {
                            name,
                            public_identifier: None,
                            system_identifier: None,
                            force_quirks: true,
                        })
                    }
                    Some(_) => {
                        self.consume();
                        None
                    }
                }
            }

            // Raw text handling for script and style
            State::RawText => {
                match ch {
                    Some('<') => {
                        self.consume();
                        self.state = State::RawTextLessThanSign;
                        None
                    }
                    None => {
                        self.state = State::Data;
                        Some(Token::EndOfFile)
                    }
                    Some(c) => {
                        self.consume();
                        Some(Token::Text(c.to_string()))
                    }
                }
            }

            State::RawTextLessThanSign => {
                match ch {
                    Some('/') => {
                        self.consume();
                        self.temporary_buffer.clear();
                        self.state = State::RawTextEndTagOpen;
                        None
                    }
                    _ => {
                        self.state = State::RawText;
                        Some(Token::Text("<".to_string()))
                    }
                }
            }

            State::RawTextEndTagOpen => {
                match ch {
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.state = State::RawTextEndTagName;
                        None
                    }
                    _ => {
                        self.state = State::RawText;
                        Some(Token::Text("</".to_string()))
                    }
                }
            }

            State::RawTextEndTagName => {
                match ch {
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.consume();
                        self.temporary_buffer.push(c.to_ascii_lowercase());
                        None
                    }
                    _ => {
                        if self.temporary_buffer == self.last_emitted_start_tag {
                            // End tag matches
                            self.current_tag_name = std::mem::take(&mut self.temporary_buffer);
                            self.state = State::BeforeAttributeName;
                            None
                        } else {
                            self.state = State::RawText;
                            let text = format!("</{}", self.temporary_buffer);
                            Some(Token::Text(text))
                        }
                    }
                }
            }
            // Unimplemented states - treat as BogusDoctype for now
            _ => {
                self.state = State::BogusDoctype;
                None
            }
        }
    }
}

/// Character entity references (simplified)
#[allow(dead_code)]
pub fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some('\u{00A0}'),
        "copy" => Some('\u{00A9}'),
        "reg" => Some('\u{00AE}'),
        "trade" => Some('\u{2122}'),
        "mdash" => Some('\u{2014}'),
        "ndash" => Some('\u{2013}'),
        "hellip" => Some('\u{2026}'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_tag() {
        let mut tokenizer = HtmlTokenizer::new("<div>");
        assert!(matches!(tokenizer.next_token(), Token::StartTag { name, .. } if name == "div"));
    }

    #[test]
    fn test_tokenize_text() {
        let mut tokenizer = HtmlTokenizer::new("Hello");
        assert!(matches!(tokenizer.next_token(), Token::Text(t) if t == "H"));
    }

    #[test]
    fn test_tokenize_comment() {
        let mut tokenizer = HtmlTokenizer::new("<!-- comment -->");
        assert!(matches!(tokenizer.next_token(), Token::Comment(t) if t == " comment "));
    }

    #[test]
    fn test_tokenize_doctype() {
        let mut tokenizer = HtmlTokenizer::new("<!DOCTYPE html>");
        assert!(matches!(tokenizer.next_token(), Token::Doctype { name: Some(n), .. } if n == "html"));
    }

    #[test]
    fn test_tokenize_attributes() {
        let mut tokenizer = HtmlTokenizer::new(r#"<div id="test" class='foo' disabled>"#);
        
        if let Token::StartTag { name, attributes, .. } = tokenizer.next_token() {
            assert_eq!(name, "div");
            assert_eq!(attributes.len(), 3);
            assert!(attributes.iter().any(|a| a.name == "id" && a.value == "test"));
            assert!(attributes.iter().any(|a| a.name == "class" && a.value == "foo"));
            assert!(attributes.iter().any(|a| a.name == "disabled"));
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_tokenize_self_closing() {
        let mut tokenizer = HtmlTokenizer::new("<br/>");
        
        if let Token::StartTag { name, self_closing, .. } = tokenizer.next_token() {
            assert_eq!(name, "br");
            assert!(self_closing);
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_tokenize_end_tag() {
        let mut tokenizer = HtmlTokenizer::new("</div>");
        assert!(matches!(tokenizer.next_token(), Token::EndTag { name } if name == "div"));
    }

    #[test]
    fn test_decode_entity() {
        assert_eq!(decode_entity("amp"), Some('&'));
        assert_eq!(decode_entity("lt"), Some('<'));
        assert_eq!(decode_entity("gt"), Some('>'));
        assert_eq!(decode_entity("quot"), Some('"'));
        assert_eq!(decode_entity("unknown"), None);
    }
}

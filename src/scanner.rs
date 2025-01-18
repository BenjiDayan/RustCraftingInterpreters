use crate::token_type::TokenType;
use crate::token_type::Token;
use crate::token_type::Literal;
use crate::error;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self{
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1
        }
    }

    pub fn scanTokens(&mut self) -> Vec<Token> {
        while !self.isAtEnd() {
            self.start = self.current;
            self.scanToken();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            Literal::Nil,
            self.line)
        );
        self.tokens.clone()
    }

    fn isAtEnd(&self) -> bool {
        self.current > self.source.len()
    }

    fn scanToken(&mut self) {
        if let Some(c) = self.advance() {
            match c {
                '(' => self.add_token(TokenType::LEFT_PAREN),
                ')' => self.add_token(TokenType::RIGHT_PAREN),
                '{' => self.add_token(TokenType::LEFT_BRACE),
                '}' => self.add_token(TokenType::RIGHT_BRACE),
                ',' => self.add_token(TokenType::COMMA),
                '.' => self.add_token(TokenType::DOT),
                '-' => self.add_token(TokenType::MINUS),
                '+' => self.add_token(TokenType::PLUS),
                ';' => self.add_token(TokenType::SEMICOLON),
                '*' => self.add_token(TokenType::STAR),
                '!' => {
                    if self.match2('=') {
                        self.add_token(TokenType::BANG_EQUAL);
                    } else {
                        self.add_token(TokenType::BANG);
                    }
                },
                '=' => {
                    if self.match2('=') {
                        self.add_token(TokenType::EQUAL_EQUAL);
                    } else {
                        self.add_token(TokenType::EQUAL);
                    }
                },
                '<' => {
                    if self.match2('=') {
                        self.add_token(TokenType::LESS_EQUAL);
                    } else {
                        self.add_token(TokenType::LESS);
                    }
                },
                '>' => {
                    if self.match2('=') {
                        self.add_token(TokenType::GREATER_EQUAL);
                    } else {
                        self.add_token(TokenType::GREATER);
                    }
                },

                '/' => {
                    if self.match2('/') {
                        // comment - we skip until we hit a newline
                        // while self.peek() != '\n' && !self.isAtEnd() {
                        //     self.advance();
                        // }
                        while let Some(c) = self.advance() {
                            if c == '\n' {
                                self.line += 1;
                                break;
                            }
                        }
                    } else {
                        self.add_token(TokenType::SLASH);
                    }
                }
                ' ' | '\r' | '\t' => {
                    // ignore whitespace
                },
                '\n' => {
                    self.line += 1;
                },
                '"' => {
                    self.string();
                },

                _ => {
                    error(self.line, &format!("Unexpected character '{}'.", c))
                }
            }
        }
    }

    fn string (&mut self) {
        while self.peek() != '"' && !self.isAtEnd() {
            if (self.peek() == '\n') {  //support multiline strings!
                self.line += 1;
            }
            self.advance();
        }

        if self.isAtEnd() {
            error(self.line, "Unterminated string.");
        }

        // start is " , and so is current. hence +1 -1
        let value: String = self.source[self.start+1..self.current-1].to_string();
        self.add_token2(TokenType::STRING, Some(Literal::String(value)));

        
    }

    // this is like a conditional advance!
    fn match2(&mut self, expected: char) -> bool{
        if self.isAtEnd() { return false};
        if self.source.chars().nth(self.current) != Some(expected) { return false; }
        
        // otherwise, advance current and return true!
        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.isAtEnd() {
            return '\0';
        }
        // unwrap_or is a safe way to get the character at the current position,
        // or return '\0' if we're at the end
        // which should be impossible because we check isAtEnd() first
        self.source.chars().nth(self.current).unwrap_or('\0')
    }
    
    fn advance(&mut self) -> Option<char> {
        let my_char = self.source.chars().nth(self.current);
        self.current += 1;
        my_char
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token2(token_type, None)
    }

    fn add_token2(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::new(
            token_type,
            text.to_string(),
            literal.unwrap_or(Literal::Nil),
            self.line
        ));
    }
}

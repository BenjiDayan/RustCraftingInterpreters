use crate::token_type::TokenType;
use crate::token_type::Token;
use crate::token_type::Literal;
use crate::token_type::RESERVED_KEYWORDS;
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

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            Literal::Nil,
            self.line)
        );
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current > self.source.len()
    }

    fn scan_token(&mut self) {
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
                    if self.match_next('=') {
                        self.add_token(TokenType::BANG_EQUAL);
                    } else {
                        self.add_token(TokenType::BANG);
                    }
                },
                '=' => {
                    if self.match_next('=') {
                        self.add_token(TokenType::EQUAL_EQUAL);
                    } else {
                        self.add_token(TokenType::EQUAL);
                    }
                },
                '<' => {
                    if self.match_next('=') {
                        self.add_token(TokenType::LESS_EQUAL);
                    } else {
                        self.add_token(TokenType::LESS);
                    }
                },
                '>' => {
                    if self.match_next('=') {
                        self.add_token(TokenType::GREATER_EQUAL);
                    } else {
                        self.add_token(TokenType::GREATER);
                    }
                },

                '/' => {
                    if self.match_next('/') {
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
                }

                _ => {
                    if Scanner::is_digit(c) {
                        self.number();
                    } else if Scanner::is_alpha(c){
                        self.identifier();
                    } else {
                        error(self.line, &format!("Unexpected character '{}'.", c))
                    }
                }
            }
        }
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let id = self.current_to_string();
        if let Some(id_type) = RESERVED_KEYWORDS.get(id.as_str()) {
            self.add_token(id_type.clone())
        } else {
            self.add_token(TokenType::IDENTIFIER)
        }
    }

    fn current_to_string(&self) -> String {
        self.source[self.start..self.current].to_string()
    }


    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            self.advance();
            while Scanner::is_digit(self.peek()) {
                self.advance();
            } 
        }
        let number: f64 = self.current_to_string().parse().unwrap();
        self.add_token2(TokenType::NUMBER, Some(Literal::Number(number)))
    }

    fn string (&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {  //support multiline strings!
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.line, "Unterminated string.");
        }

        // the closing ".
        self.advance();

        // start is at " so + 1, and current is at "+1, so -1 to get the contents
        //"my_string" -> my_string
        let value: String = self.source[self.start+1..self.current-1].to_string();
        println!("{value}, {0}, {1}", self.start, self.current);
        self.add_token2(TokenType::STRING, Some(Literal::String(value)));
        println!("finished adding!");
        
    }

    // this is like a conditional advance!
    fn match_next(&mut self, expected: char) -> bool{
        if self.is_at_end() { return false};
        if self.source.chars().nth(self.current) != Some(expected) { return false; }
        
        // otherwise, advance current and return true!
        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        // unwrap_or is a safe way to get the character at the current position,
        // or return '\0' if we're at the end
        // which should be impossible because we check isAtEnd() first
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn peek_next(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.current += 1;
        let c = self.peek();
        self.current -= 1;
        return c;

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
        let text = self.current_to_string();
        self.tokens.push(Token::new(
            token_type,
            text,
            literal.unwrap_or(Literal::Nil),
            self.line
        ));
    }
}

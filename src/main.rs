use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Set,
    Var,
    If,
    Fi,  // elif
    El,  // else
    Print,
    Input,
    Pop,
    While,
    
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Json(String),
    
    // Identifiers and Types
    Identifier(String),
    Type(String),
    
    // Punctuation
    Semicolon,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftAngle,
    RightAngle,
    Colon,
    Equals,
    NotEquals,
    LessEquals,
    GreaterEquals,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    And,
    Or,
    Not,
    
    // Block delimiters
    BlockStart, // <-
    BlockEnd,   // ->
    
    // Comments
    Comment(String),
    
    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Json(j) => j.to_string(),
            Value::Null => "null".to_string(),
        }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Json(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        value: Expression,
        var_type: Option<String>,
    },
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        elif_clauses: Vec<(Expression, Vec<Statement>)>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Value(Value),
    Variable(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
    UnaryOp {
        op: String,
        expr: Box<Expression>,
    },
}

struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Self {
            input: chars,
            position: 0,
            current_char,
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // Skip closing quote
                break;
            }
            result.push(ch);
            self.advance();
        }
        
        result
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch.is_hangul() {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result
    }
    
    fn read_number(&mut self) -> f64 {
        let mut result = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '.' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        result.parse().unwrap_or(0.0)
    }
    
    fn read_comment(&mut self) -> String {
        let mut result = String::new();
        let is_multiline = self.peek(1) == Some('`') && self.peek(2) == Some('`');
        
        if is_multiline {
            // Skip ```
            self.advance(); // first `
            self.advance(); // second `
            self.advance(); // third `
            
            while let Some(ch) = self.current_char {
                if ch == '`' && self.peek(1) == Some('`') && self.peek(2) == Some('`') {
                    // Skip closing ```
                    self.advance();
                    self.advance();
                    self.advance();
                    break;
                }
                result.push(ch);
                self.advance();
            }
        } else {
            // Single line comment
            self.advance(); // Skip opening `
            
            while let Some(ch) = self.current_char {
                if ch == '`' {
                    self.advance(); // Skip closing `
                    break;
                }
                result.push(ch);
                self.advance();
            }
        }
        
        result
    }
    
    fn read_json(&mut self) -> String {
        let mut result = String::new();
        let mut brace_count = 0;
        
        while let Some(ch) = self.current_char {
            result.push(ch);
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    self.advance();
                    break;
                }
            }
            self.advance();
        }
        
        result
    }
    
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => self.skip_whitespace(),
                '\n' => {
                    tokens.push(Token::Newline);
                    self.advance();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                '[' => {
                    tokens.push(Token::LeftBracket);
                    self.advance();
                }
                ']' => {
                    tokens.push(Token::RightBracket);
                    self.advance();
                }
                '(' => {
                    if self.peek(1) == Some('{') {
                        self.advance(); // consume '('
                        let json_str = self.read_json();
                        tokens.push(Token::Json(json_str));
                        self.skip_whitespace();
                        if self.current_char == Some(')') {
                            self.advance(); // consume ')'
                        }
                    } else {
                        tokens.push(Token::LeftParen);
                        self.advance();
                    }
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                '{' => {
                    tokens.push(Token::LeftBrace);
                    self.advance();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    self.advance();
                }
                '<' => {
                    if self.peek(1) == Some('-') {
                        tokens.push(Token::BlockStart);
                        self.advance();
                        self.advance();
                    } else if self.peek(1) == Some('=') {
                        tokens.push(Token::LessEquals);
                        self.advance();
                        self.advance();
                    } else {
                        tokens.push(Token::LeftAngle);
                        self.advance();
                    }
                }
                '>' => {
                    if self.peek(1) == Some('-') {
                        tokens.push(Token::BlockEnd);
                        self.advance();
                        self.advance();
                    } else if self.peek(1) == Some('=') {
                        tokens.push(Token::GreaterEquals);
                        self.advance();
                        self.advance();
                    } else {
                        tokens.push(Token::RightAngle);
                        self.advance();
                    }
                }
                '-' => {
                    if self.peek(1) == Some('>') {
                        tokens.push(Token::BlockEnd);
                        self.advance();
                        self.advance();
                    } else {
                        tokens.push(Token::Minus);
                        self.advance();
                    }
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equals);
                    self.advance();
                }
                '!' => {
                    if self.peek(1) == Some('=') {
                        tokens.push(Token::NotEquals);
                        self.advance();
                        self.advance();
                    } else {
                        tokens.push(Token::Not);
                        self.advance();
                    }
                }
                '+' => {
                    tokens.push(Token::Plus);
                    self.advance();
                }
                '*' => {
                    tokens.push(Token::Star);
                    self.advance();
                }
                '/' => {
                    tokens.push(Token::Slash);
                    self.advance();
                }
                '%' => {
                    tokens.push(Token::Percent);
                    self.advance();
                }
                '"' => {
                    let string_value = self.read_string('"');
                    tokens.push(Token::String(string_value));
                }
                '`' => {
                    let comment = self.read_comment();
                    tokens.push(Token::Comment(comment));
                }
                _ if ch.is_ascii_digit() => {
                    let number = self.read_number();
                    tokens.push(Token::Number(number));
                }
                _ if ch.is_alphabetic() || ch.is_hangul() => {
                    let identifier = self.read_identifier();
                    let token = match identifier.to_lowercase().as_str() {
                        "set" => Token::Set,
                        "var" => Token::Var,
                        "if" => Token::If,
                        "fi" => Token::Fi,
                        "el" => Token::El,
                        "print" => Token::Print,
                        "input" => Token::Input,
                        "pop" => Token::Pop,
                        "wh" => Token::While,
                        "and" => Token::And,
                        "or" => Token::Or,
                        "true" | "True" | "TRUE" | "tRuE" | "TrUe" => Token::Boolean(true),
                        "false" | "False" | "FALSE" | "fAlSe" => Token::Boolean(false),
                        "str" => Token::Type("str".to_string()),
                        _ => Token::Identifier(identifier),
                    };
                    tokens.push(token);
                }
                _ => {
                    // Skip unknown characters
                    self.advance();
                }
            }
        }
        
        tokens.push(Token::Eof);
        tokens
    }
}

trait IsHangul {
    fn is_hangul(&self) -> bool;
}

impl IsHangul for char {
    fn is_hangul(&self) -> bool {
        let code = *self as u32;
        // Hangul Syllables block (AC00-D7AF)
        // Hangul Jamo block (1100-11FF)
        // Hangul Compatibility Jamo block (3130-318F)
        (code >= 0xAC00 && code <= 0xD7AF) ||
        (code >= 0x1100 && code <= 0x11FF) ||
        (code >= 0x3130 && code <= 0x318F)
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_token: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let current_token = tokens.get(0).cloned().unwrap_or(Token::Eof);
        Self {
            tokens,
            position: 0,
            current_token,
        }
    }
    
    fn advance(&mut self) {
        self.position += 1;
        self.current_token = self.tokens.get(self.position).cloned().unwrap_or(Token::Eof);
    }
    
    
    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token))
        }
    }
    
    fn skip_newlines(&mut self) {
        while self.current_token == Token::Newline {
            self.advance();
        }
    }
    
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        
        while self.current_token != Token::Eof {
            self.skip_newlines();
            
            if self.current_token == Token::Eof {
                break;
            }
            
            // Skip comments
            if let Token::Comment(_) = self.current_token {
                self.advance();
                continue;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> Result<Statement, String> {
        match &self.current_token {
            Token::Set => self.parse_variable_declaration(false),
            Token::Var => self.parse_variable_declaration(true),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::LeftAngle => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Statement::Expression(expr))
            }
        }
    }
    
    fn parse_variable_declaration(&mut self, typed: bool) -> Result<Statement, String> {
        self.advance(); // Skip 'set' or 'var'
        self.expect(Token::Semicolon)?;
        self.expect(Token::LeftBracket)?;
        
        let name = if let Token::String(name) = &self.current_token {
            name.clone()
        } else {
            return Err("Expected variable name".to_string());
        };
        self.advance();
        
        self.expect(Token::RightBracket)?;
        self.expect(Token::Colon)?;
        
        let var_type = if typed {
            self.expect(Token::LeftAngle)?;
            let type_name = if let Token::Type(t) = &self.current_token {
                Some(t.clone())
            } else {
                None
            };
            self.advance();
            self.expect(Token::RightAngle)?;
            self.expect(Token::Semicolon)?;
            
            // For typed variables, expect <(value)> format
            self.expect(Token::LeftAngle)?;
            self.expect(Token::LeftParen)?;
            let value = self.parse_primary_value()?;
            self.expect(Token::RightParen)?;
            self.expect(Token::RightAngle)?;
            
            return Ok(Statement::VariableDeclaration {
                name,
                value: Expression::Value(value),
                var_type: type_name,
            });
        } else {
            None
        };
        
        let value = self.parse_expression()?;
        
        Ok(Statement::VariableDeclaration {
            name,
            value,
            var_type,
        })
    }
    
    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // Skip 'if'
        
        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RightBrace)?;
        
        self.skip_newlines();
        self.expect(Token::BlockStart)?; // <-
        let then_body = self.parse_block()?;
        self.expect(Token::BlockEnd)?; // ->
        
        let mut elif_clauses = Vec::new();
        let mut else_body = None;
        
        // Parse elif clauses
        self.skip_newlines();
        while self.current_token == Token::Fi {
            self.advance(); // Skip 'fi'
            
            self.skip_newlines();
            self.expect(Token::LeftBrace)?;
            let elif_condition = self.parse_expression()?;
            self.expect(Token::RightBrace)?;
            
            self.skip_newlines();
            self.expect(Token::BlockStart)?;
            let elif_body = self.parse_block()?;
            self.expect(Token::BlockEnd)?;
            
            elif_clauses.push((elif_condition, elif_body));
            self.skip_newlines();
        }
        
        // Parse else clause
        if self.current_token == Token::El {
            self.advance(); // Skip 'el'
            
            self.skip_newlines();
            self.expect(Token::LeftBrace)?;
            self.expect(Token::RightBrace)?;
            
            self.skip_newlines();
            self.expect(Token::BlockStart)?;
            else_body = Some(self.parse_block()?);
            self.expect(Token::BlockEnd)?;
        }
        
        Ok(Statement::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        })
    }
    
    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.advance(); // Skip 'wh'
        
        self.skip_newlines();
        self.expect(Token::LeftBrace)?;
        let condition = self.parse_expression()?;
        self.expect(Token::RightBrace)?;
        
        self.skip_newlines();
        self.expect(Token::BlockStart)?; // <-
        let body = self.parse_block()?;
        self.expect(Token::BlockEnd)?; // ->
        
        Ok(Statement::While {
            condition,
            body,
        })
    }
    
    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        
        while self.current_token != Token::BlockEnd && self.current_token != Token::Eof {
            self.skip_newlines();
            
            if self.current_token == Token::BlockEnd || self.current_token == Token::Eof {
                break;
            }
            
            // Skip comments
            if let Token::Comment(_) = self.current_token {
                self.advance();
                continue;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        
        Ok(statements)
    }
    
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;
        
        while self.current_token == Token::Or {
            self.advance();
            let right = self.parse_and()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: "or".to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;
        
        while self.current_token == Token::And {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op: "and".to_string(),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;
        
        while self.current_token == Token::Equals || self.current_token == Token::NotEquals {
            let op = if self.current_token == Token::Equals {
                "=".to_string()
            } else {
                "!=".to_string()
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_additive()?;
        
        while self.current_token == Token::LeftAngle
            || self.current_token == Token::RightAngle
            || self.current_token == Token::LessEquals
            || self.current_token == Token::GreaterEquals
        {
            let op = match &self.current_token {
                Token::LeftAngle => "<".to_string(),
                Token::RightAngle => ">".to_string(),
                Token::LessEquals => "<=".to_string(),
                Token::GreaterEquals => ">=".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_additive()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_multiplicative()?;
        
        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = if self.current_token == Token::Plus {
                "+".to_string()
            } else {
                "-".to_string()
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;
        
        while self.current_token == Token::Star
            || self.current_token == Token::Slash
            || self.current_token == Token::Percent
        {
            let op = match &self.current_token {
                Token::Star => "*".to_string(),
                Token::Slash => "/".to_string(),
                Token::Percent => "%".to_string(),
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.current_token == Token::Not {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op: "!".to_string(),
                expr: Box::new(expr),
            })
        } else if self.current_token == Token::Minus {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op: "-".to_string(),
                expr: Box::new(expr),
            })
        } else {
            self.parse_primary()
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expression, String> {
        match &self.current_token.clone() {
            Token::String(s) => {
                let expr = Expression::Value(Value::String(s.clone()));
                self.advance();
                Ok(expr)
            }
            Token::Number(n) => {
                let expr = Expression::Value(Value::Number(*n));
                self.advance();
                Ok(expr)
            }
            Token::Boolean(b) => {
                let expr = Expression::Value(Value::Boolean(*b));
                self.advance();
                Ok(expr)
            }
            Token::Json(j) => {
                let json_value: serde_json::Value = serde_json::from_str(j)
                    .map_err(|_| "Invalid JSON")?;
                let expr = Expression::Value(Value::Json(json_value));
                self.advance();
                Ok(expr)
            }
            Token::Identifier(name) => {
                let expr = Expression::Variable(name.clone());
                self.advance();
                Ok(expr)
            }
            Token::LeftParen => {
                self.advance(); // Skip '('
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftAngle => {
                self.advance(); // Skip '<'
                let func_call = self.parse_function_call()?;
                self.expect(Token::RightAngle)?;
                Ok(func_call)
            }
            _ => Err(format!("Unexpected token: {:?}", self.current_token))
        }
    }
    
    fn parse_primary_value(&mut self) -> Result<Value, String> {
        match &self.current_token.clone() {
            Token::String(s) => {
                let value = Value::String(s.clone());
                self.advance();
                Ok(value)
            }
            Token::Number(n) => {
                let value = Value::Number(*n);
                self.advance();
                Ok(value)
            }
            Token::Boolean(b) => {
                let value = Value::Boolean(*b);
                self.advance();
                Ok(value)
            }
            Token::Json(j) => {
                let json_value: serde_json::Value = serde_json::from_str(j)
                    .map_err(|_| "Invalid JSON")?;
                let value = Value::Json(json_value);
                self.advance();
                Ok(value)
            }
            _ => Err(format!("Expected value, found: {:?}", self.current_token))
        }
    }
    
    fn parse_function_call(&mut self) -> Result<Expression, String> {
        let name = match &self.current_token {
            Token::Print => "print".to_string(),
            Token::Input => "input".to_string(),
            Token::Pop => "pop".to_string(),
            Token::Identifier(name) => name.clone(),
            _ => return Err("Expected function name".to_string()),
        };
        self.advance();
        
        let mut args = Vec::new();
        
        // Handle different function call patterns
        match name.as_str() {
            "print" => {
                self.expect(Token::Colon)?;
                self.expect(Token::LeftParen)?;
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                args.push(expr);
            }
            "input" => {
                // input;print:("prompt")
                if self.current_token == Token::Semicolon {
                    self.advance();
                    if self.current_token == Token::Print {
                        self.advance();
                        self.expect(Token::Colon)?;
                        self.expect(Token::LeftParen)?;
                        let expr = self.parse_expression()?;
                        self.expect(Token::RightParen)?;
                        args.push(expr);
                    }
                }
            }
            "pop" => {
                // pop can be standalone or with arguments
            }
            _ => {
                // Generic function call or variable reference
                while self.current_token != Token::RightAngle && self.current_token != Token::Eof {
                    args.push(self.parse_expression()?);
                    if self.current_token == Token::Semicolon {
                        self.advance();
                    } else {
                        break;
                    }
                }
                if args.is_empty() {
                    return Ok(Expression::Variable(name));
                }
            }
        }
        
        Ok(Expression::FunctionCall { name, args })
    }
}

struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }
    
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }
    
    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }
    
    pub fn set(&mut self, name: String, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }
        self.scopes.last_mut().unwrap().insert(name, value);
    }
}

struct Interpreter {
    environment: Environment,
    last_input: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            last_input: None,
        }
    }
    
    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::VariableDeclaration { name, value, var_type: _ } => {
                let val = self.evaluate_expression(value)?;
                self.environment.set(name, val);
            }
            Statement::If { condition, then_body, elif_clauses, else_body } => {
                let condition_value = self.evaluate_expression(condition)?;
                
                if condition_value.is_truthy() {
                    self.environment.enter_scope();
                    for stmt in then_body {
                        self.execute_statement(stmt)?;
                    }
                    self.environment.exit_scope();
                } else {
                    let mut executed = false;
                    for (elif_condition, elif_body) in elif_clauses {
                        let elif_value = self.evaluate_expression(elif_condition)?;
                        if elif_value.is_truthy() {
                            self.environment.enter_scope();
                            for stmt in elif_body {
                                self.execute_statement(stmt)?;
                            }
                            self.environment.exit_scope();
                            executed = true;
                            break;
                        }
                    }
                    
                    if !executed {
                        if let Some(else_stmts) = else_body {
                            self.environment.enter_scope();
                            for stmt in else_stmts {
                                self.execute_statement(stmt)?;
                            }
                            self.environment.exit_scope();
                        }
                    }
                }
            }
            Statement::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate_expression(condition.clone())?;
                    if !condition_value.is_truthy() {
                        break;
                    }
                    self.environment.enter_scope();
                    for stmt in body.clone() {
                        self.execute_statement(stmt)?;
                    }
                    self.environment.exit_scope();
                }
            }
            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
            }
        }
        Ok(())
    }
    
    fn evaluate_expression(&mut self, expression: Expression) -> Result<Value, String> {
        match expression {
            Expression::Value(val) => Ok(val),
            Expression::Variable(name) => {
                self.environment.get(&name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expression::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            Expression::UnaryOp { op, expr } => {
                let val = self.evaluate_expression(*expr)?;
                match op.as_str() {
                    "!" => Ok(Value::Boolean(!val.is_truthy())),
                    "-" => match val {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err("Unary minus requires a number".to_string()),
                    },
                    _ => Err(format!("Unknown unary operator: {}", op)),
                }
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(*left)?;
                let right_val = self.evaluate_expression(*right)?;
                
                match op.as_str() {
                    "=" => Ok(Value::Boolean(self.values_equal(&left_val, &right_val))),
                    "!=" => Ok(Value::Boolean(!self.values_equal(&left_val, &right_val))),
                    "<" => Ok(Value::Boolean(self.compare_numbers(&left_val, &right_val)? < 0.0)),
                    ">" => Ok(Value::Boolean(self.compare_numbers(&left_val, &right_val)? > 0.0)),
                    "<=" => Ok(Value::Boolean(self.compare_numbers(&left_val, &right_val)? <= 0.0)),
                    ">=" => Ok(Value::Boolean(self.compare_numbers(&left_val, &right_val)? >= 0.0)),
                    "+" => self.add_values(&left_val, &right_val),
                    "-" => self.subtract_values(&left_val, &right_val),
                    "*" => self.multiply_values(&left_val, &right_val),
                    "/" => self.divide_values(&left_val, &right_val),
                    "%" => self.modulo_values(&left_val, &right_val),
                    "and" => Ok(Value::Boolean(left_val.is_truthy() && right_val.is_truthy())),
                    "or" => Ok(Value::Boolean(left_val.is_truthy() || right_val.is_truthy())),
                    _ => Err(format!("Unknown operator: {}", op)),
                }
            }
        }
    }
    
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
    
    fn compare_numbers(&self, left: &Value, right: &Value) -> Result<f64, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(a - b),
            _ => Err("Comparison requires numbers".to_string()),
        }
    }
    
    fn add_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err("Invalid operands for +".to_string()),
        }
    }
    
    fn subtract_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err("Invalid operands for -".to_string()),
        }
    }
    
    fn multiply_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err("Invalid operands for *".to_string()),
        }
    }
    
    fn divide_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err("Invalid operands for /".to_string()),
        }
    }
    
    fn modulo_values(&self, left: &Value, right: &Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err("Modulo by zero".to_string())
                } else {
                    Ok(Value::Number(a % b))
                }
            }
            _ => Err("Invalid operands for %".to_string()),
        }
    }
    
    fn call_function(&mut self, name: String, args: Vec<Expression>) -> Result<Value, String> {
        match name.as_str() {
            "print" => {
                if args.len() != 1 {
                    return Err("print expects exactly one argument".to_string());
                }
                let value = self.evaluate_expression(args[0].clone())?;
                println!("{}", value.to_string());
                Ok(Value::Null)
            }
            "input" => {
                if !args.is_empty() {
                    let prompt = self.evaluate_expression(args[0].clone())?;
                    print!("{}", prompt.to_string());
                    io::stdout().flush().unwrap();
                }
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)
                    .map_err(|_| "Failed to read input".to_string())?;
                
                let input = input.trim().to_string();
                let value = Value::String(input);
                self.last_input = Some(value.clone());
                Ok(value)
            }
            "pop" => {
                // Return the last input value
                Ok(self.last_input.clone().unwrap_or(Value::Null))
            }
            _ => Err(format!("Unknown function: {}", name))
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <file.bara>", args[0]);
        process::exit(1);
    }
    
    let filename = &args[1];
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file {}: {}", filename, err);
            process::exit(1);
        }
    };
    
    // Lexical analysis
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize();
    
    // Parsing
    let mut parser = Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };
    
    // Interpretation
    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.interpret(statements) {
        eprintln!("Runtime error: {}", err);
        process::exit(1);
    }
}

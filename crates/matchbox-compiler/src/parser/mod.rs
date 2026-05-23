use crate::ast::*;
use crate::tokenizer::*;
use anyhow::{bail, Result};

pub mod template;

pub fn parse_bxm(source: &str, filename: Option<&str>) -> Result<Vec<Statement>> {
    template::parse_template(source, filename)
}

pub fn parse(source: &str, _filename: Option<&str>) -> Result<Vec<Statement>> {
    let lexed = lex(source);
    Parser::new(source, lexed.tokens()).parse_program()
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [SyntaxToken],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, tokens: &'a [SyntaxToken]) -> Self {
        Self { source, tokens, pos: 0 }
    }

    fn kind(&self, offset: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| t.kind)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.kind(0)
    }

    fn peek_lexeme(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|t| &self.source[t.span.start..t.span.end])
    }

    fn peek_line(&self) -> u32 {
        self.tokens.get(self.pos).map(|t| t.span.line).unwrap_or(0)
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn advance(&mut self) -> Option<TokenKind> {
        let kind = self.tokens.get(self.pos).map(|t| t.kind);
        self.pos += 1;
        kind
    }

    fn advance_lexeme(&mut self) -> Option<String> {
        let lexeme = self
            .tokens
            .get(self.pos)
            .map(|t| self.source[t.span.start..t.span.end].to_string());
        self.pos += 1;
        lexeme
    }

    fn binary_operator_lexeme(&self, kind: TokenKind, raw: &str) -> String {
        match kind {
            TokenKind::PipePipe => "||".to_string(),
            TokenKind::AmpAmp => "&&".to_string(),
            TokenKind::EqualEqual => "==".to_string(),
            TokenKind::BangEqual => "!=".to_string(),
            TokenKind::Less => "<".to_string(),
            TokenKind::Greater => ">".to_string(),
            TokenKind::LessEqual => "<=".to_string(),
            TokenKind::GreaterEqual => ">=".to_string(),
            TokenKind::Contains => "contains".to_string(),
            TokenKind::InstanceOf => "instanceof".to_string(),
            TokenKind::CastAs => "castas".to_string(),
            TokenKind::Xor => "xor".to_string(),
            TokenKind::Eqv => "eqv".to_string(),
            _ => raw.to_string(),
        }
    }

    fn token_text_lower(&self, offset: usize) -> Option<String> {
        self.tokens
            .get(self.pos + offset)
            .map(|t| self.source[t.span.start..t.span.end].to_ascii_lowercase())
    }

    fn phrase_operator(&self) -> Option<(u8, String, usize)> {
        let current = self.peek_kind()?;
        let current_text = self.token_text_lower(0)?;
        match current {
            TokenKind::Not => {
                if matches!(self.kind(1), Some(TokenKind::Contains)) {
                    Some((3, "not contains".to_string(), 2))
                } else {
                    None
                }
            }
            TokenKind::Identifier if current_text == "does" => {
                if matches!(self.kind(1), Some(TokenKind::Not))
                    && matches!(self.kind(2), Some(TokenKind::Contains) | Some(TokenKind::Identifier))
                    && self.token_text_lower(2).as_deref().is_some_and(|t| t == "contain" || t == "contains")
                {
                    Some((3, "not contains".to_string(), 3))
                } else {
                    None
                }
            }
            TokenKind::Identifier if current_text == "less" || current_text == "greater" => {
                if self.token_text_lower(1).as_deref() != Some("than") {
                    return None;
                }
                let base_op = if current_text == "less" { "<" } else { ">" };
                if self.token_text_lower(2).as_deref() == Some("or")
                    && matches!(self.kind(3), Some(TokenKind::EqualEqual))
                    && self.token_text_lower(3).as_deref().is_some_and(|t| t == "eq" || t == "equal" || t == "is")
                    && self.token_text_lower(4).as_deref() == Some("to")
                {
                    Some((3, format!("{base_op}="), 5))
                } else {
                    Some((3, base_op.to_string(), 2))
                }
            }
            TokenKind::EqualEqual if current_text == "is" => {
                if matches!(self.kind(1), Some(TokenKind::Not)) {
                    Some((3, "!=".to_string(), 2))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn advance_line(&mut self) -> u32 {
        let line = self.peek_line();
        self.pos += 1;
        line
    }

    fn expect(&mut self, kind: TokenKind) -> Result<()> {
        if self.peek_kind() == Some(kind) {
            self.pos += 1;
            Ok(())
        } else {
            let found = self.peek_kind();
            bail!("Expected {:?}, found {:?} at line {}", kind, found, self.peek_line());
        }
    }

    fn expect_get(&mut self, kind: TokenKind) -> Result<String> {
        if self.peek_kind() == Some(kind) {
            let lexeme = self.peek_lexeme().unwrap_or("").to_string();
            self.pos += 1;
            Ok(lexeme)
        } else {
            bail!("Expected {:?}, found {:?}", kind, self.peek_kind());
        }
    }

    fn parse_program(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        while self.peek_kind().is_some() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let line = self.peek_line();

        match self.peek_kind() {
            Some(TokenKind::Import) => self.parse_import(line),
            Some(TokenKind::Class) => self.parse_class(line),
            Some(TokenKind::Interface) => self.parse_interface(line),
            Some(TokenKind::Function) | Some(TokenKind::At)
            | Some(TokenKind::Public) | Some(TokenKind::Private)
            | Some(TokenKind::Remote) | Some(TokenKind::Package)
            | Some(TokenKind::Static) | Some(TokenKind::Abstract) | Some(TokenKind::Final)
                if self.is_function_decl() =>
            {
                self.parse_function_decl(line)
            }
            Some(TokenKind::For) => self.parse_for(line),
            Some(TokenKind::While) => self.parse_while(line),
            Some(TokenKind::Do) => self.parse_do_while(line),
            Some(TokenKind::If) => self.parse_if(line),
            Some(TokenKind::Try) => self.parse_try(line),
            Some(TokenKind::Return) => self.parse_return(line),
            Some(TokenKind::Throw) => self.parse_throw(line),
            Some(TokenKind::Rethrow) => self.parse_rethrow(line),
            Some(TokenKind::Assert) => self.parse_assert(line),
            Some(TokenKind::Param) => self.parse_param(line),
            Some(TokenKind::Include) => self.parse_include(line),
            Some(TokenKind::Not) => self.parse_not_stmt(line),
            Some(TokenKind::Continue) => self.parse_continue(line),
            Some(TokenKind::Break) => self.parse_break(line),
            Some(TokenKind::Switch) => self.parse_switch(line),
            Some(TokenKind::Var) => self.parse_var_decl(line),
            Some(TokenKind::LeftBrace) | Some(TokenKind::LeftBracket)
                if self.is_destructure_assignment() =>
            {
                return self.parse_destructure_assignment(line);
            }
            Some(_) => {
                let expr = self.parse_expression()?;
                // Consume optional semicolon
                if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
                Ok(Statement::new(StatementKind::Expression(expr), line))
            }
            None => bail!("Unexpected EOF"),
        }
    }

    fn is_function_decl(&self) -> bool {
        let mut i = self.pos;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                TokenKind::At | TokenKind::Public | TokenKind::Private | TokenKind::Remote
                | TokenKind::Package | TokenKind::Static | TokenKind::Abstract | TokenKind::Final => {
                    i += 1;
                    continue;
                }
                TokenKind::Function => return true,
                TokenKind::Identifier => {
                    i += 1;
                    if i < self.tokens.len() && matches!(
                        self.tokens[i].kind, TokenKind::Function | TokenKind::Identifier
                    ) {
                        // Handle "returnType function" or "Public returnType function" patterns
                        if self.tokens[i].kind == TokenKind::Function {
                            return true;
                        }
                        // If it's another identifier, skip and continue (skip type name)
                        continue;
                    }
                    return false;
                }
                _ => return false,
            }
        }
        false
    }

    // ---- Statement parsers ----

    fn parse_import(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // import
        let mut path = String::new();
        // Optional prefix (js:, java:, rust:)
        if self.kind(0) == Some(TokenKind::Identifier) && self.kind(1) == Some(TokenKind::Colon) {
            path.push_str(&self.advance_lexeme().unwrap_or_default());
            path.push(':');
            self.pos += 1; // colon
        }
        // Dotted path
        loop {
            path.push_str(&self.expect_get(TokenKind::Identifier)?);
            if self.peek_is(TokenKind::Dot) {
                path.push('.');
                self.pos += 1;
            } else {
                break;
            }
        }
        let alias = if self.peek_is(TokenKind::As) {
            self.pos += 1; // as
            Some(self.expect_get(TokenKind::Identifier)?)
        } else {
            None
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Import { path, alias }, line))
    }

    fn parse_class(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // class
        let name = if self.peek_is(TokenKind::Identifier) {
            self.advance_lexeme().unwrap_or_default()
        } else {
            String::new()
        };
        let mut extends = None;
        let mut accessors = false;
        let mut implements = Vec::new();

        while matches!(self.peek_kind(),
            Some(TokenKind::Extends) | Some(TokenKind::Accessors) | Some(TokenKind::Implements)
            | Some(TokenKind::Identifier)
        ) {
            let attr_name = self.peek_lexeme().unwrap_or("").to_string();
            if matches!(attr_name.as_str(), "extends" | "accessors" | "implements") {
                self.pos += 1; // attr name
                self.expect(TokenKind::Equal)?;
                let val = if self.peek_kind() == Some(TokenKind::String) {
                    let s = self.peek_lexeme().unwrap_or("").to_string();
                    self.pos += 1;
                    if s.len() >= 2 { s[1..s.len() - 1].to_string() } else { s }
                } else {
                    bail!("Expected string value for '{}'", attr_name);
                };
                match attr_name.as_str() {
                    "extends" => extends = Some(val),
                    "accessors" => accessors = val.to_lowercase() == "true",
                    "implements" => implements = val.split(',').map(|s| s.trim().to_string()).collect(),
                    _ => {}
                }
            } else {
                break;
            }
        }

        self.expect(TokenKind::LeftBrace)?;
        let mut members = Vec::new();
        while !self.peek_is(TokenKind::RightBrace) {
            if self.peek_is(TokenKind::Property) {
                self.pos += 1; // property
                let prop_name = self.expect_get(TokenKind::Identifier)?;
                members.push(ClassMember::Property(prop_name));
                if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
            } else {
                members.push(ClassMember::Statement(self.parse_statement()?));
            }
        }
        self.pos += 1; // }
        Ok(Statement::new(
            StatementKind::ClassDecl { name, extends, accessors, implements, members },
            line,
        ))
    }

    fn parse_interface(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // interface
        let name = if self.peek_is(TokenKind::Identifier) {
            self.advance_lexeme().unwrap_or_default()
        } else {
            String::new()
        };
        self.expect(TokenKind::LeftBrace)?;
        let mut members = Vec::new();
        while !self.peek_is(TokenKind::RightBrace) {
            members.push(self.parse_statement()?);
        }
        self.pos += 1; // }
        Ok(Statement::new(StatementKind::InterfaceDecl { name, members }, line))
    }

    fn parse_function_decl(&mut self, line: u32) -> Result<Statement> {
        let mut attributes = Vec::new();
        while self.peek_is(TokenKind::At) {
            self.pos += 1; // @
            let attr_name = self.expect_get(TokenKind::Identifier)?;
            let mut args = Vec::new();
            if self.peek_is(TokenKind::LeftParen) {
                self.pos += 1; // (
                args = self.parse_args()?;
                self.expect(TokenKind::RightParen)?;
            }
            attributes.push(Attribute { name: attr_name, args });
        }

        let access_modifier = if matches!(self.peek_kind(),
            Some(TokenKind::Public) | Some(TokenKind::Private)
            | Some(TokenKind::Remote) | Some(TokenKind::Package)
        ) {
            Some(self.advance_lexeme().unwrap_or_default())
        } else {
            None
        };

        // Consume non-access modifiers: static, abstract, final
        while matches!(self.peek_kind(),
            Some(TokenKind::Static) | Some(TokenKind::Abstract) | Some(TokenKind::Final)
        ) {
            self.pos += 1;
        }

        let return_type = if self.peek_is(TokenKind::Identifier) && self.kind(1) == Some(TokenKind::Function) {
            Some(self.advance_lexeme().unwrap_or_default())
        } else {
            None
        };

        self.expect(TokenKind::Function)?;
        let name = self.expect_get(TokenKind::Identifier)?;
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RightParen)?;

        let body = if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1; // {
            FunctionBody::Block(self.parse_block()?)
        } else if self.peek_is(TokenKind::Semicolon) {
            self.pos += 1;
            FunctionBody::Abstract
        } else {
            FunctionBody::Abstract
        };

        Ok(Statement::new(
            StatementKind::FunctionDecl { name, attributes, access_modifier, return_type, params, body },
            line,
        ))
    }

    fn parse_params(&mut self) -> Result<Vec<FunctionParam>> {
        let mut params = Vec::new();
        if self.peek_is(TokenKind::RightParen) {
            return Ok(params);
        }
        loop {
            let required = self.peek_is(TokenKind::Required);
            if required { self.pos += 1; }
            let type_name = if self.peek_is(TokenKind::Identifier) && self.kind(1) == Some(TokenKind::Identifier) {
                Some(self.advance_lexeme().unwrap_or_default())
            } else {
                None
            };
            let name = self.expect_get(TokenKind::Identifier)?;
            let default_value = if self.peek_is(TokenKind::Equal) {
                self.pos += 1; // =
                Some(self.parse_expression()?)
            } else {
                None
            };
            params.push(FunctionParam { name, type_name, required, default_value });
            if self.peek_is(TokenKind::Comma) {
                self.pos += 1;
                if self.peek_is(TokenKind::RightParen) { break; }
            } else { break; }
        }
        Ok(params)
    }

    fn parse_args(&mut self) -> Result<Vec<Argument>> {
        let mut args = Vec::new();
        if self.peek_is(TokenKind::RightParen) {
            return Ok(args);
        }
        loop {
            if self.peek_is(TokenKind::DotDotDot) {
                self.pos += 1;
                let value = self.parse_expression()?;
                args.push(Argument { name: None, value: Expression::new(
                    ExpressionKind::Spread(Box::new(value)), 0,
                ) });
            } else if self.peek_is(TokenKind::Identifier) && self.kind(1) == Some(TokenKind::Equal) {
                let name = self.advance_lexeme().unwrap_or_default();
                self.pos += 1; // =
                let value = self.parse_expression()?;
                args.push(Argument { name: Some(name), value });
            } else {
                let value = self.parse_expression()?;
                args.push(Argument { name: None, value });
            }
            if self.peek_is(TokenKind::Comma) { self.pos += 1; } else { break; }
        }
        Ok(args)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        while !self.peek_is(TokenKind::RightBrace) && self.peek_kind().is_some() {
            stmts.push(self.parse_statement()?);
        }
        self.pos += 1; // }
        Ok(stmts)
    }

    fn parse_for(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // for
        self.expect(TokenKind::LeftParen)?;

        // Determine for-in vs for-classic by looking ahead for semicolons vs 'in'
        if self.is_for_in() {
            self.parse_for_in(line)
        } else {
            self.parse_for_classic(line)
        }
    }

    fn is_for_in(&self) -> bool {
        // Look ahead to see if this is a for-in pattern
        let mut i = self.pos;
        if i < self.tokens.len() && self.tokens[i].kind == TokenKind::Var {
            i += 1;
        }
        // Expect: identifier [, identifier] in
        if i < self.tokens.len() && self.tokens[i].kind == TokenKind::Identifier {
            i += 1;
            if i < self.tokens.len() && self.tokens[i].kind == TokenKind::Comma {
                i += 1;
                // After comma must be another identifier then 'in'
                if i < self.tokens.len() && self.tokens[i].kind == TokenKind::Identifier {
                    i += 1;
                    return i < self.tokens.len() && self.tokens[i].kind == TokenKind::In;
                }
                return false;
            }
            // After first identifier, check for 'in'
            return i < self.tokens.len() && self.tokens[i].kind == TokenKind::In;
        }
        // If starts with semicolon, it's for-classic
        if i < self.tokens.len() && self.tokens[i].kind == TokenKind::Semicolon {
            return false;
        }
        false
    }

    fn parse_for_in(&mut self, line: u32) -> Result<Statement> {
        let saw_var = self.peek_is(TokenKind::Var);
        if saw_var { self.pos += 1; }

        let item = self.expect_get(TokenKind::Identifier)?;
        let index = if self.peek_is(TokenKind::Comma) {
            self.pos += 1;
            Some(self.expect_get(TokenKind::Identifier)?)
        } else {
            None
        };
        self.expect(TokenKind::In)?;
        let collection = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_for_body()?;
        Ok(Statement::new(
            StatementKind::ForLoop { item, index, collection, body },
            line,
        ))
    }

    fn parse_for_classic(&mut self, line: u32) -> Result<Statement> {
        // Parse init
        let init = if self.peek_is(TokenKind::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_for_init()?))
        };
        self.expect(TokenKind::Semicolon)?;

        // Parse condition
        let condition = if self.peek_is(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenKind::Semicolon)?;

        // Parse update
        let update = if self.peek_is(TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_for_body()?;
        Ok(Statement::new(
            StatementKind::ForClassic { init, condition, update, body },
            line,
        ))
    }

    fn parse_for_init(&mut self) -> Result<Statement> {
        if self.peek_is(TokenKind::Var) {
            let line = self.peek_line();
            self.pos += 1; // var
            let name = self.expect_get(TokenKind::Identifier)?;
            self.expect(TokenKind::Equal)?;
            let value = self.parse_expression()?;
            return Ok(Statement::new(
                StatementKind::VariableDecl { name, value },
                line,
            ));
        }
        // Assignment or expression (don't consume trailing semicolon)
        let expr = self.parse_expression()?;
        Ok(Statement::new(StatementKind::Expression(expr), self.peek_line()))
    }

    fn parse_for_body(&mut self) -> Result<Vec<Statement>> {
        if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1; // {
            self.parse_block()
        } else {
            Ok(vec![self.parse_statement()?])
        }
    }

    fn parse_while(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // while
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        let body = if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1;
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };
        Ok(Statement::new(StatementKind::WhileLoop { condition, body }, line))
    }

    fn parse_do_while(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // do
        let body = if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1;
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };
        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::DoWhile { body, condition }, line))
    }

    fn parse_if(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // if
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        let then_branch = if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1;
            self.parse_block()?
        } else {
            vec![self.parse_statement_no_if()?]
        };
        let else_branch = if self.peek_is(TokenKind::Else) {
            self.pos += 1; // else
            if self.peek_is(TokenKind::If) {
                Some(vec![self.parse_if(line)?])
            } else if self.peek_is(TokenKind::LeftBrace) {
                self.pos += 1;
                Some(self.parse_block()?)
            } else {
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };
        Ok(Statement::new(StatementKind::If { condition, then_branch, else_branch }, line))
    }

    fn parse_statement_no_if(&mut self) -> Result<Statement> {
        if self.peek_is(TokenKind::If) {
            bail!("Unexpected 'if' in single-statement context");
        }
        self.parse_statement()
    }

    fn parse_try(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // try
        self.expect(TokenKind::LeftBrace)?;
        let try_branch = self.parse_block()?;
        let mut catches = Vec::new();
        while self.peek_is(TokenKind::Catch) {
            self.pos += 1; // catch
            self.expect(TokenKind::LeftParen)?;
            let exception_var = self.expect_get(TokenKind::Identifier)?;
            self.expect(TokenKind::RightParen)?;
            self.expect(TokenKind::LeftBrace)?;
            let body = self.parse_block()?;
            catches.push(CatchBlock { exception_var, body });
        }
        let finally_branch = if self.peek_is(TokenKind::Finally) {
            self.pos += 1; // finally
            self.expect(TokenKind::LeftBrace)?;
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Statement::new(StatementKind::TryCatch { try_branch, catches, finally_branch }, line))
    }

    fn parse_return(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // return
        let expr = if self.peek_is(TokenKind::Semicolon) || self.peek_kind().is_none() || self.at_statement_boundary() {
            None
        } else {
            Some(self.parse_expression()?)
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Return(expr), line))
    }

    fn parse_throw(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // throw
        let expr = if self.peek_is(TokenKind::Semicolon) || self.peek_kind().is_none() || self.at_statement_boundary() {
            None
        } else if self.peek_is(TokenKind::LeftParen) && self.is_throw_struct() {
            self.pos += 1; // (
            let mut entries = Vec::new();
            loop {
                let key_name = if self.peek_is(TokenKind::String) {
                    let s = self.advance_lexeme().unwrap_or_default();
                    if s.len() >= 2 { s[1..s.len() - 1].to_string() } else { s }
                } else {
                    self.expect_get(TokenKind::Identifier)?
                };
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expression()?;
                let key_expr = Expression::new(
                    ExpressionKind::Literal(Literal::String(vec![StringPart::Text(key_name)])),
                    line,
                );
                entries.push((key_expr, value));
                if self.peek_is(TokenKind::Comma) { self.pos += 1; } else { break; }
            }
            self.expect(TokenKind::RightParen)?;
            Some(Expression::new(ExpressionKind::Literal(Literal::Struct(entries)), line))
        } else {
            Some(self.parse_expression()?)
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Throw(expr), line))
    }

    fn parse_continue(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // continue
        // Optional label (Phase 2 feature)
        if !self.peek_is(TokenKind::Semicolon) && !self.at_statement_boundary() {
            let _label = self.advance_lexeme();
        }
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Continue, line))
    }

    fn parse_break(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // break
        if !self.peek_is(TokenKind::Semicolon) && !self.at_statement_boundary() {
            let _label = self.advance_lexeme();
        }
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Break, line))
    }

    fn parse_rethrow(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // rethrow
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Rethrow, line))
    }

    fn parse_assert(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // assert
        let condition = self.parse_expression()?;
        let message = if self.peek_is(TokenKind::Colon) {
            self.pos += 1;
            Some(self.parse_expression()?)
        } else {
            None
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Assert { condition, message }, line))
    }

    fn parse_param(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // param
        let name = self.expect_get(TokenKind::Identifier)?;
        let default = if self.peek_is(TokenKind::Equal) {
            self.pos += 1;
            Some(self.parse_expression()?)
        } else {
            None
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Param { name, default }, line))
    }

    fn parse_include(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // include
        let expr = self.parse_expression()?;
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Include(expr), line))
    }

    fn parse_not_stmt(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // not
        let expr = self.parse_expression()?;
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }
        Ok(Statement::new(StatementKind::Not(expr), line))
    }

    fn parse_switch(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // switch
        self.expect(TokenKind::LeftParen)?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::LeftBrace)?;
        let mut cases = Vec::new();
        let mut default_case = None;
        while !self.peek_is(TokenKind::RightBrace) {
            match self.peek_kind() {
                Some(TokenKind::Case) => {
                    self.pos += 1; // case
                    let case_val = self.parse_expression()?;
                    self.expect(TokenKind::Colon)?;
                    let mut body = Vec::new();
                    while !self.peek_is(TokenKind::Case)
                        && !self.peek_is(TokenKind::Default)
                        && !self.peek_is(TokenKind::RightBrace)
                        && self.peek_kind().is_some()
                    {
                        body.push(self.parse_statement()?);
                    }
                    cases.push(SwitchCase { value: case_val, body });
                }
                Some(TokenKind::Default) => {
                    self.pos += 1; // default
                    self.expect(TokenKind::Colon)?;
                    let mut body = Vec::new();
                    while !self.peek_is(TokenKind::Case)
                        && !self.peek_is(TokenKind::Default)
                        && !self.peek_is(TokenKind::RightBrace)
                        && self.peek_kind().is_some()
                    {
                        body.push(self.parse_statement()?);
                    }
                    default_case = Some(body);
                }
                _ => bail!("Expected case or default in switch"),
            }
        }
        self.pos += 1; // }
        Ok(Statement::new(StatementKind::Switch { value, cases, default_case }, line))
    }

    fn parse_var_decl(&mut self, line: u32) -> Result<Statement> {
        self.pos += 1; // var
        let target = self.parse_assignment_target()?;
        let (op_str, value) = if self.peek_is(TokenKind::Equal) {
            self.pos += 1;
            (None, self.parse_expression()?)
        } else if matches!(self.peek_kind(),
            Some(TokenKind::PlusEqual) | Some(TokenKind::MinusEqual)
            | Some(TokenKind::StarEqual) | Some(TokenKind::SlashEqual)
            | Some(TokenKind::PercentEqual)
        ) {
            let op = self.advance_lexeme().unwrap_or_default();
            let bin_op = op[..op.len() - 1].to_string();
            (Some(bin_op), self.parse_expression()?)
        } else {
            bail!("Expected '=' or compound assignment after var target");
        };
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }

        let final_value = if let Some(op) = op_str {
            Expression::new(
                ExpressionKind::Binary {
                    left: Box::new(target_to_expression(&target, line)),
                    operator: op,
                    right: Box::new(value),
                },
                line,
            )
        } else {
            value
        };

        if let AssignmentTarget::Identifier(name) = target {
            Ok(Statement::new(StatementKind::VariableDecl { name, value: final_value }, line))
        } else {
            bail!("'var' only supports simple identifiers");
        }
    }


    fn is_destructure_assignment(&self) -> bool {
        let start = self.pos;
        let mut i = start + 1; // skip { or [
        let open = self.tokens[start].kind;
        let close = if open == TokenKind::LeftBrace { TokenKind::RightBrace } else { TokenKind::RightBracket };
        if i >= self.tokens.len() { return false; }
        if self.tokens[i].kind != TokenKind::Identifier { return false; }
        // Find the matching closing brace/bracket
        while i < self.tokens.len() && self.tokens[i].kind != close {
            i += 1;
        }
        if i >= self.tokens.len() { return false; }
        // After closing brace, must be =
        i += 1;
        i < self.tokens.len() && self.tokens[i].kind == TokenKind::Equal
    }

    fn parse_destructure_assignment(&mut self, line: u32) -> Result<Statement> {
        let is_object = self.peek_is(TokenKind::LeftBrace);
        self.pos += 1; // { or [
        let close = if is_object { TokenKind::RightBrace } else { TokenKind::RightBracket };

        let mut bindings: Vec<(String, Option<String>)> = Vec::new(); // (source_name, local_name)
        loop {
            if self.peek_is(close) { self.pos += 1; break; }
            if self.peek_is(TokenKind::DotDotDot) {
                self.pos += 1;
                let _rest = self.expect_get(TokenKind::Identifier)?;
                if self.peek_is(TokenKind::Comma) { self.pos += 1; }
                continue;
            }
            let source_name = self.expect_get(TokenKind::Identifier)?;
            let local_name = if self.peek_is(TokenKind::Colon) {
                self.pos += 1;
                Some(self.expect_get(TokenKind::Identifier)?)
            } else {
                None
            };
            bindings.push((source_name, local_name));
            if self.peek_is(TokenKind::Comma) {
                self.pos += 1;
                if self.peek_is(close) { self.pos += 1; break; }
            } else {
                self.expect(close)?;
                break;
            }
        }

        self.expect(TokenKind::Equal)?;
        let source = self.parse_expression()?;
        if self.peek_is(TokenKind::Semicolon) { self.pos += 1; }

        // Desugar: for each binding, emit: localName = source.sourceName
        Ok(Statement::new(
            StatementKind::Destructure {
                kind: if is_object { DestructureKind::Object } else { DestructureKind::Array },
                source,
                bindings,
            },
            line,
        ))
    }

    fn parse_assignment_target(&mut self) -> Result<AssignmentTarget> {
        let name = self.expect_get(TokenKind::Identifier)?;
        let mut has_accessors = false;
        while matches!(self.peek_kind(), Some(TokenKind::Dot) | Some(TokenKind::LeftBracket)) {
            has_accessors = true;
            if self.peek_is(TokenKind::Dot) {
                self.pos += 1; // .
                self.expect_get(TokenKind::Identifier)?;
            } else if self.peek_is(TokenKind::LeftBracket) {
                self.pos += 1; // [
                self.parse_expression()?; // index — consumed but not stored in var context
                self.expect(TokenKind::RightBracket)?;
            }
        }
        if !has_accessors {
            Ok(AssignmentTarget::Identifier(name))
        } else {
            bail!("'var' only supports simple identifiers");
        }
    }

    fn at_statement_boundary(&self) -> bool {
        matches!(self.peek_kind(),
            Some(TokenKind::RightBrace) | Some(TokenKind::Case) | Some(TokenKind::Default)
            | Some(TokenKind::Import) | Some(TokenKind::Class) | Some(TokenKind::Interface)
            | Some(TokenKind::Function) | Some(TokenKind::For) | Some(TokenKind::While)
            | Some(TokenKind::If) | Some(TokenKind::Try) | Some(TokenKind::Return)
            | Some(TokenKind::Throw) | Some(TokenKind::Continue) | Some(TokenKind::Break)
            | Some(TokenKind::Switch) | Some(TokenKind::Var) | None
        )
    }

    // ---- Expression parser (Pratt) ----

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment_or_conditional()
    }

    fn parse_assignment_or_conditional(&mut self) -> Result<Expression> {
        let line = self.peek_line();
        let expr = self.parse_binary(0)?;

        if self.peek_is(TokenKind::QuestionColon) {
            self.pos += 1;
            let right = self.parse_expression()?;
            return Ok(Expression::new(
                ExpressionKind::Elvis { left: Box::new(expr), right: Box::new(right) },
                line,
            ));
        }
        if self.peek_is(TokenKind::Question) {
            self.pos += 1;
            let then_expr = self.parse_expression()?;
            self.expect(TokenKind::Colon)?;
            let else_expr = self.parse_expression()?;
            return Ok(Expression::new(
                ExpressionKind::Ternary {
                    condition: Box::new(expr),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                line,
            ));
        }

        Ok(expr)
    }

    fn parse_binary(&mut self, min_prec: u8) -> Result<Expression> {
        let line = self.peek_line();
        let left = self.parse_unary()?;

        self.parse_binary_tail(left, min_prec, line)
    }

    fn parse_binary_tail(&mut self, mut left: Expression, min_prec: u8, line: u32) -> Result<Expression> {
        loop {
            if let Some((op_prec, op, consume)) = self.phrase_operator() {
                if op_prec < min_prec {
                    break;
                }
                self.pos += consume;
                let right = self.parse_binary(op_prec + 1)?;
                let next_line = self.peek_line();
                if matches!(self.peek_kind(),
                    Some(TokenKind::Equal) | Some(TokenKind::PlusEqual)
                    | Some(TokenKind::MinusEqual) | Some(TokenKind::StarEqual)
                    | Some(TokenKind::SlashEqual) | Some(TokenKind::PercentEqual)
                ) {
                    let assign_op = self.advance_lexeme().unwrap_or_default();
                    let val = self.parse_expression()?;
                    let bin = Expression::new(
                        ExpressionKind::Binary { left: Box::new(left), operator: op, right: Box::new(right) },
                        line,
                    );
                    let target = expr_to_assignment_target(&bin)?;
                    let final_val = if &assign_op == "=" { val } else {
                        let bin_op = assign_op[..assign_op.len() - 1].to_string();
                        Expression::new(
                            ExpressionKind::Binary { left: Box::new(val.clone()), operator: bin_op, right: Box::new(val) },
                            next_line,
                        )
                    };
                    return Ok(Expression::new(
                        ExpressionKind::Assignment { target, value: Box::new(final_val) },
                        next_line,
                    ));
                }

                left = Expression::new(
                    ExpressionKind::Binary { left: Box::new(left), operator: op, right: Box::new(right) },
                    line,
                );
                continue;
            }

            let op_prec = match self.peek_kind() {
                Some(TokenKind::PipePipe) | Some(TokenKind::Xor) | Some(TokenKind::Eqv) => 1,
                Some(TokenKind::AmpAmp) => 2,
                Some(TokenKind::EqualEqual) | Some(TokenKind::BangEqual)
                | Some(TokenKind::Less) | Some(TokenKind::Greater)
                | Some(TokenKind::LessEqual) | Some(TokenKind::GreaterEqual)
                | Some(TokenKind::InstanceOf) | Some(TokenKind::CastAs)
                | Some(TokenKind::Contains) => 3,
                Some(TokenKind::BitwiseOr) => 4,
                Some(TokenKind::BitwiseXor) => 5,
                Some(TokenKind::BitwiseAnd) => 6,
                Some(TokenKind::BitwiseShiftLeft) | Some(TokenKind::BitwiseShiftRight)
                | Some(TokenKind::BitwiseUnsignedShiftRight) => 7,
                Some(TokenKind::DotDot) | Some(TokenKind::DotDotLess)
                | Some(TokenKind::GreaterDotDot) | Some(TokenKind::GreaterDotDotLess) => 8,
                Some(TokenKind::Ampersand) => 9,
                Some(TokenKind::Plus) | Some(TokenKind::Minus) => 10,
                Some(TokenKind::Star) | Some(TokenKind::Slash) | Some(TokenKind::Percent) => 11,
                Some(TokenKind::Caret) => 12, // power ^
                _ => 0,
            };

            if op_prec == 0 || op_prec < min_prec {
                break;
            }

            let op_kind = self.advance().unwrap_or(TokenKind::Eof);
            let raw = self
                .tokens
                .get(self.pos - 1)
                .map(|t| &self.source[t.span.start..t.span.end])
                .unwrap_or("");
            let op = self.binary_operator_lexeme(op_kind, raw);
            let right = self.parse_binary(op_prec + 1)?;
            let next_line = self.peek_line();

            // Check for compound assignment after binary expression
            if matches!(self.peek_kind(),
                Some(TokenKind::Equal) | Some(TokenKind::PlusEqual)
                | Some(TokenKind::MinusEqual) | Some(TokenKind::StarEqual)
                | Some(TokenKind::SlashEqual) | Some(TokenKind::PercentEqual)
            ) {
                let assign_op = self.advance_lexeme().unwrap_or_default();
                let val = self.parse_expression()?;
                let bin = Expression::new(
                    ExpressionKind::Binary { left: Box::new(left), operator: op, right: Box::new(right) },
                    line,
                );
                let target = expr_to_assignment_target(&bin)?;
                let final_val = if &assign_op == "=" { val } else {
                    let bin_op = assign_op[..assign_op.len() - 1].to_string();
                    Expression::new(
                        ExpressionKind::Binary { left: Box::new(val.clone()), operator: bin_op, right: Box::new(val) },
                        next_line,
                    )
                };
                return Ok(Expression::new(
                    ExpressionKind::Assignment { target, value: Box::new(final_val) },
                    next_line,
                ));
            }

            left = Expression::new(
                ExpressionKind::Binary { left: Box::new(left), operator: op, right: Box::new(right) },
                line,
            );
        }

        // Check for simple assignment after primary
        if matches!(self.peek_kind(),
            Some(TokenKind::Equal) | Some(TokenKind::PlusEqual)
            | Some(TokenKind::MinusEqual) | Some(TokenKind::StarEqual)
            | Some(TokenKind::SlashEqual) | Some(TokenKind::PercentEqual)
        ) {
            let assign_op = self.advance_lexeme().unwrap_or_default();
            let value = self.parse_expression()?;
            let target = expr_to_assignment_target(&left)?;
            let final_val = if &assign_op == "=" { value } else {
                let bin_op = assign_op[..assign_op.len() - 1].to_string();
                Expression::new(
                    ExpressionKind::Binary { left: Box::new(left), operator: bin_op, right: Box::new(value) },
                    line,
                )
            };
            return Ok(Expression::new(
                ExpressionKind::Assignment { target, value: Box::new(final_val) },
                line,
            ));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression> {
        let line = self.peek_line();
        match self.peek_kind() {
            Some(TokenKind::Bang) => {
                self.pos += 1;
                let expr = self.parse_unary()?;
                Ok(Expression::new(ExpressionKind::UnaryNot(Box::new(expr)), line))
            }
            Some(TokenKind::Minus) => {
                self.pos += 1;
                let expr = self.parse_unary()?;
                // If the expression is a Number literal, negate it directly
                if let ExpressionKind::Literal(Literal::Number(n)) = &expr.kind {
                    return Ok(Expression::new(
                        ExpressionKind::Literal(Literal::Number(-n)),
                        line,
                    ));
                }
                Ok(Expression::new(
                    ExpressionKind::Binary {
                        left: Box::new(Expression::new(
                            ExpressionKind::Literal(Literal::Number(0.0)), line,
                        )),
                        operator: "-".to_string(),
                        right: Box::new(expr),
                    },
                    line,
                ))
            }
            Some(TokenKind::PlusPlus) | Some(TokenKind::MinusMinus) => {
                let op = self.advance_lexeme().unwrap_or_default();
                let target = self.parse_assignment_target()?;
                Ok(Expression::new(ExpressionKind::Prefix { operator: op, target }, line))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression> {
        let line = self.peek_line();
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind() {
                Some(TokenKind::LeftParen) => {
                    self.pos += 1; // (
                    let args = self.parse_args()?;
                    self.expect(TokenKind::RightParen)?;
                    expr = Expression::new(
                        ExpressionKind::FunctionCall { base: Box::new(expr), args },
                        line,
                    );
                }
                Some(TokenKind::LeftBracket) => {
                    self.pos += 1; // [
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    expr = Expression::new(
                        ExpressionKind::ArrayAccess { base: Box::new(expr), index: Box::new(index) },
                        line,
                    );
                }
                Some(TokenKind::Dot) => {
                    self.pos += 1; // .
                    let member = self.expect_get(TokenKind::Identifier)?;
                    expr = Expression::new(
                        ExpressionKind::MemberAccess { base: Box::new(expr), member },
                        line,
                    );
                }
                Some(TokenKind::QuestionDot) => {
                    self.pos += 1; // ?.
                    let member = self.expect_get(TokenKind::Identifier)?;
                    expr = Expression::new(
                        ExpressionKind::SafeMemberAccess { base: Box::new(expr), member },
                        line,
                    );
                }
                Some(TokenKind::PlusPlus) | Some(TokenKind::MinusMinus) => {
                    let operator = self.advance_lexeme().unwrap_or_default();
                    expr = Expression::new(
                        ExpressionKind::Postfix { base: Box::new(expr), operator },
                        line,
                    );
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        let line = self.peek_line();

        match self.peek_kind() {
            Some(TokenKind::Number) => {
                let lexeme = self.advance_lexeme().unwrap_or_default();
                let n = lexeme.parse::<f64>().unwrap_or(0.0);
                Ok(Expression::new(ExpressionKind::Literal(Literal::Number(n)), line))
            }
            Some(TokenKind::True) => {
                self.pos += 1;
                Ok(Expression::new(ExpressionKind::Literal(Literal::Boolean(true)), line))
            }
            Some(TokenKind::False) => {
                self.pos += 1;
                Ok(Expression::new(ExpressionKind::Literal(Literal::Boolean(false)), line))
            }
            Some(TokenKind::Null) => {
                self.pos += 1;
                Ok(Expression::new(ExpressionKind::Literal(Literal::Null), line))
            }
            Some(TokenKind::ColonColon) => {
                self.pos += 1; // ::
                let name = self.expect_get(TokenKind::Identifier)?;
                Ok(Expression::new(ExpressionKind::Identifier(name), line))
            }
            Some(TokenKind::String) => {
                let lexeme = self.advance_lexeme().unwrap_or_default();
                let parts = parse_string_content(&lexeme);
                Ok(Expression::new(ExpressionKind::Literal(Literal::String(parts)), line))
            }
            Some(TokenKind::StringStart) => {
                // Fallback: skip to StringEnd, treat as empty string
                self.pos += 1;
                while self.peek_kind() != Some(TokenKind::StringEnd) && self.peek_kind().is_some() {
                    self.pos += 1;
                }
                if self.peek_is(TokenKind::StringEnd) { self.pos += 1; }
                Ok(Expression::new(ExpressionKind::Literal(Literal::String(vec![])), line))
            }
            Some(TokenKind::New) => {
                self.pos += 1; // new
                let mut class_path = String::new();
                // Optional prefix: identifier:
                if self.kind(0) == Some(TokenKind::Identifier) && self.kind(1) == Some(TokenKind::Colon) {
                    class_path.push_str(&self.advance_lexeme().unwrap_or_default());
                    class_path.push(':');
                    self.pos += 1; // :
                }
                loop {
                    class_path.push_str(&self.expect_get(TokenKind::Identifier)?);
                    if self.peek_is(TokenKind::Dot) {
                        class_path.push('.');
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::LeftParen)?;
                let args = self.parse_args()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expression::new(ExpressionKind::New { class_path, args }, line))
            }
            Some(TokenKind::Identifier) => {
                let name = self.advance_lexeme().unwrap_or_default();
                // Check for lambda: identifier => expr
                if self.peek_is(TokenKind::EqualGreater) || self.peek_is(TokenKind::MinusGreater) {
                    let _ = self.advance_lexeme();
                    let body = self.parse_lambda_body()?;
                    return Ok(Expression::new(
                        ExpressionKind::Literal(Literal::Function {
                            params: vec![FunctionParam {
                                name, type_name: None, required: false, default_value: None,
                            }],
                            body,
                        }),
                        line,
                    ));
                }
                Ok(Expression::new(ExpressionKind::Identifier(name), line))
            }
            Some(TokenKind::LeftBrace) => {
                self.pos += 1; // {
                if self.peek_is(TokenKind::RightBrace) {
                    self.pos += 1;
                    return Ok(Expression::new(ExpressionKind::Literal(Literal::Struct(Vec::new())), line));
                }
                let members = self.parse_struct_members()?;
                self.expect(TokenKind::RightBrace)?;
                Ok(Expression::new(ExpressionKind::Literal(Literal::Struct(members)), line))
            }
            Some(TokenKind::LeftParen) => {
                self.pos += 1; // (
                if self.peek_is(TokenKind::RightParen) {
                    self.pos += 1;
                    // () => or () ->
                    if self.peek_is(TokenKind::EqualGreater) || self.peek_is(TokenKind::MinusGreater) {
                        let _ = self.advance_lexeme();
                        let body = self.parse_lambda_body()?;
                        return Ok(Expression::new(
                            ExpressionKind::Literal(Literal::Function { params: vec![], body }),
                            line,
                        ));
                    }
                    return Ok(Expression::new(ExpressionKind::Literal(Literal::Null), line));
                }
                // Check if this is a lambda: (params) => ...
                if self.is_lambda_params() {
                    let params = self.parse_params()?;
                    self.expect(TokenKind::RightParen)?;
                    let _ = self.advance_lexeme(); // => or ->
                    let body = self.parse_lambda_body()?;
                    return Ok(Expression::new(
                        ExpressionKind::Literal(Literal::Function { params, body }),
                        line,
                    ));
                }
                // Plain parenthesized expression
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                // Check for lambda: (identifier) => ...
                if self.peek_is(TokenKind::EqualGreater) || self.peek_is(TokenKind::MinusGreater) {
                    let _ = self.advance_lexeme();
                    let body = self.parse_lambda_body()?;
                    let params = match &expr.kind {
                        ExpressionKind::Identifier(name) => vec![FunctionParam {
                            name: name.clone(), type_name: None, required: false, default_value: None,
                        }],
                        _ => bail!("Expected identifier before =>"),
                    };
                    return Ok(Expression::new(
                        ExpressionKind::Literal(Literal::Function { params, body }),
                        line,
                    ));
                }
                Ok(expr)
            }
            Some(TokenKind::Function) => {
                self.pos += 1; // function
                self.expect(TokenKind::LeftParen)?;
                let params = self.parse_params()?;
                self.expect(TokenKind::RightParen)?;
                self.expect(TokenKind::LeftBrace)?;
                let body_stmts = self.parse_block()?;
                Ok(Expression::new(
                    ExpressionKind::Literal(Literal::Function {
                        params,
                        body: FunctionBody::Block(body_stmts),
                    }),
                    line,
                ))
            }
            Some(TokenKind::LeftBracket) => {
                self.pos += 1; // [
                if self.peek_is(TokenKind::RightBracket) {
                    self.pos += 1;
                    return Ok(Expression::new(ExpressionKind::Literal(Literal::Array(Vec::new())), line));
                }
                let mut items = Vec::new();
                loop {
                    if self.peek_is(TokenKind::DotDotDot) {
                        self.pos += 1;
                        let expr = self.parse_expression()?;
                        items.push(Expression::new(ExpressionKind::Spread(Box::new(expr)), line));
                    } else {
                        items.push(self.parse_expression()?);
                    }
                    if self.peek_is(TokenKind::Comma) { self.pos += 1; } else { break; }
                }
                self.expect(TokenKind::RightBracket)?;
                Ok(Expression::new(ExpressionKind::Literal(Literal::Array(items)), line))
            }
            _ => bail!("Unexpected token in expression: {:?}", self.peek_kind()),
        }
    }

    fn parse_struct_members(&mut self) -> Result<Vec<(Expression, Expression)>> {
        let mut members = Vec::new();
        loop {
            let line = self.peek_line();
            if self.peek_is(TokenKind::DotDotDot) {
                self.pos += 1;
                let spread = self.parse_expression()?;
                members.push((
                    Expression::new(ExpressionKind::Spread(Box::new(spread)), line),
                    Expression::new(ExpressionKind::Literal(Literal::Null), line),
                ));
            } else {
                let key = self.parse_expression()?;
                if !matches!(self.peek_kind(), Some(TokenKind::Colon) | Some(TokenKind::Equal)) {
                    bail!("Expected ':' or '=' in struct literal");
                }
                self.pos += 1;
                let value = self.parse_expression()?;
                members.push((key, value));
            }
            if self.peek_is(TokenKind::Comma) {
                self.pos += 1;
                if self.peek_is(TokenKind::RightBrace) { break; }
            } else {
                break;
            }
        }
        Ok(members)
    }

    fn is_lambda_params(&self) -> bool {
        // Look ahead: identifier (, identifier)* ) => or ->
        let mut i = self.pos;
        if i >= self.tokens.len() || self.tokens[i].kind != TokenKind::Identifier {
            return false;
        }
        i += 1;
        while i < self.tokens.len() && self.tokens[i].kind == TokenKind::Comma {
            i += 1;
            if i >= self.tokens.len() || self.tokens[i].kind != TokenKind::Identifier {
                return false;
            }
            i += 1;
        }
        if i >= self.tokens.len() || self.tokens[i].kind != TokenKind::RightParen {
            return false;
        }
        i += 1;
        i < self.tokens.len()
            && (self.tokens[i].kind == TokenKind::EqualGreater
                || self.tokens[i].kind == TokenKind::MinusGreater)
    }

    fn parse_lambda_body(&mut self) -> Result<FunctionBody> {
        if self.peek_is(TokenKind::LeftBrace) {
            self.pos += 1;
            Ok(FunctionBody::Block(self.parse_block()?))
        } else {
            Ok(FunctionBody::Expression(Box::new(self.parse_expression()?)))
        }
    }

    fn is_destructure_pattern(&self) -> bool {
        // Look ahead: identifier (, identifier)* } =  (no : after identifiers)
        let mut i = self.pos;
        if i >= self.tokens.len() || self.tokens[i].kind != TokenKind::Identifier {
            return false;
        }
        i += 1;
        // After the first identifier, check for : or = (struct literal) vs } or , (destructure)
        if i >= self.tokens.len() { return false; }
        if self.tokens[i].kind == TokenKind::Colon || self.tokens[i].kind == TokenKind::Equal {
            return false; // This is a struct literal { key: val }
        }
        if self.tokens[i].kind == TokenKind::Comma {
            // Struct could also have comma, so check further
            i += 1;
            if i >= self.tokens.len() { return false; }
            if self.tokens[i].kind == TokenKind::Identifier {
                i += 1;
                if i >= self.tokens.len() { return false; }
                // After second identifier, check for } or , (destructure) vs : (struct)
                return self.tokens[i].kind != TokenKind::Colon
                    && self.tokens[i].kind != TokenKind::Equal;
            }
        }
        // Single identifier — must be followed by } to be destructure
        i < self.tokens.len() && self.tokens[i].kind == TokenKind::RightBrace
    }

    fn parse_destructure_expr(&mut self, line: u32) -> Result<Expression> {
        // Parse { a, b, ...rest } as destructuring pattern
        let mut bindings: Vec<String> = Vec::new();
        loop {
            if self.peek_is(TokenKind::RightBrace) {
                self.pos += 1;
                break;
            }
            if self.peek_is(TokenKind::DotDotDot) {
                self.pos += 1; // ...
                let _rest = self.expect_get(TokenKind::Identifier)?;
                // rest binding — skip for now
                if self.peek_is(TokenKind::Comma) { self.pos += 1; }
                continue;
            }
            let name = self.expect_get(TokenKind::Identifier)?;
            // Check for rename: { sourceName: localName }
            if self.peek_is(TokenKind::Colon) {
                self.pos += 1; // :
                let _local_name = self.expect_get(TokenKind::Identifier)?;
                bindings.push(name); // use source name for member access
            } else {
                bindings.push(name);
            }
            if self.peek_is(TokenKind::Comma) {
                self.pos += 1;
                if self.peek_is(TokenKind::RightBrace) {
                    self.pos += 1;
                    break;
                }
            } else {
                self.expect(TokenKind::RightBrace)?;
                break;
            }
        }

        // This will be followed by = value in parse_binary
        // Store the binding names so parse_binary can desugar
        // For now, just return a marker expression
        Ok(Expression::new(
            ExpressionKind::Identifier(bindings.join(",")),
            line,
        ))
    }

    fn is_throw_struct(&self) -> bool {
        // Look ahead: ( identifier (=|:) ... ) is throw struct syntax
        let mut i = self.pos + 1; // skip (
        // Skip whitespace conceptually — we're looking at token kind
        if i < self.tokens.len()
            && (self.tokens[i].kind == TokenKind::Identifier || self.tokens[i].kind == TokenKind::String)
        {
            i += 1;
            if i < self.tokens.len()
                && (self.tokens[i].kind == TokenKind::Equal || self.tokens[i].kind == TokenKind::Colon)
            {
                return true;
            }
        }
        false
    }
}

// Parse string content (between quotes) into StringParts, handling #expr# interpolation
fn parse_string_content(raw: &str) -> Vec<StringPart> {
    let inner = if raw.len() >= 2 { &raw[1..raw.len() - 1] } else { return vec![]; };

    let mut parts = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    let bytes = inner.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'#' {
            i += 1;
            if i < bytes.len() && bytes[i] == b'#' {
                text.push('#');
                i += 1;
                continue;
            }
            // Interpolation start
            if !text.is_empty() {
                parts.push(StringPart::Text(std::mem::take(&mut text)));
            }
            let mut expr = String::new();
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    // Skip over nested string
                    expr.push('"');
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        expr.push(bytes[i] as char);
                        i += 1;
                    }
                    if i < bytes.len() { expr.push('"'); i += 1; }
                    continue;
                }
                if bytes[i] == b'\'' {
                    expr.push('\'');
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'\'' {
                        expr.push(bytes[i] as char);
                        i += 1;
                    }
                    if i < bytes.len() { expr.push('\''); i += 1; }
                    continue;
                }
                if bytes[i] == b'#' {
                    i += 1;
                    if i < bytes.len() && bytes[i] == b'#' {
                        expr.push_str("##");
                        i += 1;
                        continue;
                    }
                    // Closing #
                    break;
                }
                expr.push(bytes[i] as char);
                i += 1;
            }
            if !expr.is_empty() {
                let lexed = crate::tokenizer::lex(&expr);
                let mut p = Parser::new(&expr, lexed.tokens());
                if let Ok(e) = p.parse_expression() {
                    parts.push(StringPart::Expression(e));
                }
            }
        } else {
            text.push(bytes[i] as char);
            i += 1;
        }
    }

    if !text.is_empty() {
        parts.push(StringPart::Text(text));
    }

    parts
}

fn target_to_expression(target: &AssignmentTarget, line: u32) -> Expression {
    match target {
        AssignmentTarget::Identifier(name) => {
            Expression::new(ExpressionKind::Identifier(name.clone()), line)
        }
        AssignmentTarget::Member { base, member } => Expression::new(
            ExpressionKind::MemberAccess { base: base.clone(), member: member.clone() }, line,
        ),
        AssignmentTarget::Index { base, index } => Expression::new(
            ExpressionKind::ArrayAccess { base: base.clone(), index: index.clone() }, line,
        ),
    }
}

fn expr_to_assignment_target(expr: &Expression) -> Result<AssignmentTarget> {
    match &expr.kind {
        ExpressionKind::Identifier(name) => Ok(AssignmentTarget::Identifier(name.clone())),
        ExpressionKind::MemberAccess { base, member } => Ok(AssignmentTarget::Member {
            base: base.clone(), member: member.clone(),
        }),
        ExpressionKind::ArrayAccess { base, index } => Ok(AssignmentTarget::Index {
            base: base.clone(), index: index.clone(),
        }),
        _ => bail!("Invalid assignment target"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_var_decl() {
        let stmts = parse("var x = 42;", None).unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            StatementKind::VariableDecl { name, value } => {
                assert_eq!(name, "x");
                match &value.kind {
                    ExpressionKind::Literal(Literal::Number(42.0)) => {}
                    other => panic!("Expected Number(42.0), got {:?}", other),
                }
            }
            other => panic!("Expected VariableDecl, got {:?}", other),
        }
    }

    #[test]
    fn parse_basic_stmts() {
        let source = "var x = 42;\nreturn x;\nvar y = x + 1;\nreturn y;\n";
        let ast = parse(source, None).unwrap();
        assert_eq!(ast.len(), 4);
        assert!(matches!(ast[0].kind, StatementKind::VariableDecl { .. }));
        assert!(matches!(ast[1].kind, StatementKind::Return(_)));
        assert!(matches!(ast[2].kind, StatementKind::VariableDecl { .. }));
        assert!(matches!(ast[3].kind, StatementKind::Return(_)));
    }

    #[test]
    fn parse_if_else() {
        let ast = parse("if (true) { var x = 1; } else { var x = 2; }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::If { .. }));
    }

    #[test]
    fn parse_while_loop() {
        let ast = parse("var i = 0;\nwhile (i < 10) { i = i + 1; }\n", None).unwrap();
        assert_eq!(ast.len(), 2);
        assert!(matches!(ast[1].kind, StatementKind::WhileLoop { .. }));
    }

    #[test]
    fn parse_function_decl() {
        let ast = parse("function foo(x) { return x; }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::FunctionDecl { .. }));
    }

    #[test]
    fn parse_for_in() {
        let ast = parse("for (item in [1,2,3]) { }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::ForLoop { .. }));
    }

    #[test]
    fn parse_for_classic() {
        let ast = parse("for (var i = 0; i < 10; i = i + 1) { }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::ForClassic { .. }));
    }

    #[test]
    fn parse_try_catch() {
        let ast = parse("try { } catch (e) { }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::TryCatch { .. }));
    }

    #[test]
    fn parse_switch() {
        let ast = parse("switch (x) { case 1: break; default: break; }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::Switch { .. }));
    }

    #[test]
    fn parse_import() {
        let ast = parse("import foo.bar.Baz as Qux;\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::Import { .. }));
    }

    #[test]
    fn parse_class() {
        let ast = parse("class Foo { property name; function bar() { } }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::ClassDecl { .. }));
    }

    #[test]
    fn parse_interface() {
        let ast = parse("interface IFoo { function bar(); }\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::InterfaceDecl { .. }));
    }

    #[test]
    fn parse_throw() {
        let ast = parse("throw \"error\";\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::Throw(_)));
    }

    #[test]
    fn parse_struct_throw() {
        let ast = parse("throw(message=\"error\");\n", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::Throw(_)));
    }
}

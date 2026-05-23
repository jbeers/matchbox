use crate::ast::*;
use crate::tokenizer::*;
use anyhow::{bail, Result};

pub fn parse_template(source: &str, filename: Option<&str>) -> Result<Vec<Statement>> {
    let tokens = tokenize_template(source);
    let _ = filename;
    TemplateParser::new(&tokens).parse()
}

struct TemplateParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> TemplateParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind)
    }

    fn advance_lexeme(&mut self) -> Option<String> {
        let lexeme = self.tokens.get(self.pos).map(|t| t.lexeme.clone());
        self.pos += 1;
        lexeme
    }

    fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(stmt) = self.parse_template_statement()? {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    fn parse_template_statement(&mut self) -> Result<Option<Statement>> {
        match self.peek_kind() {
            Some(TokenKind::ContentText) => {
                let lexeme = self.advance_lexeme().unwrap_or_default();
                // Merge adjacent ContentText
                let mut text = lexeme;
                while self.peek_kind() == Some(TokenKind::ContentText) {
                    text.push_str(&self.advance_lexeme().unwrap_or_default());
                }
                Ok(Some(Statement::new(
                    StatementKind::BufferOutput(Expression::new(
                        ExpressionKind::Literal(Literal::String(vec![StringPart::Text(text)])),
                        0,
                    )),
                    0,
                )))
            }
            Some(TokenKind::ComponentName) => {
                let name = self.advance_lexeme().unwrap_or_default();
                self.parse_component(&name)
            }
            Some(TokenKind::Less) => {
                self.pos += 1;
                Ok(None) // Skip stray <
            }
            Some(TokenKind::Identifier) | Some(TokenKind::Number) | Some(TokenKind::String)
            | Some(TokenKind::Plus) | Some(TokenKind::Minus) | Some(TokenKind::Bang) => {
                // These are expression tokens from #expr# interpolation — skip for now
                // The parser will handle them when we build the full template expression parser
                self.pos += 1;
                Ok(None)
            }
            Some(TokenKind::ComponentClose) | Some(TokenKind::ComponentSelfClose) => {
                self.pos += 1;
                Ok(None)
            }
            None => Ok(None),
            _ => {
                self.pos += 1;
                Ok(None)
            }
        }
    }

    fn parse_component(&mut self, name: &str) -> Result<Option<Statement>> {
        match name {
            "set" => self.parse_set_tag(),
            "if" => self.parse_if_tag(),
            "output" => {
                // <bx:output> body </bx:output>
                // Consume attributes (>)
                self.skip_to_close();
                let body = self.parse_until_end_component("output")?;
                Ok(Some(Statement::new(
                    StatementKind::BufferOutput(Expression::new(
                        ExpressionKind::Literal(Literal::Null),
                        0,
                    )),
                    0,
                )))
            }
            _ => {
                // Unknown component — skip until closing tag or self-close
                self.skip_to_close();
                self.parse_until_end_component(name)?;
                Ok(None)
            }
        }
    }

    fn parse_set_tag(&mut self) -> Result<Option<Statement>> {
        // <bx:set expr> or <bx:set expr />
        // Collect remaining tokens until > or />
        let mut expr_text = String::new();
        loop {
            match self.peek_kind() {
                Some(TokenKind::ComponentClose) | Some(TokenKind::ComponentSelfClose) => {
                    self.pos += 1;
                    break;
                }
                Some(TokenKind::Identifier) | Some(TokenKind::Number)
                | Some(TokenKind::String) | Some(TokenKind::Equal)
                | Some(TokenKind::Plus) | Some(TokenKind::Minus) | Some(TokenKind::Star)
                | Some(TokenKind::Slash) | Some(TokenKind::Dot) => {
                    expr_text.push(' ');
                    expr_text.push_str(&self.advance_lexeme().unwrap_or_default());
                }
                _ => { self.pos += 1; }
            }
        }
        if expr_text.is_empty() { return Ok(None); }
        // Parse expression text as BoxLang
        if let Ok(stmts) = crate::parser::parse(&expr_text, None) {
            if let Some(first) = stmts.into_iter().next() {
                return Ok(Some(first));
            }
        }
        Ok(None)
    }

    fn parse_if_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(None) // Full if/elseif/else requires proper body parsing
    }

    fn skip_to_close(&mut self) {
        while self.pos < self.tokens.len() {
            match self.peek_kind() {
                Some(TokenKind::ComponentClose) | Some(TokenKind::ComponentSelfClose) => {
                    self.pos += 1;
                    return;
                }
                _ => { self.pos += 1; }
            }
        }
    }

    fn parse_until_end_component(&mut self, _name: &str) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() {
            if self.peek_kind() == Some(TokenKind::ComponentName) {
                // Check if it's a closing tag for our component
                let saved = self.pos;
                self.pos += 1;
                let close_name = self.tokens.get(self.pos).map(|t| t.lexeme.as_str());
                // self.pos restored below — for now, just look for </...>
                self.pos = saved;
                // Skip nested component entirely
                self.pos += 1;
                self.skip_to_close();
                continue;
            }
            if let Some(stmt) = self.parse_template_statement()? {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }
}

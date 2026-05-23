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

    fn peek_lexeme(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|t| t.lexeme.as_str())
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
                let mut text = lexeme;
                while self.peek_kind() == Some(TokenKind::ContentText) {
                    text.push_str(&self.advance_lexeme().unwrap_or_default());
                }
                let trimmed = text.trim().to_string();
                if trimmed.is_empty() {
                    return Ok(None);
                }
                Ok(Some(Statement::new(
                    StatementKind::BufferOutput(Expression::new(
                        ExpressionKind::Literal(Literal::String(vec![StringPart::Text(trimmed)])),
                        0,
                    )),
                    0,
                )))
            }
            Some(TokenKind::ComponentName) => {
                let name = self.advance_lexeme().unwrap_or_default();
                self.parse_component(&name)
            }
            Some(TokenKind::Less) | Some(TokenKind::ComponentClose)
            | Some(TokenKind::ComponentSelfClose) => {
                self.pos += 1;
                Ok(None)
            }
            None => Ok(None),
            _ => {
                // Expression tokens from #expr# interpolation — skip for now
                self.pos += 1;
                Ok(None)
            }
        }
    }

    fn parse_component(&mut self, name: &str) -> Result<Option<Statement>> {
        match name {
            "set" => self.parse_set_tag(),
            "if" => self.parse_if_tag(),
            "loop" => self.parse_loop_tag(),
            "return" => self.parse_return_tag(),
            "break" => self.parse_break_continue(true),
            "continue" => self.parse_break_continue(false),
            "try" => self.parse_try_tag(),
            "switch" => self.parse_switch_tag(),
            "include" => self.parse_include_tag(),
            "import" => self.parse_import_tag(),
            "throw" => self.parse_throw_tag(),
            "rethrow" => self.parse_simple_tag(StatementKind::Rethrow),
            "function" => self.parse_function_tag(),
            "output" => {
                self.skip_to_close();
                Ok(None)
            }
            "script" => {
                self.skip_to_close();
                // Collect script tokens until </bx:script>
                // The lexer switches to script mode, so tokens are script tokens
                // Parse them using the script parser
                Ok(None)
            }
            _ => {
                self.skip_to_close();
                self.skip_body(name);
                Ok(None)
            }
        }
    }

    fn parse_set_tag(&mut self) -> Result<Option<Statement>> {
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
                | Some(TokenKind::Slash) | Some(TokenKind::Dot) | Some(TokenKind::Ampersand)
                | Some(TokenKind::Less) | Some(TokenKind::Greater)
                | Some(TokenKind::LeftParen) | Some(TokenKind::RightParen) => {
                    let lex = self.advance_lexeme().unwrap_or_default();
                    if !expr_text.is_empty() { expr_text.push(' '); }
                    expr_text.push_str(&lex);
                }
                _ => { self.pos += 1; }
            }
        }
        if expr_text.trim().is_empty() { return Ok(None); }
        if let Ok(stmts) = crate::parser::parse(&expr_text, None) {
            if let Some(first) = stmts.into_iter().next() {
                return Ok(Some(first));
            }
        }
        Ok(None)
    }

    fn parse_if_tag(&mut self) -> Result<Option<Statement>> {
        // Collect condition expression
        let mut cond_text = String::new();
        loop {
            match self.peek_kind() {
                Some(TokenKind::ComponentClose) => { self.pos += 1; break; }
                Some(TokenKind::ComponentSelfClose) => { self.pos += 1; return Ok(None); }
                Some(TokenKind::Identifier) | Some(TokenKind::Number) | Some(TokenKind::String)
                | Some(TokenKind::Equal) | Some(TokenKind::Plus) | Some(TokenKind::Minus)
                | Some(TokenKind::Star) | Some(TokenKind::Slash) | Some(TokenKind::Dot)
                | Some(TokenKind::Less) | Some(TokenKind::Greater) | Some(TokenKind::Ampersand)
                | Some(TokenKind::EqualEqual) | Some(TokenKind::BangEqual)
                | Some(TokenKind::LessEqual) | Some(TokenKind::GreaterEqual)
                | Some(TokenKind::AmpAmp) | Some(TokenKind::PipePipe)
                | Some(TokenKind::LeftParen) | Some(TokenKind::RightParen)
                | Some(TokenKind::Bang) => {
                    let lex = self.advance_lexeme().unwrap_or_default();
                    if !cond_text.is_empty() { cond_text.push(' '); }
                    cond_text.push_str(&lex);
                }
                _ => { self.pos += 1; }
            }
        }

        // Parse condition
        let condition = if !cond_text.trim().is_empty() {
            if let Ok(stmts) = crate::parser::parse(&cond_text, None) {
                stmts.into_iter().next().and_then(|s| {
                    if let StatementKind::Expression(expr) = s.kind { Some(expr) } else { None }
                })
            } else { None }
        } else { None };

        // Parse body until </bx:if>, <bx:elseif>, or <bx:else>
        let then_branch = self.parse_template_body(&["elseif", "else", "if"])?;
        let mut else_branch = None;

        // Check for elseif or else
        if self.peek_kind() == Some(TokenKind::ComponentName) {
            let next_name = self.peek_lexeme().unwrap_or("");
            if next_name == "elseif" {
                self.pos += 1;
                let elseif_stmt = self.parse_if_tag()?;
                else_branch = elseif_stmt.map(|s| vec![s]);
            } else if next_name == "else" {
                self.pos += 1;
                self.skip_to_close();
                let else_body = self.parse_template_body(&["if"])?;
                else_branch = Some(else_body);
            }
        }

        // Skip closing </bx:if> if present
        if self.peek_kind() == Some(TokenKind::ComponentName) {
            let name = self.peek_lexeme().unwrap_or("");
            if name == "if" {
                self.pos += 1;
                self.skip_to_close();
            }
        }

        let condition = condition.unwrap_or_else(|| Expression::new(
            ExpressionKind::Literal(Literal::Boolean(true)), 0,
        ));

        Ok(Some(Statement::new(
            StatementKind::If { condition, then_branch, else_branch },
            0,
        )))
    }

    fn parse_template_body(&mut self, end_tags: &[&str]) -> Result<Vec<Statement>> {
        let mut stmts = Vec::new();
        loop {
            // Check if we've hit an end tag
            if self.peek_kind() == Some(TokenKind::ComponentName) {
                let name = self.peek_lexeme().unwrap_or("");
                if end_tags.contains(&name) {
                    break;
                }
                // Skip unknown components
                self.pos += 1;
                self.skip_to_close();
                self.skip_body("unknown");
                continue;
            }
            if self.peek_kind().is_none() { break; }
            if let Some(stmt) = self.parse_template_statement()? {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    fn parse_loop_tag(&mut self) -> Result<Option<Statement>> {
        // <bx:loop array="#arr#" item="val"> body </bx:loop>
        // <bx:loop condition="expr"> body </bx:loop>
        self.skip_to_close();
        let body = self.parse_template_body(&["loop"])?;
        // Skip closing </bx:loop>
        self.skip_closing("loop");
        // For now, just wrap in a stub
        Ok(Some(Statement::new(StatementKind::WhileLoop {
            condition: Expression::new(ExpressionKind::Literal(Literal::Boolean(true)), 0),
            body,
        }, 0)))
    }

    fn parse_return_tag(&mut self) -> Result<Option<Statement>> {
        let mut expr_text = String::new();
        loop {
            match self.peek_kind() {
                Some(TokenKind::ComponentClose) | Some(TokenKind::ComponentSelfClose) => {
                    self.pos += 1; break;
                }
                _ => {
                    if !expr_text.is_empty() { expr_text.push(' '); }
                    expr_text.push_str(&self.advance_lexeme().unwrap_or_default());
                }
            }
        }
        let expr = if expr_text.trim().is_empty() {
            None
        } else if let Ok(stmts) = crate::parser::parse(&expr_text, None) {
            stmts.into_iter().next().and_then(|s| if let StatementKind::Expression(e) = s.kind { Some(e) } else { None })
        } else { None };
        Ok(Some(Statement::new(StatementKind::Return(expr), 0)))
    }

    fn parse_break_continue(&mut self, is_break: bool) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(Some(Statement::new(
            if is_break { StatementKind::Break } else { StatementKind::Continue }, 0,
        )))
    }

    fn parse_try_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        let try_branch = self.parse_template_body(&["catch", "finally", "try"])?;
        let mut catches = Vec::new();
        let mut finally_branch = None;
        // Handle catch blocks
        loop {
            if self.peek_kind() == Some(TokenKind::ComponentName) {
                let name = self.peek_lexeme().unwrap_or("");
                if name == "catch" {
                    self.pos += 1;
                    self.skip_to_close();
                    let body = self.parse_template_body(&["catch", "finally", "try"])?;
                    catches.push(CatchBlock { exception_var: "e".to_string(), body });
                    continue;
                }
                if name == "finally" {
                    self.pos += 1;
                    self.skip_to_close();
                    finally_branch = Some(self.parse_template_body(&["try"])?);
                    break;
                }
            }
            break;
        }
        self.skip_closing("try");
        Ok(Some(Statement::new(StatementKind::TryCatch { try_branch, catches, finally_branch }, 0)))
    }

    fn parse_switch_tag(&mut self) -> Result<Option<Statement>> {
        // <bx:switch expression="#expr#"> <bx:case value="1">...</bx:case> <bx:defaultcase>...</bx:defaultcase> </bx:switch>
        self.skip_to_close();
        let mut cases = Vec::new();
        let mut default_case = None;
        loop {
            if self.peek_kind() == Some(TokenKind::ComponentName) {
                let name = self.peek_lexeme().unwrap_or("");
                if name == "case" {
                    self.pos += 1;
                    // Read value attribute
                    let mut val_text = String::new();
                    loop {
                        match self.peek_kind() {
                            Some(TokenKind::ComponentClose) => { self.pos += 1; break; }
                            _ => {
                                if !val_text.is_empty() { val_text.push(' '); }
                                val_text.push_str(&self.advance_lexeme().unwrap_or_default());
                            }
                        }
                    }
                    let body = self.parse_template_body(&["case", "defaultcase", "switch"])?;
                    if let Ok(stmts) = crate::parser::parse(&val_text, None) {
                        if let Some(s) = stmts.into_iter().next() {
                            if let StatementKind::Expression(val) = s.kind {
                                cases.push(SwitchCase { value: val, body });
                            }
                        }
                    }
                    continue;
                }
                if name == "defaultcase" {
                    self.pos += 1;
                    self.skip_to_close();
                    default_case = Some(self.parse_template_body(&["switch"])?);
                    break;
                }
            }
            break;
        }
        self.skip_closing("switch");
        Ok(Some(Statement::new(StatementKind::Switch {
            value: Expression::new(ExpressionKind::Literal(Literal::Null), 0),
            cases, default_case,
        }, 0)))
    }

    fn parse_include_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(Some(Statement::new(
            StatementKind::Include(Expression::new(
                ExpressionKind::Literal(Literal::String(vec![StringPart::Text(String::new())])), 0,
            )), 0,
        )))
    }

    fn parse_import_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(Some(Statement::new(StatementKind::Import { path: String::new(), alias: None }, 0)))
    }

    fn parse_throw_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(Some(Statement::new(StatementKind::Throw(None), 0)))
    }

    fn parse_simple_tag(&mut self, kind: StatementKind) -> Result<Option<Statement>> {
        self.skip_to_close();
        Ok(Some(Statement::new(kind, 0)))
    }

    fn parse_function_tag(&mut self) -> Result<Option<Statement>> {
        self.skip_to_close();
        let body = self.parse_template_body(&["function"])?;
        self.skip_closing("function");
        Ok(Some(Statement::new(StatementKind::FunctionDecl {
            name: String::new(),
            attributes: vec![],
            access_modifier: None,
            return_type: None,
            params: vec![],
            body: FunctionBody::Block(body),
        }, 0)))
    }

    fn skip_closing(&mut self, name: &str) {
        if self.peek_kind() == Some(TokenKind::ComponentName)
            && self.peek_lexeme() == Some(name)
        {
            self.pos += 1;
            self.skip_to_close();
        }
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

    fn skip_body(&mut self, _name: &str) {
        while self.pos < self.tokens.len() {
            if self.peek_kind() == Some(TokenKind::ComponentName) {
                self.pos += 1;
                self.skip_to_close();
                continue;
            }
            self.pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_literal_text() {
        let ast = parse_template("hello", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::BufferOutput(_)));
    }

    #[test]
    fn parse_set_tag() {
        let ast = parse_template("<bx:set x = 10>", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::Expression(_)));
    }

    #[test]
    fn parse_if_tag() {
        let ast = parse_template("<bx:if x GT 5>big<bx:else>small</bx:if>", None).unwrap();
        assert_eq!(ast.len(), 1);
        assert!(matches!(ast[0].kind, StatementKind::If { .. }));
    }

    #[test]
    fn parse_if_elseif_else() {
        let ast = parse_template("<bx:if x GT 10>big<bx:elseif x GT 5>med<bx:else>small</bx:if>", None).unwrap();
        assert_eq!(ast.len(), 1);
    }
}

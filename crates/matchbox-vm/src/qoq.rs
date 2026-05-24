use crate::datasource::traits::{QueryColumn, QueryColumnType, QueryResult, SqlValue};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    BlockComment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Identifiers and literals
    Identifier,
    Number,
    String,
    OdbcDateTimeLiteral,
    BindParameter,

    // Keywords
    Select,
    From,
    Where,
    Group,
    By,
    Having,
    Order,
    Asc,
    Desc,
    Limit,
    Distinct,
    Union,
    All,
    As,
    And,
    Or,
    Not,
    In,
    Between,
    Like,
    Is,
    Null,
    Case,
    When,
    Then,
    Else,
    End,
    Cast,
    Convert,
    On,
    Join,
    Inner,
    Left,
    Right,
    Full,
    Outer,
    Cross,
    True,
    False,
    Contains,
    InstanceOf,
    CastAs,
    Xor,
    Eqv,

    // Punctuation and operators
    Comma,
    Dot,
    LeftParen,
    RightParen,
    Asterisk,
    Plus,
    Minus,
    Slash,
    Percent,
    Equal,
    Less,
    Greater,
    Bang,
    Colon,
    Semicolon,
    Question,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    BangEqual,
    NotEqualAngle,
    PipePipe,
    AmpAmp,
    Ampersand,
    QuestionDot,
    QuestionColon,
    ColonColon,
    DotDot,
    DotDotLess,
    GreaterDotDot,
    GreaterDotDotLess,
    Caret,

    Eof,
}

#[derive(Debug, Clone)]
pub struct LexedSource<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    trivia: Vec<Trivia>,
}

impl<'a> LexedSource<'a> {
    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn trivia(&self) -> &[Trivia] {
        &self.trivia
    }

    pub fn into_parts(self) -> (&'a str, Vec<Token>, Vec<Trivia>) {
        (self.source, self.tokens, self.trivia)
    }

    pub fn text(&self, span: Span) -> &'a str {
        &self.source[span.start..span.end]
    }
}

pub fn lex(source: &str) -> LexedSource<'_> {
    Lexer::new(source).lex()
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: u32,
    col: u32,
    tokens: Vec<Token>,
    trivia: Vec<Trivia>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            trivia: Vec::new(),
        }
    }

    fn lex(mut self) -> LexedSource<'a> {
        while self.pos < self.source.len() {
            self.skip_trivia();
            if self.pos >= self.source.len() {
                break;
            }
            let start = self.pos;
            let line = self.line;
            let col = self.col;
            let kind = self.next_token_kind();
            self.tokens.push(Token {
                kind,
                span: Span {
                    start,
                    end: self.pos,
                    line,
                    col,
                },
            });
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.pos,
                end: self.pos,
                line: self.line,
                col: self.col,
            },
        });
        LexedSource {
            source: self.source,
            tokens: self.tokens,
            trivia: self.trivia,
        }
    }

    fn skip_trivia(&mut self) {
        loop {
            let start = self.pos;
            let line = self.line;
            let col = self.col;
            let Some(ch) = self.current_char() else {
                return;
            };

            if ch.is_whitespace() {
                self.consume_while(|c| c.is_whitespace());
                self.trivia.push(Trivia {
                    kind: TriviaKind::Whitespace,
                    span: Span {
                        start,
                        end: self.pos,
                        line,
                        col,
                    },
                });
                continue;
            }

            if self.source[self.pos..].starts_with("--") {
                self.advance();
                self.advance();
                while let Some(c) = self.current_char() {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    self.advance();
                }
                self.trivia.push(Trivia {
                    kind: TriviaKind::LineComment,
                    span: Span {
                        start,
                        end: self.pos,
                        line,
                        col,
                    },
                });
                continue;
            }

            if self.source[self.pos..].starts_with("/*") {
                self.advance();
                self.advance();
                while self.pos < self.source.len() && !self.source[self.pos..].starts_with("*/") {
                    self.advance();
                }
                if self.source[self.pos..].starts_with("*/") {
                    self.advance();
                    self.advance();
                }
                self.trivia.push(Trivia {
                    kind: TriviaKind::BlockComment,
                    span: Span {
                        start,
                        end: self.pos,
                        line,
                        col,
                    },
                });
                continue;
            }

            break;
        }
    }

    fn next_token_kind(&mut self) -> TokenKind {
        if let Some(kind) = self.lex_odbc_datetime_literal() {
            return kind;
        }

        let ch = self.current_char().unwrap_or('\0');
        if ch == '?' {
            self.advance();
            return TokenKind::BindParameter;
        }
        if ch == ':' {
            if self.peek_ident_start(1) {
                self.advance();
                self.consume_identifier();
                return TokenKind::BindParameter;
            }
        }
        if ch == '\'' {
            self.consume_string();
            return TokenKind::String;
        }
        if ch.is_ascii_digit() || (ch == '.' && self.peek_char(1).is_some_and(|c| c.is_ascii_digit())) {
            self.consume_number();
            return TokenKind::Number;
        }
        if self.is_ident_start(ch) || ch == '"' || ch == '`' || ch == '[' {
            let ident = self.consume_identifier_text();
            return keyword_or_ident(ident);
        }

        self.consume_operator_or_punct()
    }

    fn lex_odbc_datetime_literal(&mut self) -> Option<TokenKind> {
        let rest = &self.source[self.pos..];
        let lower = rest.to_ascii_lowercase();
        let kind = if lower.starts_with("{ts '") {
            Some("{ts '")
        } else if lower.starts_with("{d '") {
            Some("{d '")
        } else if lower.starts_with("{t '") {
            Some("{t '")
        } else {
            None
        }?;

        self.advance_n(kind.len());
        while self.pos < self.source.len() {
            if self.current_char() == Some('\'') && self.peek_char(1) == Some('}') {
                self.advance();
                self.advance();
                return Some(TokenKind::OdbcDateTimeLiteral);
            }
            self.advance();
        }
        Some(TokenKind::OdbcDateTimeLiteral)
    }

    fn consume_operator_or_punct(&mut self) -> TokenKind {
        let ch = self.current_char().unwrap_or('\0');
        match ch {
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            '.' => {
                if self.peek_char(1) == Some('.') {
                    self.advance();
                    self.advance();
                    if self.current_char() == Some('<') {
                        self.advance();
                        TokenKind::DotDotLess
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    self.advance();
                    TokenKind::Dot
                }
            }
            '(' => {
                self.advance();
                TokenKind::LeftParen
            }
            ')' => {
                self.advance();
                TokenKind::RightParen
            }
            '*' => {
                self.advance();
                TokenKind::Asterisk
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                TokenKind::Minus
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '%' => {
                self.advance();
                TokenKind::Percent
            }
            '^' => {
                self.advance();
                TokenKind::Caret
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            ':' => {
                if self.peek_char(1) == Some(':') {
                    self.advance();
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    self.advance();
                    TokenKind::Colon
                }
            }
            '?' => {
                if self.peek_char(1) == Some('.') {
                    self.advance();
                    self.advance();
                    TokenKind::QuestionDot
                } else if self.peek_char(1) == Some(':') {
                    self.advance();
                    self.advance();
                    TokenKind::QuestionColon
                } else {
                    self.advance();
                    TokenKind::Question
                }
            }
            '=' => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    TokenKind::EqualEqual
                } else if self.peek_char(1) == Some('>') {
                    self.advance();
                    self.advance();
                    TokenKind::Equal
                } else {
                    self.advance();
                    TokenKind::Equal
                }
            }
            '<' => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    TokenKind::LessEqual
                } else if self.peek_char(1) == Some('>') {
                    self.advance();
                    self.advance();
                    TokenKind::NotEqualAngle
                } else if self.peek_char(1) == Some('.') && self.peek_char(2) == Some('.') {
                    self.advance();
                    self.advance();
                    self.advance();
                    TokenKind::DotDotLess
                } else {
                    self.advance();
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    TokenKind::GreaterEqual
                } else if self.peek_char(1) == Some('.') && self.peek_char(2) == Some('.') {
                    self.advance();
                    self.advance();
                    self.advance();
                    if self.current_char() == Some('<') {
                        self.advance();
                        TokenKind::GreaterDotDotLess
                    } else {
                        TokenKind::GreaterDotDot
                    }
                } else {
                    self.advance();
                    TokenKind::Greater
                }
            }
            '!' => {
                if self.peek_char(1) == Some('=') {
                    self.advance();
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    self.advance();
                    TokenKind::Bang
                }
            }
            '|' => {
                if self.peek_char(1) == Some('|') {
                    self.advance();
                    self.advance();
                    TokenKind::PipePipe
                } else {
                    self.advance();
                    TokenKind::PipePipe
                }
            }
            '&' => {
                if self.peek_char(1) == Some('&') {
                    self.advance();
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    self.advance();
                    TokenKind::Ampersand
                }
            }
            _ => {
                self.advance();
                TokenKind::Identifier
            }
        }
    }

    fn consume_identifier_text(&mut self) -> &'a str {
        let start = self.pos;
        if matches!(self.current_char(), Some('"') | Some('`') | Some('[')) {
            let quote = self.current_char().unwrap();
            self.advance();
            while let Some(ch) = self.current_char() {
                match quote {
                    '"' if ch == '"' && self.peek_char(1) != Some('"') => {
                        self.advance();
                        break;
                    }
                    '`' if ch == '`' && self.peek_char(1) != Some('`') => {
                        self.advance();
                        break;
                    }
                    '[' if ch == ']' => {
                        self.advance();
                        break;
                    }
                    _ => self.advance(),
                }
            }
            return &self.source[start..self.pos];
        }

        self.consume_identifier();
        &self.source[start..self.pos]
    }

    fn consume_identifier(&mut self) {
        self.advance();
        while let Some(ch) = self.current_char() {
            if self.is_ident_continue(ch) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn consume_number(&mut self) {
        if self.current_char() == Some('.') {
            self.advance();
        }
        if self.current_char() == Some('0') && matches!(self.peek_char(1), Some('x' | 'X')) {
            self.advance();
            self.advance();
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_hexdigit() {
                    self.advance();
                } else {
                    break;
                }
            }
            return;
        }
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.current_char() == Some('.') && self.peek_char(1).is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        if matches!(self.current_char(), Some('e' | 'E')) {
            let save = self.pos;
            self.advance();
            if matches!(self.current_char(), Some('+' | '-')) {
                self.advance();
            }
            let mut saw_digit = false;
            while let Some(ch) = self.current_char() {
                if ch.is_ascii_digit() {
                    saw_digit = true;
                    self.advance();
                } else {
                    break;
                }
            }
            if !saw_digit {
                self.pos = save;
            }
        }
    }

    fn consume_string(&mut self) {
        self.advance();
        while let Some(ch) = self.current_char() {
            self.advance();
            if ch == '\'' {
                if self.current_char() == Some('\'') {
                    self.advance();
                    continue;
                }
                break;
            }
        }
    }

    fn consume_while<F>(&mut self, mut predicate: F)
    where
        F: FnMut(char) -> bool,
    {
        while let Some(ch) = self.current_char() {
            if predicate(ch) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }

    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    fn current_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.source[self.pos..].chars().nth(offset)
    }

    fn peek_ident_start(&self, offset: usize) -> bool {
        self.peek_char(offset).is_some_and(|ch| self.is_ident_start(ch))
    }

    fn is_ident_start(&self, ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
    }

    fn is_ident_continue(&self, ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }
}

fn keyword_or_ident(lexeme: &str) -> TokenKind {
    match lexeme.to_ascii_lowercase().as_str() {
        "select" => TokenKind::Select,
        "from" => TokenKind::From,
        "where" => TokenKind::Where,
        "group" => TokenKind::Group,
        "by" => TokenKind::By,
        "having" => TokenKind::Having,
        "order" => TokenKind::Order,
        "asc" => TokenKind::Asc,
        "desc" => TokenKind::Desc,
        "limit" => TokenKind::Limit,
        "distinct" => TokenKind::Distinct,
        "union" => TokenKind::Union,
        "all" => TokenKind::All,
        "as" => TokenKind::As,
        "and" => TokenKind::And,
        "or" => TokenKind::Or,
        "not" => TokenKind::Not,
        "in" => TokenKind::In,
        "between" => TokenKind::Between,
        "like" => TokenKind::Like,
        "is" => TokenKind::Is,
        "null" => TokenKind::Null,
        "case" => TokenKind::Case,
        "when" => TokenKind::When,
        "then" => TokenKind::Then,
        "else" => TokenKind::Else,
        "end" => TokenKind::End,
        "cast" => TokenKind::Cast,
        "convert" => TokenKind::Convert,
        "on" => TokenKind::On,
        "join" => TokenKind::Join,
        "inner" => TokenKind::Inner,
        "left" => TokenKind::Left,
        "right" => TokenKind::Right,
        "full" => TokenKind::Full,
        "outer" => TokenKind::Outer,
        "cross" => TokenKind::Cross,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "contains" => TokenKind::Contains,
        "instanceof" => TokenKind::InstanceOf,
        "castas" => TokenKind::CastAs,
        "xor" => TokenKind::Xor,
        "eqv" => TokenKind::Eqv,
        _ => TokenKind::Identifier,
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at line {}, col {}",
            self.message, self.span.line, self.span.col
        )
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub kind: QueryKind,
    pub span: Span,
}

impl Query {
    pub fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::Query(self));
        match &self.kind {
            QueryKind::Select(select) => select.walk(f),
            QueryKind::Union { left, right, .. } => {
                left.walk(f);
                right.walk(f);
            }
        }
    }

    pub fn expressions<'a>(&'a self) -> Vec<&'a Expression> {
        let mut out = Vec::new();
        self.collect_expressions(&mut out);
        out
    }

    pub fn tables<'a>(&'a self) -> Vec<&'a TableRef> {
        let mut out = Vec::new();
        self.collect_tables(&mut out);
        out
    }

    fn collect_expressions<'a>(&'a self, out: &mut Vec<&'a Expression>) {
        match &self.kind {
            QueryKind::Select(select) => select.collect_expressions(out),
            QueryKind::Union { left, right, .. } => {
                left.collect_expressions(out);
                right.collect_expressions(out);
            }
        }
    }

    fn collect_tables<'a>(&'a self, out: &mut Vec<&'a TableRef>) {
        match &self.kind {
            QueryKind::Select(select) => select.collect_tables(out),
            QueryKind::Union { left, right, .. } => {
                left.collect_tables(out);
                right.collect_tables(out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryKind {
    Select(SelectStatement),
    Union {
        left: Box<Query>,
        all: bool,
        right: Box<Query>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    pub distinct: bool,
    pub projection: Vec<SelectItem>,
    pub from: Vec<TableRef>,
    pub where_clause: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub having: Option<Expression>,
    pub order_by: Vec<OrderByItem>,
    pub limit: Option<u64>,
}

impl SelectStatement {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::SelectStatement(self));
        for item in &self.projection {
            item.walk(f);
        }
        for table in &self.from {
            table.walk(f);
        }
        if let Some(expr) = &self.where_clause {
            expr.walk(f);
        }
        for expr in &self.group_by {
            expr.walk(f);
        }
        if let Some(expr) = &self.having {
            expr.walk(f);
        }
        for item in &self.order_by {
            item.walk(f);
        }
    }

    fn collect_expressions<'a>(&'a self, out: &mut Vec<&'a Expression>) {
        for item in &self.projection {
            out.push(&item.expr);
        }
        for table in &self.from {
            table.collect_expressions(out);
        }
        if let Some(expr) = &self.where_clause {
            out.push(expr);
        }
        for expr in &self.group_by {
            out.push(expr);
        }
        if let Some(expr) = &self.having {
            out.push(expr);
        }
        for item in &self.order_by {
            out.push(&item.expr);
        }
    }

    fn collect_tables<'a>(&'a self, out: &mut Vec<&'a TableRef>) {
        for table in &self.from {
            table.collect_tables(out);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectItem {
    pub expr: Expression,
    pub alias: Option<String>,
}

impl SelectItem {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::SelectItem(self));
        self.expr.walk(f);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRef {
    pub source: TableSource,
    pub alias: Option<String>,
    pub joins: Vec<JoinClause>,
}

impl TableRef {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::TableRef(self));
        if let TableSource::Subquery(query) = &self.source {
            query.walk(f);
        }
        for join in &self.joins {
            join.walk(f);
        }
    }

    fn collect_expressions<'a>(&'a self, out: &mut Vec<&'a Expression>) {
        if let TableSource::Subquery(query) = &self.source {
            query.collect_expressions(out);
        }
        for join in &self.joins {
            join.collect_expressions(out);
        }
    }

    fn collect_tables<'a>(&'a self, out: &mut Vec<&'a TableRef>) {
        out.push(self);
        if let TableSource::Subquery(query) = &self.source {
            query.collect_tables(out);
        }
        for join in &self.joins {
            join.collect_tables(out);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableSource {
    Named(Vec<String>),
    Subquery(Box<Query>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub kind: JoinKind,
    pub table: TableRef,
    pub on: Option<Expression>,
}

impl JoinClause {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::JoinClause(self));
        self.table.walk(f);
        if let Some(expr) = &self.on {
            expr.walk(f);
        }
    }

    fn collect_expressions<'a>(&'a self, out: &mut Vec<&'a Expression>) {
        self.table.collect_expressions(out);
        if let Some(expr) = &self.on {
            out.push(expr);
        }
    }

    fn collect_tables<'a>(&'a self, out: &mut Vec<&'a TableRef>) {
        self.table.collect_tables(out);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinKind {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expression,
    pub descending: bool,
}

impl OrderByItem {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::OrderByItem(self));
        self.expr.walk(f);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Vec<String>),
    String(String),
    Number(String),
    Boolean(bool),
    Null,
    OdbcDateTime(String),
    BindParameter(Option<String>),
    Star,
    QualifiedStar(Vec<String>),
    FunctionCall {
        name: Vec<String>,
        args: Vec<Expression>,
    },
    Subquery(Box<Query>),
    Case {
        branches: Vec<(Expression, Expression)>,
        else_expr: Option<Box<Expression>>,
    },
    Paren(Box<Expression>),
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    IsNull {
        expr: Box<Expression>,
        negated: bool,
    },
}

impl Expression {
    fn walk(&self, f: &mut impl FnMut(QueryNode<'_>)) {
        f(QueryNode::Expression(self));
        match self {
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    arg.walk(f);
                }
            }
            Expression::Subquery(query) => query.walk(f),
            Expression::Case { branches, else_expr } => {
                for (cond, value) in branches {
                    cond.walk(f);
                    value.walk(f);
                }
                if let Some(expr) = else_expr {
                    expr.walk(f);
                }
            }
            Expression::Paren(inner)
            | Expression::Unary { expr: inner, .. }
            | Expression::IsNull { expr: inner, .. } => inner.walk(f),
            Expression::Binary { left, right, .. } => {
                left.walk(f);
                right.walk(f);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryNode<'a> {
    Query(&'a Query),
    SelectStatement(&'a SelectStatement),
    SelectItem(&'a SelectItem),
    TableRef(&'a TableRef),
    JoinClause(&'a JoinClause),
    OrderByItem(&'a OrderByItem),
    Expression(&'a Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    Contains,
    InstanceOf,
    CastAs,
    Xor,
    Eqv,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Pow,
}

pub fn parse(source: &str) -> ParseResult<Query> {
    let lexed = lex(source);
    Parser::new(source, lexed.tokens()).parse_query()
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, tokens: &'a [Token]) -> Self {
        Self { source, tokens, pos: 0 }
    }

    fn parse_query(&mut self) -> ParseResult<Query> {
        let query = self.parse_query_inner()?;
        self.expect_eof()?;
        Ok(query)
    }

    fn parse_query_inner(&mut self) -> ParseResult<Query> {
        let mut query = self.parse_select_query()?;
        while self.peek_is(TokenKind::Union) {
            let start = query.span;
            self.pos += 1;
            let all = self.peek_is(TokenKind::All);
            if all {
                self.pos += 1;
            }
            let right = self.parse_select_query()?;
            let span = self.merge_spans(start, right.span);
            query = Query {
                span,
                kind: QueryKind::Union {
                    left: Box::new(query),
                    all,
                    right: Box::new(right),
                },
            };
        }
        Ok(query)
    }

    fn parse_select_query(&mut self) -> ParseResult<Query> {
        let start = self.expect_kind(TokenKind::Select)?;
        let distinct = self.consume_if(TokenKind::Distinct);

        let mut projection = Vec::new();
        loop {
            projection.push(self.parse_select_item()?);
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }

        let mut from = Vec::new();
        if self.consume_if(TokenKind::From) {
            loop {
                from.push(self.parse_table_ref()?);
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
        }

        let where_clause = if self.consume_if(TokenKind::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let mut group_by = Vec::new();
        if self.consume_if(TokenKind::Group) {
            self.expect_kind(TokenKind::By)?;
            loop {
                group_by.push(self.parse_expression()?);
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
        }

        let having = if self.consume_if(TokenKind::Having) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let mut order_by = Vec::new();
        if self.consume_if(TokenKind::Order) {
            self.expect_kind(TokenKind::By)?;
            loop {
                let expr = self.parse_expression()?;
                let descending = self.consume_if(TokenKind::Desc);
                let _ascending = self.consume_if(TokenKind::Asc);
                order_by.push(OrderByItem { expr, descending });
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
        }

        let limit = if self.consume_if(TokenKind::Limit) {
            let token = self.expect_any(&[TokenKind::Number])?;
            let raw = self.token_text(token);
            Some(raw.parse::<u64>().map_err(|_| self.error(token, "invalid LIMIT value"))?)
        } else {
            None
        };

        let end = self.prev_span().unwrap_or(start);
        Ok(Query {
            span: self.merge_spans(start, end),
            kind: QueryKind::Select(SelectStatement {
                distinct,
                projection,
                from,
                where_clause,
                group_by,
                having,
                order_by,
                limit,
            }),
        })
    }

    fn parse_select_item(&mut self) -> ParseResult<SelectItem> {
        let expr = if self.consume_if(TokenKind::Asterisk) {
            Expression::Star
        } else if self.peek_is(TokenKind::Identifier) && self.peek_kind_n(1) == Some(TokenKind::Dot) && self.peek_kind_n(2) == Some(TokenKind::Asterisk) {
            let path = self.parse_path()?;
            self.expect_kind(TokenKind::Dot)?;
            self.expect_kind(TokenKind::Asterisk)?;
            Expression::QualifiedStar(path)
        } else {
            self.parse_expression()?
        };

        let alias = if self.consume_if(TokenKind::As) {
            Some(self.expect_identifier_string()?)
        } else {
            None
        };

        Ok(SelectItem { expr, alias })
    }

    fn parse_table_ref(&mut self) -> ParseResult<TableRef> {
        let source = if self.consume_if(TokenKind::LeftParen) {
            let subquery = self.parse_query_inner()?;
            self.expect_kind(TokenKind::RightParen)?;
            TableSource::Subquery(Box::new(subquery))
        } else {
            TableSource::Named(self.parse_path()?)
        };

        let alias = if self.consume_if(TokenKind::As) {
            Some(self.expect_identifier_string()?)
        } else if self.peek_is(TokenKind::Identifier) {
            Some(self.expect_identifier_string()?)
        } else {
            None
        };

        let mut joins = Vec::new();
        while self.peek_is(TokenKind::Join)
            || self.peek_is(TokenKind::Inner)
            || self.peek_is(TokenKind::Left)
            || self.peek_is(TokenKind::Right)
            || self.peek_is(TokenKind::Full)
            || self.peek_is(TokenKind::Cross)
        {
            joins.push(self.parse_join_clause()?);
        }

        Ok(TableRef { source, alias, joins })
    }

    fn parse_join_clause(&mut self) -> ParseResult<JoinClause> {
        let kind = if self.consume_if(TokenKind::Inner) {
            let _ = self.consume_if(TokenKind::Join);
            JoinKind::Inner
        } else if self.consume_if(TokenKind::Left) {
            let _ = self.consume_if(TokenKind::Outer);
            self.expect_kind(TokenKind::Join)?;
            JoinKind::Left
        } else if self.consume_if(TokenKind::Right) {
            let _ = self.consume_if(TokenKind::Outer);
            self.expect_kind(TokenKind::Join)?;
            JoinKind::Right
        } else if self.consume_if(TokenKind::Full) {
            let _ = self.consume_if(TokenKind::Outer);
            self.expect_kind(TokenKind::Join)?;
            JoinKind::Full
        } else if self.consume_if(TokenKind::Cross) {
            self.expect_kind(TokenKind::Join)?;
            JoinKind::Cross
        } else {
            self.expect_kind(TokenKind::Join)?;
            JoinKind::Inner
        };

        let table = self.parse_table_ref()?;
        let on = if self.consume_if(TokenKind::On) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(JoinClause { kind, table, on })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, min_prec: u8) -> ParseResult<Expression> {
        let mut left = self.parse_prefix_expression()?;
        loop {
            if self.peek_is(TokenKind::Is) {
                let save = self.pos;
                self.pos += 1;
                let negated = self.consume_if(TokenKind::Not);
                if self.peek_is(TokenKind::Null) {
                    self.pos += 1;
                    left = Expression::IsNull {
                        expr: Box::new(left),
                        negated,
                    };
                    continue;
                }
                self.pos = save;
            }

            let Some((prec, op)) = self.peek_binary_op() else {
                break;
            };
            if prec < min_prec {
                break;
            }
            self.pos += self.binary_op_span_len();
            let right = self.parse_binary_expression(prec + 1)?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_prefix_expression(&mut self) -> ParseResult<Expression> {
        if self.consume_if(TokenKind::Not) {
            return Ok(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(self.parse_prefix_expression()?),
            });
        }
        if self.consume_if(TokenKind::Minus) {
            return Ok(Expression::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(self.parse_prefix_expression()?),
            });
        }

        let mut expr = self.parse_primary_expression()?;
        loop {
            if self.consume_if(TokenKind::LeftParen) {
                let mut args = Vec::new();
                if !self.consume_if(TokenKind::RightParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        self.expect_kind(TokenKind::RightParen)?;
                        break;
                    }
                }
                expr = match expr {
                    Expression::Identifier(name) | Expression::QualifiedStar(name) => Expression::FunctionCall { name, args },
                    Expression::Paren(inner) => Expression::FunctionCall {
                        name: vec![self.text_for_expr_name(&inner)?],
                        args,
                    },
                    other => Expression::FunctionCall {
                        name: vec![self.expr_name(other)?],
                        args,
                    },
                };
                continue;
            }
            if self.consume_if(TokenKind::Dot) {
                let member = self.expect_identifier_string()?;
                expr = match expr {
                    Expression::Identifier(mut path) => {
                        path.push(member);
                        Expression::Identifier(path)
                    }
                    _ => Expression::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Concat,
                        right: Box::new(Expression::Identifier(vec![member])),
                    },
                };
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> ParseResult<Expression> {
        if self.consume_if(TokenKind::LeftParen) {
            if self.peek_is(TokenKind::Select) {
                let subquery = self.parse_query_inner()?;
                self.expect_kind(TokenKind::RightParen)?;
                return Ok(Expression::Subquery(Box::new(subquery)));
            }
            let expr = self.parse_expression()?;
            self.expect_kind(TokenKind::RightParen)?;
            return Ok(Expression::Paren(Box::new(expr)));
        }
        if self.consume_if(TokenKind::Case) {
            let mut branches = Vec::new();
            while self.consume_if(TokenKind::When) {
                let cond = self.parse_expression()?;
                self.expect_kind(TokenKind::Then)?;
                let then_expr = self.parse_expression()?;
                branches.push((cond, then_expr));
            }
            let else_expr = if self.consume_if(TokenKind::Else) {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            self.expect_kind(TokenKind::End)?;
            return Ok(Expression::Case { branches, else_expr });
        }
        if self.peek_is(TokenKind::String) {
            return Ok(Expression::String(self.consume_raw_text()?));
        }
        if self.peek_is(TokenKind::Number) {
            return Ok(Expression::Number(self.consume_raw_text()?));
        }
        if self.consume_if(TokenKind::True) {
            return Ok(Expression::Boolean(true));
        }
        if self.consume_if(TokenKind::False) {
            return Ok(Expression::Boolean(false));
        }
        if self.consume_if(TokenKind::Null) {
            return Ok(Expression::Null);
        }
        if self.peek_is(TokenKind::BindParameter) {
            let text = self.consume_raw_text()?;
            return Ok(Expression::BindParameter(text.strip_prefix(':').map(|s| s.to_string())));
        }
        if self.peek_is(TokenKind::OdbcDateTimeLiteral) {
            return Ok(Expression::OdbcDateTime(self.consume_raw_text()?));
        }
        if self.peek_is(TokenKind::Asterisk) {
            self.pos += 1;
            return Ok(Expression::Star);
        }
        if self.peek_is(TokenKind::Select) {
            let _subquery = self.parse_select_query()?;
            return Ok(Expression::Paren(Box::new(Expression::Identifier(vec!["subquery".to_string()]))));
        }
        if self.peek_is(TokenKind::Identifier) {
            let path = self.parse_path()?;
            if self.peek_is(TokenKind::LeftParen) {
                return Ok(Expression::Identifier(path));
            }
            return Ok(Expression::Identifier(path));
        }
        Err(self.error_current("expected expression"))
    }

    fn peek_binary_op(&self) -> Option<(u8, BinaryOp)> {
        match self.peek_kind()? {
            TokenKind::Or => Some((1, BinaryOp::Or)),
            TokenKind::Xor => Some((1, BinaryOp::Xor)),
            TokenKind::Eqv => Some((1, BinaryOp::Eqv)),
            TokenKind::AmpAmp | TokenKind::And => Some((2, BinaryOp::And)),
            TokenKind::EqualEqual | TokenKind::Equal => Some((3, BinaryOp::Eq)),
            TokenKind::BangEqual | TokenKind::NotEqualAngle => Some((3, BinaryOp::NotEq)),
            TokenKind::Less => Some((3, BinaryOp::Lt)),
            TokenKind::LessEqual => Some((3, BinaryOp::Lte)),
            TokenKind::Greater => Some((3, BinaryOp::Gt)),
            TokenKind::GreaterEqual => Some((3, BinaryOp::Gte)),
            TokenKind::Like => Some((3, BinaryOp::Like)),
            TokenKind::Contains => Some((3, BinaryOp::Contains)),
            TokenKind::InstanceOf => Some((3, BinaryOp::InstanceOf)),
            TokenKind::CastAs => Some((3, BinaryOp::CastAs)),
            TokenKind::Plus => Some((4, BinaryOp::Add)),
            TokenKind::Minus => Some((4, BinaryOp::Sub)),
            TokenKind::Asterisk => Some((5, BinaryOp::Mul)),
            TokenKind::Slash => Some((5, BinaryOp::Div)),
            TokenKind::Percent => Some((5, BinaryOp::Mod)),
            TokenKind::Ampersand => Some((6, BinaryOp::Concat)),
            TokenKind::Caret => Some((7, BinaryOp::Pow)),
            _ => None,
        }
    }

    fn binary_op_span_len(&self) -> usize {
        1
    }

    fn parse_path(&mut self) -> ParseResult<Vec<String>> {
        let mut path = vec![self.expect_identifier_string()?];
        while self.consume_if(TokenKind::Dot) {
            path.push(self.expect_identifier_string()?);
        }
        Ok(path)
    }

    fn expect_identifier_string(&mut self) -> ParseResult<String> {
        if self.peek_is(TokenKind::Identifier) {
            Ok(self.consume_raw_text()?)
        } else {
            Err(self.error_current("expected identifier"))
        }
    }

    fn consume_raw_text(&mut self) -> ParseResult<String> {
        let token = *self
            .current_token()
            .ok_or_else(|| self.error_eof("unexpected EOF"))?;
        self.pos += 1;
        Ok(self.token_text(token).to_string())
    }

    fn expect_kind(&mut self, kind: TokenKind) -> ParseResult<Span> {
        if self.peek_is(kind) {
            let span = self.current_span().unwrap();
            self.pos += 1;
            Ok(span)
        } else {
            Err(self.error_current(&format!("expected {:?}", kind)))
        }
    }

    fn expect_any(&mut self, kinds: &[TokenKind]) -> ParseResult<Token> {
        if kinds.iter().any(|kind| self.peek_is(*kind)) {
            let token = *self.current_token().unwrap();
            self.pos += 1;
            Ok(token)
        } else {
            Err(self.error_current("unexpected token"))
        }
    }

    fn consume_if(&mut self, kind: TokenKind) -> bool {
        if self.peek_is(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        self.peek_kind() == Some(kind)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.current_token().map(|t| t.kind)
    }

    fn peek_kind_n(&self, n: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos + n).map(|t| t.kind)
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn current_span(&self) -> Option<Span> {
        self.current_token().map(|t| t.span)
    }

    fn prev_span(&self) -> Option<Span> {
        self.tokens.get(self.pos.saturating_sub(1)).map(|t| t.span)
    }

    fn token_text(&self, token: Token) -> &str {
        self.source[token.span.start..token.span.end].as_ref()
    }

    fn merge_spans(&self, start: Span, end: Span) -> Span {
        Span {
            start: start.start,
            end: end.end,
            line: start.line,
            col: start.col,
        }
    }

    fn expect_eof(&mut self) -> ParseResult<()> {
        if self.peek_is(TokenKind::Eof) {
            Ok(())
        } else {
            Err(self.error_current("unexpected trailing tokens"))
        }
    }

    fn error_current(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            span: self.current_span().unwrap_or(Span {
                start: self.source.len(),
                end: self.source.len(),
                line: 0,
                col: 0,
            }),
        }
    }

    fn error_eof(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            span: Span {
                start: self.source.len(),
                end: self.source.len(),
                line: self.current_span().map(|s| s.line).unwrap_or(0),
                col: self.current_span().map(|s| s.col).unwrap_or(0),
            },
        }
    }

    fn error(&self, token: Token, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            span: token.span,
        }
    }

    fn text_for_expr_name(&self, expr: &Expression) -> ParseResult<String> {
        self.expr_name(expr.clone())
    }

    fn expr_name(&self, expr: Expression) -> ParseResult<String> {
        match expr {
            Expression::Identifier(parts) => Ok(parts.join(".")),
            Expression::Paren(inner) => self.expr_name(*inner),
            _ => Err(self.error_current("expected function name")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Default)]
pub struct BindParams {
    pub positional: Vec<SqlValue>,
    pub named: HashMap<String, SqlValue>,
}

pub fn bind_params(query: &Query, params: &BindParams) -> Result<Query, ExecutionError> {
    let mut positional = 0usize;
    bind_query(query, params, &mut positional)
}

fn bind_query(
    query: &Query,
    params: &BindParams,
    positional: &mut usize,
) -> Result<Query, ExecutionError> {
    Ok(Query {
        span: query.span,
        kind: match &query.kind {
            QueryKind::Select(select) => QueryKind::Select(bind_select(select, params, positional)?),
            QueryKind::Union { left, all, right } => QueryKind::Union {
                left: Box::new(bind_query(left, params, positional)?),
                all: *all,
                right: Box::new(bind_query(right, params, positional)?),
            },
        },
    })
}

fn bind_select(
    select: &SelectStatement,
    params: &BindParams,
    positional: &mut usize,
) -> Result<SelectStatement, ExecutionError> {
    Ok(SelectStatement {
        distinct: select.distinct,
        projection: select
            .projection
            .iter()
            .map(|item| bind_select_item(item, params, positional))
            .collect::<Result<_, _>>()?,
        from: select
            .from
            .iter()
            .map(|table| bind_table_ref(table, params, positional))
            .collect::<Result<_, _>>()?,
        where_clause: select
            .where_clause
            .as_ref()
            .map(|expr| bind_expression(expr, params, positional))
            .transpose()?,
        group_by: select
            .group_by
            .iter()
            .map(|expr| bind_expression(expr, params, positional))
            .collect::<Result<_, _>>()?,
        having: select
            .having
            .as_ref()
            .map(|expr| bind_expression(expr, params, positional))
            .transpose()?,
        order_by: select
            .order_by
            .iter()
            .map(|item| bind_order_by_item(item, params, positional))
            .collect::<Result<_, _>>()?,
        limit: select.limit,
    })
}

fn bind_select_item(
    item: &SelectItem,
    params: &BindParams,
    positional: &mut usize,
) -> Result<SelectItem, ExecutionError> {
    Ok(SelectItem {
        expr: bind_expression(&item.expr, params, positional)?,
        alias: item.alias.clone(),
    })
}

fn bind_table_ref(
    table: &TableRef,
    params: &BindParams,
    positional: &mut usize,
) -> Result<TableRef, ExecutionError> {
    Ok(TableRef {
        source: match &table.source {
            TableSource::Named(path) => TableSource::Named(path.clone()),
            TableSource::Subquery(query) => TableSource::Subquery(Box::new(bind_query(query, params, positional)?)),
        },
        alias: table.alias.clone(),
        joins: table
            .joins
            .iter()
            .map(|join| bind_join_clause(join, params, positional))
            .collect::<Result<_, _>>()?,
    })
}

fn bind_join_clause(
    join: &JoinClause,
    params: &BindParams,
    positional: &mut usize,
) -> Result<JoinClause, ExecutionError> {
    Ok(JoinClause {
        kind: join.kind.clone(),
        table: bind_table_ref(&join.table, params, positional)?,
        on: join
            .on
            .as_ref()
            .map(|expr| bind_expression(expr, params, positional))
            .transpose()?,
    })
}

fn bind_order_by_item(
    item: &OrderByItem,
    params: &BindParams,
    positional: &mut usize,
) -> Result<OrderByItem, ExecutionError> {
    Ok(OrderByItem {
        expr: bind_expression(&item.expr, params, positional)?,
        descending: item.descending,
    })
}

fn bind_expression(
    expr: &Expression,
    params: &BindParams,
    positional: &mut usize,
) -> Result<Expression, ExecutionError> {
    Ok(match expr {
        Expression::BindParameter(Some(name)) => {
            let key = name.to_lowercase();
            let value = params.named.get(&key).ok_or_else(|| ExecutionError {
                message: format!("missing named QoQ parameter '{}'", name),
                span: None,
            })?;
            sql_value_to_expression(value)
        }
        Expression::BindParameter(None) => {
            let value = params.positional.get(*positional).ok_or_else(|| ExecutionError {
                message: "missing positional QoQ parameter".to_string(),
                span: None,
            })?;
            *positional += 1;
            sql_value_to_expression(value)
        }
        Expression::FunctionCall { name, args } => Expression::FunctionCall {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| bind_expression(arg, params, positional))
                .collect::<Result<_, _>>()?,
        },
        Expression::Subquery(query) => Expression::Subquery(Box::new(bind_query(query, params, positional)?)),
        Expression::Case { branches, else_expr } => Expression::Case {
            branches: branches
                .iter()
                .map(|(cond, value)| {
                    Ok((
                        bind_expression(cond, params, positional)?,
                        bind_expression(value, params, positional)?,
                    ))
                })
                .collect::<Result<Vec<_>, ExecutionError>>()?,
            else_expr: else_expr
                .as_ref()
                .map(|expr| bind_expression(expr, params, positional).map(Box::new))
                .transpose()?,
        },
        Expression::Paren(inner) => Expression::Paren(Box::new(bind_expression(inner, params, positional)?)),
        Expression::Unary { op, expr } => Expression::Unary {
            op: op.clone(),
            expr: Box::new(bind_expression(expr, params, positional)?),
        },
        Expression::Binary { left, op, right } => Expression::Binary {
            left: Box::new(bind_expression(left, params, positional)?),
            op: op.clone(),
            right: Box::new(bind_expression(right, params, positional)?),
        },
        Expression::IsNull { expr, negated } => Expression::IsNull {
            expr: Box::new(bind_expression(expr, params, positional)?),
            negated: *negated,
        },
        Expression::Identifier(parts) => Expression::Identifier(parts.clone()),
        Expression::String(s) => Expression::String(s.clone()),
        Expression::Number(n) => Expression::Number(n.clone()),
        Expression::Boolean(b) => Expression::Boolean(*b),
        Expression::Null => Expression::Null,
        Expression::OdbcDateTime(s) => Expression::OdbcDateTime(s.clone()),
        Expression::Star => Expression::Star,
        Expression::QualifiedStar(parts) => Expression::QualifiedStar(parts.clone()),
    })
}

fn sql_value_to_expression(value: &SqlValue) -> Expression {
    match value {
        SqlValue::Null => Expression::Null,
        SqlValue::Bool(b) => Expression::Boolean(*b),
        SqlValue::Int(i) => Expression::Number(i.to_string()),
        SqlValue::Float(f) => Expression::Number({
            let mut s = f.to_string();
            if s.ends_with(".0") {
                s.truncate(s.len() - 2);
            }
            s
        }),
        SqlValue::Text(s) => Expression::String(s.clone()),
        SqlValue::Bytes(bytes) => Expression::String(String::from_utf8_lossy(bytes).to_string()),
    }
}

pub fn execute(
    query: &Query,
    sources: &HashMap<String, QueryResult>,
) -> Result<QueryResult, ExecutionError> {
    execute_with_source_resolver(query, |path| {
        let key = path.join(".");
        Ok(sources
            .iter()
            .find(|(candidate, _)| candidate.eq_ignore_ascii_case(&key))
            .map(|(_, result)| result.clone()))
    })
}

pub fn execute_with_source_resolver<'a, F>(
    query: &Query,
    source_resolver: F,
) -> Result<QueryResult, ExecutionError>
where
    F: FnMut(&[String]) -> Result<Option<QueryResult>, ExecutionError> + 'a,
{
    QueryExecutor::new(source_resolver).execute(query)
}

pub struct QueryExecutor<'a> {
    source_resolver: std::cell::RefCell<Box<SourceResolver<'a>>>,
}

type SourceResolver<'a> =
    dyn FnMut(&[String]) -> Result<Option<QueryResult>, ExecutionError> + 'a;

impl<'a> QueryExecutor<'a> {
    pub fn new<F>(source_resolver: F) -> Self
    where
        F: FnMut(&[String]) -> Result<Option<QueryResult>, ExecutionError> + 'a,
    {
        Self {
            source_resolver: std::cell::RefCell::new(Box::new(source_resolver)),
        }
    }

    pub fn execute(&self, query: &Query) -> Result<QueryResult, ExecutionError> {
        match &query.kind {
            QueryKind::Select(select) => {
                if let Some(result) = self.execute_fast_simple_aggregate(select)? {
                    return Ok(result);
                }
                self.execute_select(select)
            }
            QueryKind::Union { left, right, all } => {
                let mut left_result = self.execute(left)?;
                let right_result = self.execute(right)?;
                left_result.rows.extend(right_result.rows);
                left_result.columns = merge_union_columns(&left_result.columns, &right_result.columns);
                if !all {
                    dedupe_rows(&mut left_result.rows);
                }
                Ok(left_result)
            }
        }
    }

    fn resolve_source(&self, path: &[String]) -> Result<Option<QueryResult>, ExecutionError> {
        (self.source_resolver.borrow_mut())(path)
    }

    fn execute_fast_simple_aggregate(
        &self,
        select: &SelectStatement,
    ) -> Result<Option<QueryResult>, ExecutionError> {
        if select.distinct
            || !select.group_by.is_empty()
            || select.having.is_some()
            || !select.order_by.is_empty()
            || select.limit.is_some()
            || select.where_clause.is_some()
            || select.from.len() != 1
            || !select.from[0].joins.is_empty()
            || !matches!(select.from[0].source, TableSource::Named(_))
            || select.projection.len() != 1
        {
            return Ok(None);
        }

        let item = &select.projection[0];
        let (func_name, arg_expr) = match &item.expr {
            Expression::FunctionCall { name, args } if args.len() == 1 => (name, &args[0]),
            _ => return Ok(None),
        };

        if !matches!(func_name.last().map(|s| s.as_str()), Some(name) if name.eq_ignore_ascii_case("avg")) {
            return Ok(None);
        }

        let source_path = match &select.from[0].source {
            TableSource::Named(path) => path,
            TableSource::Subquery(_) => return Ok(None),
        };
        let result = self
            .resolve_source(source_path)?
            .ok_or_else(|| self.error(format!("unknown table '{}'", source_path.join(".")), None))?;

        let column_name = match arg_expr {
            Expression::Identifier(path) if path.len() == 1 => &path[0],
            _ => return Ok(None),
        };

        let col_idx = result
            .columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| self.error(format!("unknown identifier '{}'", column_name), None))?;

        let mut total = 0.0;
        let mut count = 0.0;
        for row in &result.rows {
            if let Some(value) = row.get(col_idx).and_then(sql_as_f64_ref) {
                total += value;
                count += 1.0;
            }
        }

        let alias = item
            .alias
            .clone()
            .unwrap_or_else(|| projection_name(&item.expr, 0));

        let output = if count == 0.0 {
            SqlValue::Null
        } else {
            SqlValue::Float(total / count)
        };

        Ok(Some(QueryResult {
            columns: vec![QueryColumn {
                name: alias,
                col_type: QueryColumnType::Double,
            }],
            rows: vec![vec![output]],
        }))
    }

    fn execute_select(&self, select: &SelectStatement) -> Result<QueryResult, ExecutionError> {
        let mut rows = self.resolve_from(&select.from)?;

        if let Some(expr) = &select.where_clause {
            rows.retain(|row| self.eval_truthy(expr, row, None).unwrap_or(false));
        }

        let grouped = select.group_by.iter().any(|_| true)
            || select
                .projection
                .iter()
                .any(|item| contains_aggregate(&item.expr))
            || select
                .having
                .as_ref()
                .map(|expr| contains_aggregate(expr))
                .unwrap_or(false);

        let mut output_rows = if grouped {
            self.execute_grouped(select, &rows)?
        } else {
            self.execute_rowwise(select, &rows)?
        };

        if select.distinct {
            dedupe_rows(&mut output_rows.rows);
        }

        if !select.order_by.is_empty() {
            self.apply_order_by(select, &mut output_rows)?;
        }

        if let Some(limit) = select.limit {
            output_rows.rows.truncate(limit as usize);
        }

        Ok(output_rows)
    }

    fn execute_rowwise(
        &self,
        select: &SelectStatement,
        rows: &[ResolvedRow],
    ) -> Result<QueryResult, ExecutionError> {
        let mut out_rows = Vec::new();
        let mut columns = Vec::new();
        for row in rows {
            let mut out_row = Vec::new();
            for item in &select.projection {
                let value = match &item.expr {
                    Expression::Star => {
                        return Err(self.error("star projection requires query expansion", None))
                    }
                    Expression::QualifiedStar(_) => {
                        return Err(self.error("qualified star projection requires query expansion", None))
                    }
                    expr => self.eval(expr, row, None)?,
                };
                out_row.push(value);
            }
            if columns.is_empty() {
                columns = self.projection_columns(select, row)?;
            }
            out_rows.push(out_row);
        }
        Ok(QueryResult { columns, rows: out_rows })
    }

    fn execute_grouped(
        &self,
        select: &SelectStatement,
        rows: &[ResolvedRow],
    ) -> Result<QueryResult, ExecutionError> {
        let mut groups: HashMap<Vec<String>, Vec<&ResolvedRow>> = HashMap::new();
        if select.group_by.is_empty() {
            groups.insert(vec!["__all__".to_string()], rows.iter().collect());
        } else {
            for row in rows {
                let mut key = Vec::new();
                for expr in &select.group_by {
                    key.push(sql_value_key(&self.eval(expr, row, None)?));
                }
                groups.entry(key).or_default().push(row);
            }
        }

        let mut out_rows = Vec::new();
        let mut columns = Vec::new();
        for group_rows in groups.values() {
            let first = *group_rows.first().ok_or_else(|| self.error("empty group", None))?;
            if let Some(having) = &select.having {
                if !self.eval_truthy(having, first, Some(group_rows))? {
                    continue;
                }
            }

            let mut out_row = Vec::new();
            for item in &select.projection {
                let value = self.eval(&item.expr, first, Some(group_rows))?;
                out_row.push(value);
            }
            if columns.is_empty() {
                columns = self.projection_columns(select, first)?;
            }
            out_rows.push(out_row);
        }
        Ok(QueryResult { columns, rows: out_rows })
    }

    fn projection_columns(
        &self,
        select: &SelectStatement,
        row: &ResolvedRow,
    ) -> Result<Vec<QueryColumn>, ExecutionError> {
        let mut columns = Vec::new();
        for (idx, item) in select.projection.iter().enumerate() {
            let name = item.alias.clone().unwrap_or_else(|| projection_name(&item.expr, idx));
            let value = match &item.expr {
                Expression::Star | Expression::QualifiedStar(_) => SqlValue::Null,
                expr => self.eval(expr, row, None).unwrap_or(SqlValue::Null),
            };
            columns.push(QueryColumn {
                name,
                col_type: infer_query_type(&value),
            });
        }
        Ok(columns)
    }

    fn apply_order_by(
        &self,
        select: &SelectStatement,
        result: &mut QueryResult,
    ) -> Result<(), ExecutionError> {
        let order_specs = select.order_by.clone();
        let columns = result.columns.clone();
        result.rows.sort_by(|a, b| {
            for spec in &order_specs {
                let idx = match &spec.expr {
                    Expression::Number(n) => n.parse::<usize>().ok().and_then(|n| n.checked_sub(1)),
                    Expression::Identifier(path) if path.len() == 1 => {
                        columns.iter().position(|c| c.name.eq_ignore_ascii_case(&path[0]))
                    }
                    _ => None,
                };
                let ord = idx
                    .and_then(|i| a.get(i).zip(b.get(i)))
                    .and_then(|(l, r)| compare_sql_values(l, r))
                    .unwrap_or(Ordering::Equal);
                if ord != Ordering::Equal {
                    return if spec.descending { ord.reverse() } else { ord };
                }
            }
            Ordering::Equal
        });
        Ok(())
    }

    fn resolve_from(&self, from: &[TableRef]) -> Result<Vec<ResolvedRow>, ExecutionError> {
        if from.is_empty() {
            return Ok(vec![ResolvedRow::default()]);
        }

        let mut current = self.resolve_table_ref(&from[0])?;
        for table in &from[1..] {
            let next = self.resolve_table_ref(table)?;
            current = cross_join(&current, &next, None)?;
        }
        Ok(current.rows)
    }

    fn resolve_table_ref(&self, table: &TableRef) -> Result<ResolvedTable, ExecutionError> {
        let base = match &table.source {
            TableSource::Named(path) => self.resolve_source(path)?.ok_or_else(|| {
                self.error(format!("unknown table '{}'", path.join(".")), None)
            })?,
            TableSource::Subquery(query) => self.execute(query)?,
        };

        let alias = table
            .alias
            .clone()
            .or_else(|| match &table.source {
                TableSource::Named(path) => path.last().cloned(),
                TableSource::Subquery(_) => Some("subquery".to_string()),
            })
            .unwrap_or_else(|| "query".to_string());

        let mut resolved = materialize_table(&base, &alias);
        for join in &table.joins {
            let right = self.resolve_table_ref(&join.table)?;
            resolved = apply_join(resolved, right, join, self)?;
        }
        Ok(resolved)
    }

    fn eval_truthy(
        &self,
        expr: &Expression,
        row: &ResolvedRow,
        group_rows: Option<&[&ResolvedRow]>,
    ) -> Result<bool, ExecutionError> {
        Ok(is_truthy(&self.eval(expr, row, group_rows)?))
    }

    fn eval(
        &self,
        expr: &Expression,
        row: &ResolvedRow,
        group_rows: Option<&[&ResolvedRow]>,
    ) -> Result<SqlValue, ExecutionError> {
        match expr {
            Expression::Identifier(path) => row
                .lookup(&path.join("."))
                .cloned()
                .ok_or_else(|| self.error(format!("unknown identifier '{}'", path.join(".")), None)),
            Expression::String(s) => Ok(SqlValue::Text(s.clone())),
            Expression::Number(n) => parse_sql_number(n),
            Expression::Boolean(b) => Ok(SqlValue::Bool(*b)),
            Expression::Null => Ok(SqlValue::Null),
            Expression::OdbcDateTime(s) => Ok(SqlValue::Text(s.clone())),
            Expression::BindParameter(_) => Err(self.error("bind parameters are not supported in QoQ execution yet", None)),
            Expression::Star | Expression::QualifiedStar(_) => Err(self.error("star is only valid in select projection", None)),
            Expression::Paren(inner) => self.eval(inner, row, group_rows),
            Expression::Unary { op, expr } => {
                let value = self.eval(expr, row, group_rows)?;
                match op {
                    UnaryOp::Not => Ok(SqlValue::Bool(!is_truthy(&value))),
                    UnaryOp::Negate => negate_sql_value(value),
                }
            }
            Expression::Binary { left, op, right } => {
                let left = self.eval(left, row, group_rows)?;
                let right = self.eval(right, row, group_rows)?;
                eval_binary(left, op, right)
            }
            Expression::IsNull { expr, negated } => {
                let value = self.eval(expr, row, group_rows)?;
                Ok(SqlValue::Bool(if *negated { !matches!(value, SqlValue::Null) } else { matches!(value, SqlValue::Null) }))
            }
            Expression::FunctionCall { name, args } => {
                self.eval_function(name, args, row, group_rows)
            }
            Expression::Subquery(query) => {
                let result = self.execute(query)?;
                if result.rows.len() == 1 && result.rows[0].len() == 1 {
                    Ok(result.rows[0][0].clone())
                } else {
                    Err(self.error("subquery expressions must return a single value", Some(query.span)))
                }
            }
            Expression::Case { branches, else_expr } => {
                for (cond, value) in branches {
                    if is_truthy(&self.eval(cond, row, group_rows)?) {
                        return self.eval(value, row, group_rows);
                    }
                }
                if let Some(expr) = else_expr {
                    self.eval(expr, row, group_rows)
                } else {
                    Ok(SqlValue::Null)
                }
            }
        }
    }

    fn eval_function(
        &self,
        name: &[String],
        args: &[Expression],
        row: &ResolvedRow,
        group_rows: Option<&[&ResolvedRow]>,
    ) -> Result<SqlValue, ExecutionError> {
        let fname = name.last().map(|s| s.to_lowercase()).unwrap_or_default();
        if let Some(group_rows) = group_rows {
            match fname.as_str() {
                "count" => {
                    if args.is_empty() || matches!(args.first(), Some(Expression::Star)) {
                        return Ok(SqlValue::Int(group_rows.len() as i64));
                    }
                    let mut count = 0i64;
                    for r in group_rows {
                        if !matches!(self.eval(&args[0], r, None)?, SqlValue::Null) {
                            count += 1;
                        }
                    }
                    return Ok(SqlValue::Int(count));
                }
                "sum" => return aggregate_numeric(group_rows, |acc, v| acc + v, 0.0, &args[0], self, row),
                "min" => return aggregate_minmax(group_rows, true, &args[0], self, row),
                "max" => return aggregate_minmax(group_rows, false, &args[0], self, row),
                "avg" => {
                    let mut total = 0.0;
                    let mut count = 0.0;
                    for r in group_rows {
                        let val = self.eval(&args[0], r, None)?;
                        if let Some(n) = sql_as_f64(&val) {
                            total += n;
                            count += 1.0;
                        }
                    }
                    return Ok(if count == 0.0 { SqlValue::Null } else { SqlValue::Float(total / count) });
                }
                _ => {}
            }
        }

        match fname.as_str() {
            "lower" => {
                let value = self.eval(&args[0], row, None)?;
                Ok(SqlValue::Text(sql_to_string(&value).to_lowercase()))
            }
            "upper" => {
                let value = self.eval(&args[0], row, None)?;
                Ok(SqlValue::Text(sql_to_string(&value).to_uppercase()))
            }
            "trim" => {
                let value = self.eval(&args[0], row, None)?;
                Ok(SqlValue::Text(sql_to_string(&value).trim().to_string()))
            }
            "count" => Ok(SqlValue::Int(1)),
            _ => Err(self.error(format!("unsupported QoQ function '{}'", fname), None)),
        }
    }

    fn error(&self, message: impl Into<String>, span: Option<Span>) -> ExecutionError {
        ExecutionError {
            message: message.into(),
            span,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ResolvedRow {
    values: HashMap<String, SqlValue>,
}

impl ResolvedRow {
    fn lookup(&self, name: &str) -> Option<&SqlValue> {
        self.values.get(&name.to_lowercase())
    }

    fn insert(&mut self, name: impl Into<String>, value: SqlValue) {
        self.values.insert(name.into().to_lowercase(), value);
    }
}

#[derive(Clone, Debug)]
struct ResolvedTable {
    columns: Vec<QueryColumn>,
    rows: Vec<ResolvedRow>,
}

fn materialize_table(result: &QueryResult, alias: &str) -> ResolvedTable {
    let mut rows = Vec::new();
    for row in &result.rows {
        let mut resolved = ResolvedRow::default();
        for (idx, col) in result.columns.iter().enumerate() {
            let value = row.get(idx).cloned().unwrap_or(SqlValue::Null);
            let col_name = col.name.to_lowercase();
            resolved.insert(col_name.clone(), value.clone());
            resolved.insert(format!("{alias}.{}", col.name), value);
        }
        rows.push(resolved);
    }

    let columns = result.columns.clone();
    ResolvedTable { columns, rows }
}

fn cross_join(
    left: &ResolvedTable,
    right: &ResolvedTable,
    on: Option<&Expression>,
) -> Result<ResolvedTable, ExecutionError> {
    let mut rows = Vec::new();
    for l in &left.rows {
        for r in &right.rows {
            let mut merged = l.clone();
            for (k, v) in &r.values {
                merged.values.insert(k.clone(), v.clone());
            }
            if let Some(expr) = on {
                // `on` is evaluated by the caller once a QueryExecutor exists.
                let _ = expr;
            }
            rows.push(merged);
        }
    }

    let mut columns = left.columns.clone();
    columns.extend(right.columns.clone());
    Ok(ResolvedTable { columns, rows })
}

fn apply_join(
    left: ResolvedTable,
    right: ResolvedTable,
    join: &JoinClause,
    executor: &QueryExecutor<'_>,
) -> Result<ResolvedTable, ExecutionError> {
    let mut rows = Vec::new();
    let mut matched_right = vec![false; right.rows.len()];

    for l in &left.rows {
        let mut matched = false;
        for (idx, r) in right.rows.iter().enumerate() {
            let mut merged = l.clone();
            for (k, v) in &r.values {
                merged.values.insert(k.clone(), v.clone());
            }
            let passes = match &join.on {
                Some(expr) => executor.eval_truthy(expr, &merged, None)?,
                None => true,
            };
            if passes {
                matched = true;
                matched_right[idx] = true;
                rows.push(merged);
            }
        }
        if !matched && matches!(join.kind, JoinKind::Left | JoinKind::Full) {
            rows.push(l.clone());
        }
    }

    if matches!(join.kind, JoinKind::Right | JoinKind::Full) {
        for (idx, r) in right.rows.iter().enumerate() {
            if !matched_right[idx] {
                rows.push(r.clone());
            }
        }
    }

    Ok(ResolvedTable {
        columns: merge_columns(&left.columns, &right.columns),
        rows,
    })
}

fn merge_columns(left: &[QueryColumn], right: &[QueryColumn]) -> Vec<QueryColumn> {
    let mut columns = left.to_vec();
    columns.extend(right.iter().cloned());
    columns
}

fn merge_union_columns(left: &[QueryColumn], right: &[QueryColumn]) -> Vec<QueryColumn> {
    if left.len() >= right.len() {
        left.to_vec()
    } else {
        right.to_vec()
    }
}

fn dedupe_rows(rows: &mut Vec<Vec<SqlValue>>) {
    let mut seen = HashSet::new();
    rows.retain(|row| seen.insert(row_key(row)));
}

fn row_key(row: &[SqlValue]) -> String {
    row.iter().map(sql_value_key).collect::<Vec<_>>().join("|")
}

fn sql_value_key(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => "null".to_string(),
        SqlValue::Bool(b) => format!("b:{b}"),
        SqlValue::Int(i) => format!("i:{i}"),
        SqlValue::Float(f) => format!("f:{f}"),
        SqlValue::Text(s) => format!("s:{s}"),
        SqlValue::Bytes(b) => format!("x:{:x?}", b),
    }
}

fn compare_sql_values(left: &SqlValue, right: &SqlValue) -> Option<Ordering> {
    match (left, right) {
        (SqlValue::Null, SqlValue::Null) => Some(Ordering::Equal),
        (SqlValue::Bool(a), SqlValue::Bool(b)) => Some(a.cmp(b)),
        _ => {
            if let (Some(a), Some(b)) = (sql_as_f64(left), sql_as_f64(right)) {
                a.partial_cmp(&b)
            } else {
                Some(sql_to_string(left).cmp(&sql_to_string(right)))
            }
        }
    }
}

fn eval_binary(left: SqlValue, op: &BinaryOp, right: SqlValue) -> Result<SqlValue, ExecutionError> {
    Ok(match op {
        BinaryOp::Or => SqlValue::Bool(is_truthy(&left) || is_truthy(&right)),
        BinaryOp::And => SqlValue::Bool(is_truthy(&left) && is_truthy(&right)),
        BinaryOp::Eq => SqlValue::Bool(sql_equals(&left, &right)),
        BinaryOp::NotEq => SqlValue::Bool(!sql_equals(&left, &right)),
        BinaryOp::Lt => SqlValue::Bool(compare_sql_values(&left, &right).is_some_and(|o| o == Ordering::Less)),
        BinaryOp::Lte => SqlValue::Bool(compare_sql_values(&left, &right).is_some_and(|o| o != Ordering::Greater)),
        BinaryOp::Gt => SqlValue::Bool(compare_sql_values(&left, &right).is_some_and(|o| o == Ordering::Greater)),
        BinaryOp::Gte => SqlValue::Bool(compare_sql_values(&left, &right).is_some_and(|o| o != Ordering::Less)),
        BinaryOp::Like => SqlValue::Bool(sql_to_string(&left).contains(&sql_to_string(&right))),
        BinaryOp::Contains => SqlValue::Bool(sql_to_string(&left).contains(&sql_to_string(&right))),
        BinaryOp::InstanceOf => SqlValue::Bool(sql_to_string(&left).eq_ignore_ascii_case(&sql_to_string(&right))),
        BinaryOp::CastAs => left,
        BinaryOp::Xor => SqlValue::Bool(is_truthy(&left) ^ is_truthy(&right)),
        BinaryOp::Eqv => SqlValue::Bool(is_truthy(&left) == is_truthy(&right)),
        BinaryOp::Add => numeric_binop(left, right, |a, b| a + b)?,
        BinaryOp::Sub => numeric_binop(left, right, |a, b| a - b)?,
        BinaryOp::Mul => numeric_binop(left, right, |a, b| a * b)?,
        BinaryOp::Div => numeric_binop(left, right, |a, b| a / b)?,
        BinaryOp::Mod => numeric_binop(left, right, |a, b| a % b)?,
        BinaryOp::Concat => SqlValue::Text(format!("{}{}", sql_to_string(&left), sql_to_string(&right))),
        BinaryOp::Pow => numeric_binop(left, right, |a, b| a.powf(b))?,
    })
}

fn numeric_binop(
    left: SqlValue,
    right: SqlValue,
    op: impl FnOnce(f64, f64) -> f64,
) -> Result<SqlValue, ExecutionError> {
    let a = sql_as_f64(&left).ok_or_else(|| ExecutionError {
        message: "numeric operator expected number".to_string(),
        span: None,
    })?;
    let b = sql_as_f64(&right).ok_or_else(|| ExecutionError {
        message: "numeric operator expected number".to_string(),
        span: None,
    })?;
    Ok(SqlValue::Float(op(a, b)))
}

fn negate_sql_value(value: SqlValue) -> Result<SqlValue, ExecutionError> {
    Ok(SqlValue::Float(-sql_as_f64(&value).ok_or_else(|| ExecutionError {
        message: "unary - expected number".to_string(),
        span: None,
    })?))
}

fn parse_sql_number(text: &str) -> Result<SqlValue, ExecutionError> {
    if let Ok(i) = text.parse::<i64>() {
        Ok(SqlValue::Int(i))
    } else if let Ok(f) = text.parse::<f64>() {
        Ok(SqlValue::Float(f))
    } else {
        Err(ExecutionError {
            message: format!("invalid number literal '{text}'"),
            span: None,
        })
    }
}

fn sql_to_string(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => String::new(),
        SqlValue::Bool(b) => b.to_string(),
        SqlValue::Int(i) => i.to_string(),
        SqlValue::Float(f) => {
            let mut s = f.to_string();
            if s.ends_with(".0") {
                s.truncate(s.len() - 2);
            }
            s
        }
        SqlValue::Text(s) => s.clone(),
        SqlValue::Bytes(b) => String::from_utf8_lossy(b).to_string(),
    }
}

fn sql_as_f64(value: &SqlValue) -> Option<f64> {
    match value {
        SqlValue::Int(i) => Some(*i as f64),
        SqlValue::Float(f) => Some(*f),
        SqlValue::Text(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn sql_as_f64_ref(value: &SqlValue) -> Option<f64> {
    sql_as_f64(value)
}

fn is_truthy(value: &SqlValue) -> bool {
    match value {
        SqlValue::Null => false,
        SqlValue::Bool(b) => *b,
        SqlValue::Int(i) => *i != 0,
        SqlValue::Float(f) => *f != 0.0,
        SqlValue::Text(s) => !s.is_empty(),
        SqlValue::Bytes(b) => !b.is_empty(),
    }
}

fn sql_equals(left: &SqlValue, right: &SqlValue) -> bool {
    match (left, right) {
        (SqlValue::Null, SqlValue::Null) => true,
        (SqlValue::Bool(a), SqlValue::Bool(b)) => a == b,
        _ => {
            if let (Some(a), Some(b)) = (sql_as_f64(left), sql_as_f64(right)) {
                (a - b).abs() < f64::EPSILON
            } else {
                sql_to_string(left) == sql_to_string(right)
            }
        }
    }
}

fn infer_query_type(value: &SqlValue) -> QueryColumnType {
    match value {
        SqlValue::Null => QueryColumnType::Null,
        SqlValue::Bool(_) => QueryColumnType::Boolean,
        SqlValue::Int(_) => QueryColumnType::Integer,
        SqlValue::Float(_) => QueryColumnType::Double,
        SqlValue::Text(_) => QueryColumnType::Varchar,
        SqlValue::Bytes(_) => QueryColumnType::Blob,
    }
}

fn projection_name(expr: &Expression, idx: usize) -> String {
    match expr {
        Expression::Identifier(parts) => parts.last().cloned().unwrap_or_else(|| format!("column{}", idx + 1)),
        Expression::FunctionCall { name, .. } => name.last().cloned().unwrap_or_else(|| format!("expr{}", idx + 1)),
        _ => format!("expr{}", idx + 1),
    }
}

fn contains_aggregate(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionCall { name, .. } => {
            matches!(name.last().map(|s| s.to_lowercase()).as_deref(), Some("count" | "sum" | "min" | "max" | "avg"))
        }
        Expression::Binary { left, right, .. } => contains_aggregate(left) || contains_aggregate(right),
        Expression::Unary { expr, .. } | Expression::Paren(expr) | Expression::IsNull { expr, .. } => contains_aggregate(expr),
        Expression::Case { branches, else_expr } => {
            branches.iter().any(|(c, v)| contains_aggregate(c) || contains_aggregate(v))
                || else_expr.as_deref().is_some_and(contains_aggregate)
        }
        _ => false,
    }
}

fn aggregate_numeric(
    group_rows: &[&ResolvedRow],
    op: impl Fn(f64, f64) -> f64,
    init: f64,
    expr: &Expression,
    executor: &QueryExecutor<'_>,
    current: &ResolvedRow,
) -> Result<SqlValue, ExecutionError> {
    let mut acc = init;
    let mut first = true;
    for row in group_rows {
        if let Some(v) = sql_as_f64(&executor.eval(expr, row, None)?) {
            acc = if first { v } else { op(acc, v) };
            first = false;
        }
    }
    if first {
        Ok(SqlValue::Null)
    } else {
        let _ = current;
        Ok(SqlValue::Float(acc))
    }
}

fn aggregate_minmax(
    group_rows: &[&ResolvedRow],
    min: bool,
    expr: &Expression,
    executor: &QueryExecutor<'_>,
    current: &ResolvedRow,
) -> Result<SqlValue, ExecutionError> {
    let mut best: Option<SqlValue> = None;
    for row in group_rows {
        let value = executor.eval(expr, row, None)?;
        best = match best {
            None => Some(value),
            Some(prev) => {
                if let Some(ord) = compare_sql_values(&value, &prev) {
                    if (min && ord == Ordering::Less) || (!min && ord == Ordering::Greater) {
                        Some(value)
                    } else {
                        Some(prev)
                    }
                } else {
                    Some(prev)
                }
            }
        };
    }
    let _ = current;
    Ok(best.unwrap_or(SqlValue::Null))
}

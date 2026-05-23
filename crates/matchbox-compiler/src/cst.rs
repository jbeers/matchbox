use crate::tokenizer::{lex, lex_template, Span, SyntaxToken, TokenKind, Trivia};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
    Statement,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxElement {
    Node(Box<SyntaxNode>),
    Token(SyntaxToken),
    Trivia(Trivia),
    Source(Span),
}

impl SyntaxElement {
    pub fn span(&self) -> Span {
        match self {
            SyntaxElement::Node(node) => node.span,
            SyntaxElement::Token(token) => token.span,
            SyntaxElement::Trivia(trivia) => trivia.span,
            SyntaxElement::Source(span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Span,
    pub children: Vec<SyntaxElement>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTree<'a> {
    source: &'a str,
    root: SyntaxNode,
    elements: Vec<SyntaxElement>,
    tokens: Vec<SyntaxToken>,
    trivia: Vec<Trivia>,
}

pub use crate::tokenizer::TriviaKind;

pub fn parse_script(source: &str) -> SyntaxTree<'_> {
    let lexed = lex(source);
    let (source, tokens, trivia) = lexed.into_parts();
    let elements = merge_elements(&tokens, &trivia);
    let structured_elements = group_blocks(source, &elements);
    let root = SyntaxNode {
        kind: SyntaxKind::Root,
        span: Span {
            start: 0,
            end: source.len(),
            line: 1,
            col: 1,
        },
        children: group_top_level_statements(source, &structured_elements),
    };

    SyntaxTree {
        source,
        root,
        elements,
        tokens,
        trivia,
    }
}

pub fn parse_template(source: &str) -> SyntaxTree<'_> {
    let lexed = lex_template(source);
    let (source, tokens, trivia) = lexed.into_parts();
    let elements = merge_elements(&tokens, &trivia);
    let root = SyntaxNode {
        kind: SyntaxKind::Root,
        span: Span {
            start: 0,
            end: source.len(),
            line: 1,
            col: 1,
        },
        children: make_lossless_elements(source, &elements),
    };

    SyntaxTree {
        source,
        root,
        elements,
        tokens,
        trivia,
    }
}

impl<'a> SyntaxTree<'a> {
    pub fn root(&self) -> &SyntaxNode {
        &self.root
    }

    pub fn tokens(&self) -> impl Iterator<Item = &SyntaxToken> {
        self.tokens.iter()
    }

    pub fn trivia(&self) -> impl Iterator<Item = &Trivia> {
        self.trivia.iter()
    }

    pub fn elements(&self) -> impl Iterator<Item = &SyntaxElement> {
        self.elements.iter()
    }

    pub fn text(&self, span: Span) -> &'a str {
        &self.source[span.start..span.end]
    }

    pub fn to_source(&self) -> String {
        let mut source = String::with_capacity(self.source.len());
        append_node_source(&mut source, self, &self.root);
        source
    }
}

fn append_node_source(output: &mut String, tree: &SyntaxTree<'_>, node: &SyntaxNode) {
    for child in &node.children {
        match child {
            SyntaxElement::Node(node) => append_node_source(output, tree, node),
            SyntaxElement::Token(_) | SyntaxElement::Trivia(_) | SyntaxElement::Source(_) => {
                output.push_str(tree.text(child.span()));
            }
        }
    }
}

fn make_lossless_elements(source: &str, elements: &[SyntaxElement]) -> Vec<SyntaxElement> {
    let mut children = Vec::with_capacity(elements.len());
    let mut cursor = 0;

    for element in elements {
        let span = element.span();
        if cursor < span.start {
            children.push(SyntaxElement::Source(span_from_offsets(
                source, cursor, span.start,
            )));
        }
        children.push(element.clone());
        cursor = span.end;
    }

    if cursor < source.len() {
        children.push(SyntaxElement::Source(span_from_offsets(
            source,
            cursor,
            source.len(),
        )));
    }

    children
}

fn merge_elements(tokens: &[SyntaxToken], trivia: &[Trivia]) -> Vec<SyntaxElement> {
    let mut children = Vec::with_capacity(tokens.len() + trivia.len());
    let mut token_index = 0;
    let mut trivia_index = 0;

    while token_index < tokens.len() || trivia_index < trivia.len() {
        let next_token = tokens.get(token_index);
        let next_trivia = trivia.get(trivia_index);

        match (next_token, next_trivia) {
            (Some(token), Some(trivia)) if token.span.start <= trivia.span.start => {
                children.push(SyntaxElement::Token(*token));
                token_index += 1;
            }
            (Some(_), Some(trivia)) => {
                children.push(SyntaxElement::Trivia(*trivia));
                trivia_index += 1;
            }
            (Some(token), None) => {
                children.push(SyntaxElement::Token(*token));
                token_index += 1;
            }
            (None, Some(trivia)) => {
                children.push(SyntaxElement::Trivia(*trivia));
                trivia_index += 1;
            }
            (None, None) => break,
        }
    }

    children
}

fn group_blocks(source: &str, elements: &[SyntaxElement]) -> Vec<SyntaxElement> {
    let mut index = 0;
    group_blocks_until(source, elements, &mut index, false)
}

fn group_blocks_until(
    source: &str,
    elements: &[SyntaxElement],
    index: &mut usize,
    stop_at_right_brace: bool,
) -> Vec<SyntaxElement> {
    let mut children = Vec::new();

    while *index < elements.len() {
        match &elements[*index] {
            SyntaxElement::Token(token)
                if token.kind == TokenKind::RightBrace && stop_at_right_brace =>
            {
                break;
            }
            SyntaxElement::Token(token) if token.kind == TokenKind::LeftBrace => {
                let left_brace = elements[*index].clone();
                let start_span = token.span;
                *index += 1;

                let inner = group_blocks_until(source, elements, index, true);
                let inner_children = group_top_level_statements(source, &inner);

                let right_brace = if *index < elements.len() {
                    match &elements[*index] {
                        SyntaxElement::Token(token) if token.kind == TokenKind::RightBrace => {
                            let element = elements[*index].clone();
                            *index += 1;
                            Some(element)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                let mut block_children = Vec::with_capacity(inner_children.len() + 2);
                block_children.push(left_brace);
                block_children.extend(inner_children);
                if let Some(right_brace) = right_brace {
                    block_children.push(right_brace);
                }

                let mut span = span_for_elements(&block_children);
                span.start = start_span.start;
                span.line = start_span.line;
                span.col = start_span.col;
                children.push(SyntaxElement::Node(Box::new(SyntaxNode {
                    kind: SyntaxKind::Block,
                    span,
                    children: block_children,
                })));
            }
            _ => {
                children.push(elements[*index].clone());
                *index += 1;
            }
        }
    }

    children
}

fn group_top_level_statements(source: &str, elements: &[SyntaxElement]) -> Vec<SyntaxElement> {
    let mut children = Vec::new();
    let mut current = Vec::new();
    let mut has_token = false;
    let mut depth = 0i32;
    let mut finish_after_trivia = false;

    for element in elements {
        if finish_after_trivia {
            match element {
                SyntaxElement::Trivia(trivia) => {
                    if let Some((before, after)) = split_trivia_at_first_newline(source, *trivia) {
                        current.push(SyntaxElement::Trivia(before));
                        push_current_group(&mut children, &mut current, has_token);
                        if let Some(after) = after {
                            current.push(SyntaxElement::Trivia(after));
                        }
                        has_token = false;
                        finish_after_trivia = false;
                    } else {
                        current.push(element.clone());
                    }
                    continue;
                }
                SyntaxElement::Token(_) => {
                    push_current_group(&mut children, &mut current, has_token);
                    has_token = false;
                    finish_after_trivia = false;
                }
                SyntaxElement::Node(_) => {}
                SyntaxElement::Source(_) => {}
            }
        }

        current.push(element.clone());

        match element {
            SyntaxElement::Token(token) => {
                has_token = true;
                match token.kind {
                    TokenKind::LeftParen | TokenKind::LeftBracket => {
                        depth += 1;
                    }
                    TokenKind::RightParen | TokenKind::RightBracket => {
                        depth = (depth - 1).max(0);
                    }
                    TokenKind::Semicolon if depth == 0 => {
                        finish_after_trivia = true;
                    }
                    _ => {}
                }
            }
            SyntaxElement::Node(node)
                if node.kind == SyntaxKind::Block
                    && depth == 0
                    && current_begins_with_block_statement(&current) =>
            {
                has_token = true;
                finish_after_trivia = true;
            }
            SyntaxElement::Node(_) => {
                has_token = true;
            }
            SyntaxElement::Trivia(_) | SyntaxElement::Source(_) => {}
        }
    }

    if !current.is_empty() {
        push_current_group(&mut children, &mut current, has_token);
    }

    children
}

fn split_trivia_at_first_newline(source: &str, trivia: Trivia) -> Option<(Trivia, Option<Trivia>)> {
    let text = &source[trivia.span.start..trivia.span.end];
    let newline_index = text.find('\n')?;
    let split_at = trivia.span.start + newline_index + 1;

    let before = Trivia {
        kind: trivia.kind,
        span: Span {
            start: trivia.span.start,
            end: split_at,
            line: trivia.span.line,
            col: trivia.span.col,
        },
    };

    let after = (split_at < trivia.span.end).then_some(Trivia {
        kind: trivia.kind,
        span: Span {
            start: split_at,
            end: trivia.span.end,
            line: trivia.span.line + 1,
            col: 1,
        },
    });

    Some((before, after))
}

fn current_begins_with_block_statement(elements: &[SyntaxElement]) -> bool {
    for element in elements {
        match element {
            SyntaxElement::Token(token) => {
                return matches!(
                    token.kind,
                    TokenKind::Class
                        | TokenKind::Do
                        | TokenKind::For
                        | TokenKind::Function
                        | TokenKind::If
                        | TokenKind::Interface
                        | TokenKind::Switch
                        | TokenKind::Try
                        | TokenKind::While
                );
            }
            SyntaxElement::Node(node) => {
                if node.kind == SyntaxKind::Block {
                    return false;
                }
                if current_begins_with_block_statement(&node.children) {
                    return true;
                }
            }
            SyntaxElement::Trivia(_) | SyntaxElement::Source(_) => {}
        }
    }

    false
}

fn push_current_group(
    children: &mut Vec<SyntaxElement>,
    current: &mut Vec<SyntaxElement>,
    has_token: bool,
) {
    if has_token {
        let span = span_for_elements(current);
        let node = SyntaxNode {
            kind: SyntaxKind::Statement,
            span,
            children: std::mem::take(current),
        };
        children.push(SyntaxElement::Node(Box::new(node)));
    } else {
        children.append(current);
    }
}

fn span_for_elements(elements: &[SyntaxElement]) -> Span {
    let first = elements.first().map(SyntaxElement::span).unwrap_or(Span {
        start: 0,
        end: 0,
        line: 1,
        col: 1,
    });
    let last = elements.last().map(SyntaxElement::span).unwrap_or(first);
    Span {
        start: first.start,
        end: last.end,
        line: first.line,
        col: first.col,
    }
}

fn span_from_offsets(source: &str, start: usize, end: usize) -> Span {
    let mut line = 1;
    let mut col = 1;

    for ch in source[..start].chars() {
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Span {
        start,
        end,
        line,
        col,
    }
}

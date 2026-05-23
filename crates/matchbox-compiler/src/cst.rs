use crate::tokenizer::{lex, lex_template, Span, SyntaxToken, TokenKind, Trivia};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
    Statement,
    VariableDecl,
    Return,
    Throw,
    Continue,
    Break,
    Rethrow,
    Assert,
    Param,
    Include,
    Not,
    If,
    For,
    While,
    Do,
    Try,
    Switch,
    FunctionDecl,
    ClassDecl,
    InterfaceDecl,
    Block,
    Interpolation,
    ScriptIsland,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SyntaxNodeId(pub usize);

pub struct SyntaxDescendants<'a> {
    stack: Vec<&'a SyntaxNode>,
}

impl<'a> Iterator for SyntaxDescendants<'a> {
    type Item = &'a SyntaxNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        for child in node.children.iter().rev() {
            if let SyntaxElement::Node(child) = child {
                self.stack.push(child);
            }
        }
        Some(node)
    }
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
    pub id: SyntaxNodeId,
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
    let mut root = SyntaxNode {
        id: SyntaxNodeId::default(),
        kind: SyntaxKind::Root,
        span: Span {
            start: 0,
            end: source.len(),
            line: 1,
            col: 1,
        },
        children: group_top_level_statements(source, &structured_elements),
    };
    assign_node_ids(&mut root, 0);

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
    let lossless_elements = make_lossless_elements(source, &elements);
    let structured_elements = group_template_regions(source, &lossless_elements);
    let mut root = SyntaxNode {
        id: SyntaxNodeId::default(),
        kind: SyntaxKind::Root,
        span: Span {
            start: 0,
            end: source.len(),
            line: 1,
            col: 1,
        },
        children: structured_elements,
    };
    assign_node_ids(&mut root, 0);

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

    pub fn descendants(&self) -> SyntaxDescendants<'_> {
        self.root.descendants()
    }

    pub fn node(&self, id: SyntaxNodeId) -> Option<&SyntaxNode> {
        self.root.find_node(id)
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

fn assign_node_ids(node: &mut SyntaxNode, next_id: usize) -> usize {
    node.id = SyntaxNodeId(next_id);
    let mut next = next_id + 1;
    for child in &mut node.children {
        if let SyntaxElement::Node(child_node) = child {
            next = assign_node_ids(child_node, next);
        }
    }
    next
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

fn group_template_regions(source: &str, elements: &[SyntaxElement]) -> Vec<SyntaxElement> {
    let mut children = Vec::new();
    let mut index = 0;

    while index < elements.len() {
        match &elements[index] {
            SyntaxElement::Token(token) if token.kind == TokenKind::InterpStart => {
                let start_index = index;
                index += 1;
                let mut found = false;
                while index < elements.len() {
                    if matches!(
                        &elements[index],
                        SyntaxElement::Token(token) if token.kind == TokenKind::InterpEnd
                    ) {
                        let end_index = index;
                        let inner_start = token.span.end;
                        let inner_end = match &elements[end_index] {
                            SyntaxElement::Token(end) => end.span.start,
                            _ => unreachable!(),
                        };
                        let mut inner = parse_script(&source[inner_start..inner_end]).root.clone();
                        shift_node(&mut inner, inner_start, source);
                        let node = SyntaxNode {
                            id: SyntaxNodeId::default(),
                            kind: SyntaxKind::Interpolation,
                            span: span_for_elements(&elements[start_index..=end_index]),
                            children: vec![
                                elements[start_index].clone(),
                                SyntaxElement::Node(Box::new(inner)),
                                elements[end_index].clone(),
                            ],
                        };
                        children.push(SyntaxElement::Node(Box::new(node)));
                        index = end_index + 1;
                        found = true;
                        break;
                    }
                    index += 1;
                }
                if !found {
                    children.extend(elements[start_index..index].iter().cloned());
                }
            }
            SyntaxElement::Token(token) if token.kind == TokenKind::ScriptStart => {
                let start_index = index;
                let open_start = token.span.end;
                index += 1;
                let mut found = false;
                while index < elements.len() {
                    if matches!(
                        &elements[index],
                        SyntaxElement::Token(token) if token.kind == TokenKind::ScriptEnd
                    ) {
                        let end_index = index;
                        let script_end = match &elements[end_index] {
                            SyntaxElement::Token(end) => end.span.start,
                            _ => unreachable!(),
                        };
                        let body_start = elements
                            .get(start_index + 1)
                            .map(SyntaxElement::span)
                            .map(|span| if matches!(elements[start_index + 1], SyntaxElement::Source(_)) {
                                span.end
                            } else {
                                span.start
                            })
                            .unwrap_or(open_start);
                        let mut body = parse_script(&source[body_start..script_end]).root.clone();
                        shift_node(&mut body, body_start, source);
                        let opening_span = Span {
                            start: open_start,
                            end: body_start,
                            line: token.span.line,
                            col: token.span.col + 1,
                        };
                        let node = SyntaxNode {
                            id: SyntaxNodeId::default(),
                            kind: SyntaxKind::ScriptIsland,
                            span: span_for_elements(&elements[start_index..=end_index]),
                            children: vec![
                                elements[start_index].clone(),
                                SyntaxElement::Source(opening_span),
                                SyntaxElement::Node(Box::new(body)),
                                elements[end_index].clone(),
                            ],
                        };
                        children.push(SyntaxElement::Node(Box::new(node)));
                        index = end_index + 1;
                        found = true;
                        break;
                    }
                    index += 1;
                }
                if !found {
                    children.extend(elements[start_index..index].iter().cloned());
                }
            }
            _ => {
                children.push(elements[index].clone());
                index += 1;
            }
        }
    }

    children
}

fn shift_node(node: &mut SyntaxNode, base_start: usize, source: &str) {
    let base_span = span_from_offsets(source, base_start, base_start);
    shift_node_with_base(node, base_start, base_span.line, base_span.col);
}

fn shift_node_with_base(node: &mut SyntaxNode, base_start: usize, base_line: u32, base_col: u32) {
    node.span = shift_span(node.span, base_start, base_line, base_col);
    for child in &mut node.children {
        match child {
            SyntaxElement::Node(node) => shift_node_with_base(node, base_start, base_line, base_col),
            SyntaxElement::Token(token) => {
                token.span = shift_span(token.span, base_start, base_line, base_col);
            }
            SyntaxElement::Trivia(trivia) => {
                trivia.span = shift_span(trivia.span, base_start, base_line, base_col);
            }
            SyntaxElement::Source(span) => {
                *span = shift_span(*span, base_start, base_line, base_col);
            }
        }
    }
}

fn shift_span(span: Span, base_start: usize, base_line: u32, base_col: u32) -> Span {
    let start = base_start + span.start;
    let end = base_start + span.end;
    let line = if span.line == 1 {
        base_line
    } else {
        base_line + span.line - 1
    };
    let col = if span.line == 1 {
        base_col + span.col - 1
    } else {
        span.col
    };

    Span {
        start,
        end,
        line,
        col,
    }
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
            SyntaxElement::Token(token) if token.kind == TokenKind::RightBrace => {
                let span = token.span;
                children.push(SyntaxElement::Node(Box::new(SyntaxNode {
                    id: SyntaxNodeId::default(),
                    kind: SyntaxKind::Error,
                    span,
                    children: vec![elements[*index].clone()],
                })));
                *index += 1;
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

                let has_right_brace = right_brace.is_some();
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
                let kind = if has_right_brace {
                    SyntaxKind::Block
                } else {
                    SyntaxKind::Error
                };
                children.push(SyntaxElement::Node(Box::new(SyntaxNode {
                    id: SyntaxNodeId::default(),
                    kind,
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
            id: SyntaxNodeId::default(),
            kind: statement_kind_for_elements(current),
            span,
            children: std::mem::take(current),
        };
        children.push(SyntaxElement::Node(Box::new(node)));
    } else {
        children.append(current);
    }
}

fn statement_kind_for_elements(elements: &[SyntaxElement]) -> SyntaxKind {
    for element in elements {
        match element {
            SyntaxElement::Token(token) => {
                return match token.kind {
                    TokenKind::Var => SyntaxKind::VariableDecl,
                    TokenKind::Return => SyntaxKind::Return,
                    TokenKind::Throw => SyntaxKind::Throw,
                    TokenKind::Continue => SyntaxKind::Continue,
                    TokenKind::Break => SyntaxKind::Break,
                    TokenKind::Rethrow => SyntaxKind::Rethrow,
                    TokenKind::Assert => SyntaxKind::Assert,
                    TokenKind::Param => SyntaxKind::Param,
                    TokenKind::Include => SyntaxKind::Include,
                    TokenKind::Not => SyntaxKind::Not,
                    TokenKind::If => SyntaxKind::If,
                    TokenKind::For => SyntaxKind::For,
                    TokenKind::While => SyntaxKind::While,
                    TokenKind::Do => SyntaxKind::Do,
                    TokenKind::Try => SyntaxKind::Try,
                    TokenKind::Switch => SyntaxKind::Switch,
                    TokenKind::Function => SyntaxKind::FunctionDecl,
                    TokenKind::Class => SyntaxKind::ClassDecl,
                    TokenKind::Interface => SyntaxKind::InterfaceDecl,
                    _ => SyntaxKind::Statement,
                };
            }
            SyntaxElement::Node(node) => {
                let kind = node.kind;
                if kind != SyntaxKind::Block && kind != SyntaxKind::Error {
                    return kind;
                }
            }
            SyntaxElement::Trivia(_) | SyntaxElement::Source(_) => {}
        }
    }

    SyntaxKind::Statement
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

impl SyntaxNode {
    pub fn children_nodes(&self) -> impl Iterator<Item = &SyntaxNode> {
        self.children.iter().filter_map(|child| match child {
            SyntaxElement::Node(node) => Some(node.as_ref()),
            _ => None,
        })
    }

    pub fn descendants(&self) -> SyntaxDescendants<'_> {
        SyntaxDescendants { stack: vec![self] }
    }

    pub fn find_node(&self, id: SyntaxNodeId) -> Option<&SyntaxNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let SyntaxElement::Node(node) = child {
                if let Some(found) = node.find_node(id) {
                    return Some(found);
                }
            }
        }
        None
    }
}

use matchbox_compiler::cst::{SyntaxElement, SyntaxKind, TriviaKind};
use matchbox_compiler::parser;
use matchbox_compiler::tokenizer::TokenKind;

#[test]
fn script_cst_preserves_tokens_trivia_and_source_text() {
    let source = "var x = 1; // keep me\nx + 2\n";

    let tree = matchbox_compiler::cst::parse_script(source);

    assert_eq!(tree.to_source(), source);
    assert_eq!(tree.root().span.start, 0);
    assert_eq!(tree.root().span.end, source.len());

    let token_text: Vec<(TokenKind, &str)> = tree
        .tokens()
        .map(|token| (token.kind, tree.text(token.span)))
        .collect();
    assert_eq!(
        token_text,
        vec![
            (TokenKind::Var, "var"),
            (TokenKind::Identifier, "x"),
            (TokenKind::Equal, "="),
            (TokenKind::Number, "1"),
            (TokenKind::Semicolon, ";"),
            (TokenKind::Identifier, "x"),
            (TokenKind::Plus, "+"),
            (TokenKind::Number, "2"),
        ]
    );

    let trivia_text: Vec<(TriviaKind, &str)> = tree
        .trivia()
        .map(|trivia| (trivia.kind, tree.text(trivia.span)))
        .collect();
    assert_eq!(
        trivia_text,
        vec![
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::LineComment, "// keep me"),
            (TriviaKind::Whitespace, "\n"),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, "\n"),
        ]
    );

    let flat_elements: Vec<&SyntaxElement> = tree.elements().collect();
    assert!(matches!(flat_elements[0], SyntaxElement::Token(_)));
    assert!(flat_elements
        .iter()
        .any(|element| matches!(element, SyntaxElement::Trivia(_))));
}

#[test]
fn script_cst_groups_top_level_statements_with_source_spans() {
    let source = "var x = 1;\nreturn x;";

    let tree = matchbox_compiler::cst::parse_script(source);
    let statements: Vec<_> = tree
        .root()
        .children
        .iter()
        .filter_map(|element| match element {
            SyntaxElement::Node(node) => Some(node.as_ref()),
            _ => None,
        })
        .collect();

    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].kind, SyntaxKind::Statement);
    assert_eq!(tree.text(statements[0].span), "var x = 1;\n");
    assert_eq!(statements[1].kind, SyntaxKind::Statement);
    assert_eq!(tree.text(statements[1].span), "return x;");
    assert_eq!(tree.to_source(), source);
}

#[test]
fn script_cst_keeps_braced_blocks_as_one_top_level_statement() {
    let source = "if (x) { var y = 1; }\nvar z = 2;";

    let tree = matchbox_compiler::cst::parse_script(source);
    let statement_text: Vec<&str> = tree
        .root()
        .children
        .iter()
        .filter_map(|element| match element {
            SyntaxElement::Node(node) => Some(tree.text(node.span)),
            _ => None,
        })
        .collect();

    assert_eq!(
        statement_text,
        vec!["if (x) { var y = 1; }\n", "var z = 2;"]
    );
    assert_eq!(tree.to_source(), source);
}

#[test]
fn script_cst_groups_nested_block_statements() {
    let source = "if (x) {\n  var y = 1;\n  if (y) { return y; }\n}\nvar z = 2;";

    let tree = matchbox_compiler::cst::parse_script(source);
    let top_statement = tree
        .root()
        .children
        .iter()
        .find_map(|element| match element {
            SyntaxElement::Node(node) => Some(node.as_ref()),
            _ => None,
        })
        .unwrap();

    let blocks: Vec<_> = collect_nodes(top_statement, SyntaxKind::Block);
    assert_eq!(blocks.len(), 2);
    assert_eq!(
        tree.text(blocks[0].span),
        "{\n  var y = 1;\n  if (y) { return y; }\n}"
    );
    assert_eq!(tree.text(blocks[1].span), "{ return y; }");

    let outer_block_statements: Vec<&str> = blocks[0]
        .children
        .iter()
        .filter_map(|element| match element {
            SyntaxElement::Node(node) if node.kind == SyntaxKind::Statement => {
                Some(tree.text(node.span))
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        outer_block_statements,
        vec!["\n  var y = 1;\n", "  if (y) { return y; }\n"]
    );
    assert_eq!(tree.to_source(), source);
}

#[test]
fn cst_keeps_comments_while_script_parser_ignores_trivia() {
    let source = "/* doc\ncomment */\nvar answer = 42;";

    let tree = matchbox_compiler::cst::parse_script(source);
    let trivia_text: Vec<(TriviaKind, &str)> = tree
        .trivia()
        .map(|trivia| (trivia.kind, tree.text(trivia.span)))
        .collect();
    assert_eq!(
        trivia_text,
        vec![
            (TriviaKind::BlockComment, "/* doc\ncomment */"),
            (TriviaKind::Whitespace, "\n"),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
            (TriviaKind::Whitespace, " "),
        ]
    );

    let ast = parser::parse(source, Some("tooling-test.bxs")).unwrap();
    assert_eq!(ast.len(), 1);
    assert!(matches!(
        &ast[0].kind,
        matchbox_compiler::ast::StatementKind::VariableDecl { name, .. } if name == "answer"
    ));
}

#[test]
fn template_cst_preserves_source_gaps_and_round_trips() {
    let source = "Hello <bx:output>#name#</bx:output>\n<bx:script>var x = 1;</bx:script>";

    let tree = matchbox_compiler::cst::parse_template(source);

    assert_eq!(tree.to_source(), source);
    assert_eq!(tree.root().span.start, 0);
    assert_eq!(tree.root().span.end, source.len());
    assert!(tree.root().children.iter().any(
        |element| matches!(element, SyntaxElement::Source(span) if tree.text(*span) == "<bx:")
    ));
    let token_text: Vec<(TokenKind, &str)> = tree
        .tokens()
        .map(|token| (token.kind, tree.text(token.span)))
        .collect();
    assert!(token_text.contains(&(TokenKind::ContentText, "Hello ")));
    assert!(token_text.contains(&(TokenKind::ComponentName, "output")));
    assert!(token_text.contains(&(TokenKind::Identifier, "x")));
}

fn collect_nodes(
    node: &matchbox_compiler::cst::SyntaxNode,
    kind: SyntaxKind,
) -> Vec<&matchbox_compiler::cst::SyntaxNode> {
    let mut found = Vec::new();
    for child in &node.children {
        if let SyntaxElement::Node(node) = child {
            if node.kind == kind {
                found.push(node.as_ref());
            }
            found.extend(collect_nodes(node, kind));
        }
    }
    found
}

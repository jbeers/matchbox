#![cfg(feature = "qoq")]

use matchbox_compiler::qoq::{TokenKind, TriviaKind, lex};

#[test]
fn lexes_case_insensitive_keywords_and_literals() {
    let source = "SeLeCt foo, 42 FROM bar WHERE created >= {ts '2024-01-02 03:04:05'} AND note = 'O''Brien' AND name = :name";
    let lexed = lex(source);
    let kinds: Vec<_> = lexed.tokens().iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Select,
            TokenKind::Identifier,
            TokenKind::Comma,
            TokenKind::Number,
            TokenKind::From,
            TokenKind::Identifier,
            TokenKind::Where,
            TokenKind::Identifier,
            TokenKind::GreaterEqual,
            TokenKind::OdbcDateTimeLiteral,
            TokenKind::And,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::String,
            TokenKind::And,
            TokenKind::Identifier,
            TokenKind::Equal,
            TokenKind::BindParameter,
            TokenKind::Eof,
        ]
    );

    assert_eq!(lexed.text(lexed.tokens()[0].span), "SeLeCt");
    assert_eq!(lexed.text(lexed.tokens()[3].span), "42");
    assert_eq!(
        lexed.text(lexed.tokens()[9].span),
        "{ts '2024-01-02 03:04:05'}"
    );
    assert_eq!(lexed.text(lexed.tokens()[13].span), "'O''Brien'");
    assert_eq!(lexed.text(lexed.tokens()[17].span), ":name");
}

#[test]
fn preserves_comments_as_trivia() {
    let source = "select a -- line\n/* block */ from q";
    let lexed = lex(source);

    assert!(
        lexed
            .trivia()
            .iter()
            .any(|t| t.kind == TriviaKind::LineComment)
    );
    assert!(
        lexed
            .trivia()
            .iter()
            .any(|t| t.kind == TriviaKind::BlockComment)
    );
}

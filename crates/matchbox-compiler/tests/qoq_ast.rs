#![cfg(feature = "qoq")]

use matchbox_compiler::qoq::{QueryKind, QueryNode, parse};

#[test]
fn ast_walk_exposes_tables_and_expressions() {
    let query = parse(
        "SELECT a, COUNT(*) FROM foo JOIN bar ON foo.id = bar.id WHERE a > 1 ORDER BY a LIMIT 5",
    )
    .unwrap();
    let mut tables = 0usize;
    let mut expressions = 0usize;
    let mut select_nodes = 0usize;

    query.walk(&mut |node| match node {
        QueryNode::TableRef(_) => tables += 1,
        QueryNode::Expression(_) => expressions += 1,
        QueryNode::SelectStatement(_) => select_nodes += 1,
        _ => {}
    });

    assert!(tables >= 2);
    assert!(expressions >= 5);
    assert_eq!(select_nodes, 1);
    assert_eq!(query.tables().len(), 2);
    assert!(!query.expressions().is_empty());
}

#[test]
fn root_span_covers_the_entire_query() {
    let source = "SELECT 1 FROM foo";
    let query = parse(source).unwrap();

    assert_eq!(query.span.start, 0);
    assert_eq!(query.span.end, source.len());
    match query.kind {
        QueryKind::Select(_) => {}
        other => panic!("expected select query, got {other:?}"),
    }
}

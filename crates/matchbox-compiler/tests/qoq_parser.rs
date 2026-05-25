#![cfg(feature = "qoq")]

use matchbox_compiler::qoq::{BinaryOp, Expression, QueryKind, TableSource, parse};

#[test]
fn parses_select_from_where_order_and_limit() {
    let query = parse("SELECT a, b FROM foo WHERE a > 1 ORDER BY b DESC LIMIT 10").unwrap();
    let select = match query.kind {
        QueryKind::Select(select) => select,
        other => panic!("expected select query, got {other:?}"),
    };

    assert_eq!(select.projection.len(), 2);
    assert_eq!(select.from.len(), 1);
    assert_eq!(select.order_by.len(), 1);
    assert_eq!(select.limit, Some(10));

    match &select.from[0].source {
        TableSource::Named(path) => assert_eq!(path, &vec!["foo".to_string()]),
        other => panic!("expected named table source, got {other:?}"),
    }

    match &select.where_clause {
        Some(Expression::Binary {
            op: BinaryOp::Gt, ..
        }) => {}
        other => panic!("expected greater-than where clause, got {other:?}"),
    }
}

#[test]
fn parses_union_subquery_case_and_function_calls() {
    let query = parse(
        "SELECT CASE WHEN x = 1 THEN y ELSE z END AS v, COUNT(*) FROM (SELECT * FROM source) q WHERE q.x IS NOT NULL GROUP BY q.x HAVING COUNT(*) > 0 ORDER BY v DESC LIMIT 5 UNION ALL SELECT foo FROM bar",
    )
    .unwrap();

    let (left, all, right) = match query.kind {
        QueryKind::Union { left, all, right } => (left, all, right),
        other => panic!("expected union query, got {other:?}"),
    };
    assert!(all);

    let left_select = match left.kind {
        QueryKind::Select(select) => select,
        other => panic!("expected left select, got {other:?}"),
    };
    assert_eq!(left_select.projection.len(), 2);
    assert_eq!(left_select.from.len(), 1);
    assert_eq!(left_select.group_by.len(), 1);
    assert_eq!(left_select.having.is_some(), true);

    match &left_select.from[0].source {
        TableSource::Subquery(_) => {}
        other => panic!("expected subquery table source, got {other:?}"),
    }

    let right_select = match right.kind {
        QueryKind::Select(select) => select,
        other => panic!("expected right select, got {other:?}"),
    };
    assert_eq!(right_select.projection.len(), 1);
    assert_eq!(right_select.from.len(), 1);
}

#[test]
fn parse_errors_include_source_location() {
    let err = parse("SELECT FROM").unwrap_err();
    assert!(err.span.line >= 1);
    assert!(err.message.contains("expected"));
}

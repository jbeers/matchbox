#![cfg(feature = "qoq")]

use std::collections::HashMap;

use matchbox_compiler::qoq::{execute, parse};
use matchbox_vm::datasource::traits::{QueryColumn, QueryColumnType, QueryResult, SqlValue};

fn table(columns: &[&str], rows: Vec<Vec<SqlValue>>) -> QueryResult {
    QueryResult {
        columns: columns
            .iter()
            .map(|name| QueryColumn {
                name: (*name).to_string(),
                col_type: QueryColumnType::Varchar,
            })
            .collect(),
        rows,
    }
}

#[test]
fn executes_basic_select_where_order_and_limit() {
    let query = parse("SELECT id, name FROM people WHERE age >= 30 ORDER BY name DESC LIMIT 1").unwrap();
    let mut sources = HashMap::new();
    sources.insert(
        "people".to_string(),
        table(
            &["id", "name", "age"],
            vec![
                vec![SqlValue::Int(1), SqlValue::Text("Ada".to_string()), SqlValue::Int(29)],
                vec![SqlValue::Int(2), SqlValue::Text("Bea".to_string()), SqlValue::Int(34)],
                vec![SqlValue::Int(3), SqlValue::Text("Cora".to_string()), SqlValue::Int(41)],
            ],
        ),
    );

    let result = execute(&query, &sources).unwrap();

    assert_eq!(result.columns.len(), 2);
    assert_eq!(result.columns[0].name, "id");
    assert_eq!(result.columns[1].name, "name");
    assert_eq!(result.rows.len(), 1);
    match &result.rows[0][0] {
        SqlValue::Int(v) => assert_eq!(*v, 3),
        other => panic!("expected int, got {other:?}"),
    }
    match &result.rows[0][1] {
        SqlValue::Text(v) => assert_eq!(v, "Cora"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn executes_join_group_by_having_and_order_by() {
    let query = parse(
        "SELECT d.name AS dept_name, COUNT(*) AS c FROM people p INNER JOIN depts d ON p.dept = d.id WHERE p.age >= 30 GROUP BY d.name HAVING COUNT(*) > 1 ORDER BY c DESC",
    )
    .unwrap();
    let mut sources = HashMap::new();
    sources.insert(
        "people".to_string(),
        table(
            &["id", "dept", "age"],
            vec![
                vec![SqlValue::Int(1), SqlValue::Int(10), SqlValue::Int(29)],
                vec![SqlValue::Int(2), SqlValue::Int(10), SqlValue::Int(34)],
                vec![SqlValue::Int(3), SqlValue::Int(20), SqlValue::Int(41)],
                vec![SqlValue::Int(4), SqlValue::Int(10), SqlValue::Int(51)],
            ],
        ),
    );
    sources.insert(
        "depts".to_string(),
        table(
            &["id", "name"],
            vec![
                vec![SqlValue::Int(10), SqlValue::Text("Engineering".to_string())],
                vec![SqlValue::Int(20), SqlValue::Text("Support".to_string())],
            ],
        ),
    );

    let result = execute(&query, &sources).unwrap();

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns[0].name, "dept_name");
    assert_eq!(result.columns[1].name, "c");
    match &result.rows[0][0] {
        SqlValue::Text(v) => assert_eq!(v, "Engineering"),
        other => panic!("expected text, got {other:?}"),
    }
    match &result.rows[0][1] {
        SqlValue::Int(v) => assert_eq!(*v, 2),
        other => panic!("expected count, got {other:?}"),
    }
}

#[test]
fn executes_union_subquery_and_distinct() {
    let query = parse(
        "SELECT DISTINCT name FROM (SELECT name FROM first UNION ALL SELECT name FROM second) q ORDER BY name",
    )
    .unwrap();
    let mut sources = HashMap::new();
    sources.insert(
        "first".to_string(),
        table(
            &["name"],
            vec![
                vec![SqlValue::Text("Ada".to_string())],
                vec![SqlValue::Text("Bea".to_string())],
            ],
        ),
    );
    sources.insert(
        "second".to_string(),
        table(
            &["name"],
            vec![
                vec![SqlValue::Text("Bea".to_string())],
                vec![SqlValue::Text("Cora".to_string())],
            ],
        ),
    );

    let result = execute(&query, &sources).unwrap();

    assert_eq!(result.rows.len(), 3);
    match &result.rows[0][0] {
        SqlValue::Text(v) => assert_eq!(v, "Ada"),
        other => panic!("expected text, got {other:?}"),
    }
    match &result.rows[1][0] {
        SqlValue::Text(v) => assert_eq!(v, "Bea"),
        other => panic!("expected text, got {other:?}"),
    }
    match &result.rows[2][0] {
        SqlValue::Text(v) => assert_eq!(v, "Cora"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn executes_simple_avg_fast_path() {
    let query = parse("SELECT AVG(value) AS avg_value FROM data").unwrap();
    let mut sources = HashMap::new();
    sources.insert(
        "data".to_string(),
        table(
            &["value"],
            vec![
                vec![SqlValue::Int(1)],
                vec![SqlValue::Int(2)],
                vec![SqlValue::Int(3)],
                vec![SqlValue::Int(4)],
            ],
        ),
    );

    let result = execute(&query, &sources).unwrap();

    assert_eq!(result.columns.len(), 1);
    assert_eq!(result.columns[0].name, "avg_value");
    match &result.rows[0][0] {
        SqlValue::Float(v) => assert_eq!(*v, 2.5),
        other => panic!("expected float, got {other:?}"),
    }
}

#![cfg(feature = "qoq")]

use std::collections::HashMap;

use matchbox_compiler::qoq::{
    QuerySource, SourceColumnUsage, execute, execute_with_source_resolver, parse,
    source_column_dependency_plan,
};
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

fn assert_sql_value(actual: &SqlValue, expected: &SqlValue, sql: &str) {
    match (actual, expected) {
        (SqlValue::Null, SqlValue::Null) => {}
        (SqlValue::Bool(actual), SqlValue::Bool(expected)) => assert_eq!(actual, expected, "{sql}"),
        (SqlValue::Int(actual), SqlValue::Int(expected)) => assert_eq!(actual, expected, "{sql}"),
        (SqlValue::Float(actual), SqlValue::Float(expected)) => {
            assert_eq!(actual, expected, "{sql}")
        }
        (SqlValue::Text(actual), SqlValue::Text(expected)) => assert_eq!(actual, expected, "{sql}"),
        (SqlValue::Bytes(actual), SqlValue::Bytes(expected)) => {
            assert_eq!(actual, expected, "{sql}")
        }
        _ => panic!("expected {expected:?}, got {actual:?} for SQL: {sql}"),
    }
}

#[derive(Clone, Debug)]
struct TestSource {
    columns: Vec<QueryColumn>,
    rows: Vec<Vec<SqlValue>>,
    forbidden_cols: Vec<usize>,
}

impl TestSource {
    fn new(columns: &[&str], rows: Vec<Vec<SqlValue>>) -> Self {
        Self {
            columns: columns
                .iter()
                .map(|name| QueryColumn {
                    name: (*name).to_string(),
                    col_type: QueryColumnType::Varchar,
                })
                .collect(),
            rows,
            forbidden_cols: Vec::new(),
        }
    }

    fn forbidding_col(mut self, col_idx: usize) -> Self {
        self.forbidden_cols.push(col_idx);
        self
    }
}

impl QuerySource for TestSource {
    fn columns(&self) -> &[QueryColumn] {
        &self.columns
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn value(&self, row_idx: usize, col_idx: usize) -> SqlValue {
        if self.forbidden_cols.contains(&col_idx) {
            panic!("QoQ read unneeded source column {col_idx}");
        }
        self.rows
            .get(row_idx)
            .and_then(|row| row.get(col_idx))
            .cloned()
            .unwrap_or(SqlValue::Null)
    }
}

#[test]
fn source_column_dependency_plan_records_required_columns_by_usage() {
    let query = parse(
        "SELECT p.name, AVG(p.salary) AS avg_salary FROM people p WHERE p.age > 30 GROUP BY p.name HAVING AVG(p.salary) > 100 ORDER BY p.name",
    )
    .unwrap();

    let plan = source_column_dependency_plan(&query);
    let people = plan
        .sources
        .iter()
        .find(|source| source.alias.eq_ignore_ascii_case("p"))
        .expect("people source dependency");

    assert_eq!(people.source_path, vec!["people".to_string()]);
    assert!(!people.all_columns_required);

    let name = people
        .columns
        .iter()
        .find(|column| column.name.eq_ignore_ascii_case("name"))
        .expect("name dependency");
    assert!(name.usages.contains(&SourceColumnUsage::Projection));
    assert!(name.usages.contains(&SourceColumnUsage::GroupBy));
    assert!(name.usages.contains(&SourceColumnUsage::OrderBy));

    let salary = people
        .columns
        .iter()
        .find(|column| column.name.eq_ignore_ascii_case("salary"))
        .expect("salary dependency");
    assert!(salary.usages.contains(&SourceColumnUsage::Projection));
    assert!(salary.usages.contains(&SourceColumnUsage::Having));
    assert!(salary.usages.contains(&SourceColumnUsage::Aggregate));

    let age = people
        .columns
        .iter()
        .find(|column| column.name.eq_ignore_ascii_case("age"))
        .expect("age dependency");
    assert_eq!(age.usages, vec![SourceColumnUsage::Where]);
}

#[test]
fn executes_basic_select_where_order_and_limit() {
    let query =
        parse("SELECT id, name FROM people WHERE age >= 30 ORDER BY name DESC LIMIT 1").unwrap();
    let mut sources = HashMap::new();
    sources.insert(
        "people".to_string(),
        table(
            &["id", "name", "age"],
            vec![
                vec![
                    SqlValue::Int(1),
                    SqlValue::Text("Ada".to_string()),
                    SqlValue::Int(29),
                ],
                vec![
                    SqlValue::Int(2),
                    SqlValue::Text("Bea".to_string()),
                    SqlValue::Int(34),
                ],
                vec![
                    SqlValue::Int(3),
                    SqlValue::Text("Cora".to_string()),
                    SqlValue::Int(41),
                ],
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

#[test]
fn simple_aggregate_fast_path_reads_only_referenced_source_column() {
    let query = parse("SELECT AVG(value) AS avg_value FROM data").unwrap();
    let source = TestSource::new(
        &["value", "unused"],
        vec![
            vec![SqlValue::Int(1), SqlValue::Int(100)],
            vec![SqlValue::Int(2), SqlValue::Int(200)],
            vec![SqlValue::Int(3), SqlValue::Int(300)],
        ],
    )
    .forbidding_col(1);

    let result = execute_with_source_resolver(&query, move |path| {
        if path.len() == 1 && path[0].eq_ignore_ascii_case("data") {
            Ok(Some(Box::new(source.clone()) as Box<dyn QuerySource>))
        } else {
            Ok(None)
        }
    })
    .unwrap();

    match &result.rows[0][0] {
        SqlValue::Float(v) => assert_eq!(*v, 2.0),
        other => panic!("expected avg float, got {other:?}"),
    }
}

#[test]
fn simple_aggregate_fast_path_supports_sum_count_min_and_max() {
    let source = TestSource::new(
        &["value"],
        vec![
            vec![SqlValue::Int(4)],
            vec![SqlValue::Int(2)],
            vec![SqlValue::Int(9)],
        ],
    );

    let cases = [
        ("SELECT SUM(value) AS n FROM data", SqlValue::Float(15.0)),
        ("SELECT COUNT(value) AS n FROM data", SqlValue::Int(3)),
        ("SELECT COUNT(*) AS n FROM data", SqlValue::Int(3)),
        ("SELECT MIN(value) AS n FROM data", SqlValue::Int(2)),
        ("SELECT MAX(value) AS n FROM data", SqlValue::Int(9)),
    ];

    for (sql, expected) in cases {
        let query = parse(sql).unwrap();
        let source = source.clone();
        let result = execute_with_source_resolver(&query, move |path| {
            if path.len() == 1 && path[0].eq_ignore_ascii_case("data") {
                Ok(Some(Box::new(source.clone()) as Box<dyn QuerySource>))
            } else {
                Ok(None)
            }
        })
        .unwrap();
        assert_sql_value(&result.rows[0][0], &expected, sql);
    }
}

#[test]
fn generic_source_materialization_skips_unused_source_columns() {
    let query = parse("SELECT name FROM people WHERE age >= 30 ORDER BY name").unwrap();
    let source = TestSource::new(
        &["id", "name", "age", "unused"],
        vec![
            vec![
                SqlValue::Int(1),
                SqlValue::Text("Ada".to_string()),
                SqlValue::Int(29),
                SqlValue::Text("nope".to_string()),
            ],
            vec![
                SqlValue::Int(2),
                SqlValue::Text("Bea".to_string()),
                SqlValue::Int(34),
                SqlValue::Text("nope".to_string()),
            ],
        ],
    )
    .forbidding_col(0)
    .forbidding_col(3);

    let result = execute_with_source_resolver(&query, move |path| {
        if path.len() == 1 && path[0].eq_ignore_ascii_case("people") {
            Ok(Some(Box::new(source.clone()) as Box<dyn QuerySource>))
        } else {
            Ok(None)
        }
    })
    .unwrap();

    assert_eq!(result.rows.len(), 1);
    assert_sql_value(
        &result.rows[0][0],
        &SqlValue::Text("Bea".to_string()),
        "SELECT name FROM people WHERE age >= 30 ORDER BY name",
    );
}

#[test]
fn count_star_grouping_does_not_materialize_unused_source_columns() {
    let query = parse("SELECT dept, COUNT(*) AS c FROM people GROUP BY dept").unwrap();
    let source = TestSource::new(
        &["dept", "unused"],
        vec![
            vec![SqlValue::Text("eng".to_string()), SqlValue::Int(10)],
            vec![SqlValue::Text("eng".to_string()), SqlValue::Int(20)],
        ],
    )
    .forbidding_col(1);

    let result = execute_with_source_resolver(&query, move |path| {
        if path.len() == 1 && path[0].eq_ignore_ascii_case("people") {
            Ok(Some(Box::new(source.clone()) as Box<dyn QuerySource>))
        } else {
            Ok(None)
        }
    })
    .unwrap();

    assert_eq!(result.rows.len(), 1);
    assert_sql_value(&result.rows[0][0], &SqlValue::Text("eng".to_string()), "");
    assert_sql_value(&result.rows[0][1], &SqlValue::Int(2), "");
}

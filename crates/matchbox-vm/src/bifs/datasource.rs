#[cfg(feature = "bif-datasource")]
use crate::types::{BxVM, BxValue};

#[cfg(feature = "bif-datasource")]
use crate::datasource::traits::{
    DatasourceConfig, QueryColumn, QueryColumnType, QueryParam, SqlValue,
};
#[cfg(feature = "bif-datasource")]
use crate::datasource::{BxQuery, bx_to_sql, registry, sql_to_bx};
#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
use crate::qoq;
#[cfg(feature = "bif-datasource")]
use std::cell::RefCell;
#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
use std::collections::HashMap;
#[cfg(feature = "bif-datasource")]
use std::rc::Rc;
#[cfg(feature = "bif-datasource")]
use std::sync::Arc;
#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
use std::sync::{Mutex, OnceLock};

// ─── datasourceRegister ──────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn datasource_register(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("datasourceRegister() expects 2 arguments: (name, configStruct)".to_string());
    }
    let name = vm.to_string(args[0]);
    let cfg_id = args[1]
        .as_gc_id()
        .ok_or_else(|| "datasourceRegister() second argument must be a struct".to_string())?;

    let config = DatasourceConfig {
        driver: vm.to_string(vm.struct_get(cfg_id, "driver")),
        host: vm.to_string(vm.struct_get(cfg_id, "host")),
        port: {
            let v = vm.struct_get(cfg_id, "port");
            if v.is_number() {
                v.as_number() as u16
            } else {
                5432
            }
        },
        database: vm.to_string(vm.struct_get(cfg_id, "database")),
        username: vm.to_string(vm.struct_get(cfg_id, "username")),
        password: vm.to_string(vm.struct_get(cfg_id, "password")),
        max_connections: {
            let v = vm.struct_get(cfg_id, "maxConnections");
            if v.is_number() {
                v.as_number() as u32
            } else {
                10
            }
        },
    };

    let driver_name = config.driver.to_lowercase();

    use crate::datasource::drivers::postgres::PostgresDriver;
    match driver_name.as_str() {
        "postgresql" | "postgres" => {
            let driver = PostgresDriver::new(&config)
                .map_err(|e| format!("Failed to create PostgreSQL datasource '{}': {}", name, e))?;
            registry::register(&name, Arc::new(driver));
            return Ok(BxValue::new_bool(true));
        }
        other => {
            return Err(format!(
                "Unknown datasource driver: '{}'. Supported: postgresql",
                other
            ));
        }
    }
}

// ─── queryExecute ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_execute(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err(
            "queryExecute() expects at least 1 argument: (sql [, params [, options]])".to_string(),
        );
    }
    let sql = vm.to_string(args[0]);

    let params = if args.len() > 1 && !args[1].is_null() {
        parse_query_params(vm, args[1])?
    } else {
        vec![]
    };
    let (datasource_name, return_type, db_type) = if args.len() > 2 && !args[2].is_null() {
        if let Some(opts_id) = args[2].as_gc_id() {
            let ds = {
                let v = vm.struct_get(opts_id, "datasource");
                if v.is_null() {
                    "default".to_string()
                } else {
                    vm.to_string(v)
                }
            };
            let rt = {
                let v = vm.struct_get(opts_id, "returnType");
                if v.is_null() {
                    "query".to_string()
                } else {
                    vm.to_string(v).to_lowercase()
                }
            };
            let dbt = {
                let v = vm.struct_get(opts_id, "dbtype");
                if v.is_null() {
                    String::new()
                } else {
                    vm.to_string(v).to_lowercase()
                }
            };
            (ds, rt, dbt)
        } else {
            ("default".to_string(), "query".to_string(), String::new())
        }
    } else {
        ("default".to_string(), "query".to_string(), String::new())
    };

    if db_type == "query" {
        #[cfg(feature = "qoq")]
        {
            let qoq_params = if args.len() > 1 && !args[1].is_null() {
                parse_qoq_bind_params(vm, args[1])?
            } else {
                qoq::BindParams::default()
            };
            return query_execute_qoq(vm, &sql, &return_type, &qoq_params);
        }
        #[cfg(not(feature = "qoq"))]
        {
            return Err("QoQ support is not enabled in this build".to_string());
        }
    }

    let driver = registry::get(&datasource_name).ok_or_else(|| {
        format!(
            "Datasource '{}' not registered. Use datasourceRegister() first.",
            datasource_name
        )
    })?;

    let result = driver.execute(&sql, &params)?;
    let query = BxQuery::from_result(result);

    match return_type.as_str() {
        "array" => query_result_to_array(vm, query),
        "struct" => query_result_to_struct(vm, query),
        _ => {
            let id = vm.native_object_new(Rc::new(RefCell::new(query)));
            Ok(BxValue::new_ptr(id))
        }
    }
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
fn query_execute_qoq(
    vm: &mut dyn BxVM,
    sql: &str,
    return_type: &str,
    params: &qoq::BindParams,
) -> Result<BxValue, String> {
    let query = parse_cached_qoq(sql)?;
    let query =
        qoq::bind_params(&query, params).map_err(|e| qoq_error_to_string("QoQ bind error", e))?;
    let result = {
        let vm_ref: &dyn BxVM = &*vm;
        qoq::execute_with_source_resolver(&query, move |path| resolve_qoq_source(vm_ref, path))
            .map_err(|e| qoq_error_to_string("QoQ execution error", e))?
    };
    let query = BxQuery::from_result(result);

    match return_type {
        "array" => query_result_to_array(vm, query),
        "struct" => query_result_to_struct(vm, query),
        _ => {
            let id = vm.native_object_new(Rc::new(RefCell::new(query)));
            Ok(BxValue::new_ptr(id))
        }
    }
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
static QOQ_PARSE_CACHE: OnceLock<Mutex<HashMap<String, qoq::Query>>> = OnceLock::new();

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
const QOQ_PARSE_CACHE_LIMIT: usize = 128;

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
fn parse_cached_qoq(sql: &str) -> Result<qoq::Query, String> {
    let cache = QOQ_PARSE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Some(query) = cache
        .lock()
        .map_err(|_| "QoQ parse cache is poisoned".to_string())?
        .get(sql)
        .cloned()
    {
        return Ok(query);
    }

    let query = qoq::parse(sql).map_err(|e| format!("QoQ parse error: {}", e))?;
    let mut cache = cache
        .lock()
        .map_err(|_| "QoQ parse cache is poisoned".to_string())?;
    if cache.len() >= QOQ_PARSE_CACHE_LIMIT {
        cache.clear();
    }
    cache.insert(sql.to_string(), query.clone());
    Ok(query)
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
struct VmQuerySource<'a> {
    vm: &'a dyn BxVM,
    id: usize,
    columns: Vec<QueryColumn>,
    row_count: usize,
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
impl<'a> qoq::QuerySource for VmQuerySource<'a> {
    fn columns(&self) -> &[QueryColumn] {
        &self.columns
    }

    fn row_count(&self) -> usize {
        self.row_count
    }

    fn value(&self, row_idx: usize, col_idx: usize) -> SqlValue {
        self.vm
            .native_object_query_cell(self.id, row_idx, col_idx)
            .unwrap_or(SqlValue::Null)
    }
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
fn resolve_qoq_source<'a>(
    vm: &'a dyn BxVM,
    path: &[String],
) -> Result<Option<Box<dyn qoq::QuerySource + 'a>>, qoq::ExecutionError> {
    let key = path.join(".");
    let value = vm
        .resolve_query_source_path(path)
        .ok_or_else(|| qoq::ExecutionError {
            message: format!("unknown QoQ source '{}'", key),
            span: None,
        })?;
    let id = value.as_gc_id().ok_or_else(|| qoq::ExecutionError {
        message: format!("QoQ source '{}' is not a query object", key),
        span: None,
    })?;
    let columns = vm
        .native_object_query_columns(id)
        .ok_or_else(|| qoq::ExecutionError {
            message: format!("QoQ source '{}' is not a query object", key),
            span: None,
        })?;
    let row_count = vm
        .native_object_query_row_count(id)
        .ok_or_else(|| qoq::ExecutionError {
            message: format!("QoQ source '{}' is not a query object", key),
            span: None,
        })?;
    Ok(Some(Box::new(VmQuerySource {
        vm,
        id,
        columns,
        row_count,
    })))
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
fn qoq_error_to_string(prefix: &str, e: qoq::ExecutionError) -> String {
    if let Some(span) = e.span {
        format!(
            "{}: {} at line {}, col {}",
            prefix, e.message, span.line, span.col
        )
    } else {
        format!("{}: {}", prefix, e.message)
    }
}

#[cfg(all(
    feature = "bif-datasource",
    feature = "qoq",
    not(target_arch = "wasm32")
))]
fn parse_qoq_bind_params(vm: &mut dyn BxVM, val: BxValue) -> Result<qoq::BindParams, String> {
    if let Some(id) = val.as_gc_id() {
        if vm.struct_len(id) > 0 {
            let mut named = std::collections::HashMap::new();
            for key in vm.struct_key_array(id) {
                let item = vm.struct_get(id, &key);
                let sql_val = if let Some(item_id) = item.as_gc_id() {
                    if vm.struct_key_exists(item_id, "value") {
                        let v = vm.struct_get(item_id, "value");
                        let sql_type_str = {
                            let t = vm.struct_get(item_id, "cfsqltype");
                            if t.is_null() {
                                None
                            } else {
                                Some(vm.to_string(t))
                            }
                        };
                        coerce_cf_sql_type(vm, v, sql_type_str.as_deref())
                    } else {
                        bx_to_sql(vm, item)
                    }
                } else {
                    bx_to_sql(vm, item)
                };
                named.insert(key.to_lowercase(), sql_val);
            }
            Ok(qoq::BindParams {
                positional: vec![],
                named,
            })
        } else {
            let len = vm.array_len(id);
            let mut positional = Vec::with_capacity(len);
            for i in 0..len {
                let item = vm.array_get(id, i);
                let sql_val = if let Some(item_id) = item.as_gc_id() {
                    if vm.struct_key_exists(item_id, "value") {
                        let v = vm.struct_get(item_id, "value");
                        let sql_type_str = {
                            let t = vm.struct_get(item_id, "cfsqltype");
                            if t.is_null() {
                                None
                            } else {
                                Some(vm.to_string(t))
                            }
                        };
                        coerce_cf_sql_type(vm, v, sql_type_str.as_deref())
                    } else {
                        bx_to_sql(vm, item)
                    }
                } else {
                    bx_to_sql(vm, item)
                };
                positional.push(sql_val);
            }
            Ok(qoq::BindParams {
                positional,
                named: std::collections::HashMap::new(),
            })
        }
    } else {
        Ok(qoq::BindParams {
            positional: vec![bx_to_sql(vm, val)],
            named: std::collections::HashMap::new(),
        })
    }
}

#[cfg(all(
    feature = "bif-datasource",
    not(feature = "qoq"),
    not(target_arch = "wasm32")
))]
fn query_execute_qoq(
    _vm: &mut dyn BxVM,
    _sql: &str,
    _return_type: &str,
) -> Result<BxValue, String> {
    Err("QoQ support is not enabled in this build".to_string())
}

// ─── queryNew ────────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_new(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let column_arg = args
        .first()
        .ok_or_else(|| "queryNew() requires a column list".to_string())?;
    let col_names = query_list_arg(vm, *column_arg, "columnList")?;

    let col_types = if args.len() > 1 {
        query_list_arg(vm, args[1], "columnTypeList")?
    } else {
        vec![]
    };

    let columns: Vec<QueryColumn> = col_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let col_type = col_types
                .get(i)
                .map(|t| parse_col_type(t))
                .unwrap_or(QueryColumnType::Varchar);
            QueryColumn { name, col_type }
        })
        .collect();

    let mut query = BxQuery::new(columns);
    if args.len() > 2 && !args[2].is_null() {
        populate_query_rows(vm, &mut query, args[2])?;
    }

    let id = vm.native_object_new(Rc::new(RefCell::new(query)));
    Ok(BxValue::new_ptr(id))
}

#[cfg(feature = "bif-datasource")]
fn query_list_arg(vm: &dyn BxVM, value: BxValue, name: &str) -> Result<Vec<String>, String> {
    if value.is_null() {
        return Ok(vec![]);
    }

    if vm.is_array_value(value) {
        let arr_id = value
            .as_gc_id()
            .ok_or_else(|| format!("queryNew() {name} array is invalid"))?;
        return Ok((0..vm.array_len(arr_id))
            .map(|i| vm.to_string(vm.array_get(arr_id, i)).trim().to_string())
            .filter(|s| !s.is_empty())
            .collect());
    }

    if vm.is_string_value(value) {
        return Ok(vm
            .to_string(value)
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect());
    }

    Err(format!(
        "queryNew() {name} must be a comma-delimited string or array"
    ))
}

#[cfg(feature = "bif-datasource")]
fn populate_query_rows(
    vm: &mut dyn BxVM,
    query: &mut BxQuery,
    row_data: BxValue,
) -> Result<(), String> {
    let row_data_id = row_data
        .as_gc_id()
        .filter(|_| vm.is_array_value(row_data))
        .ok_or_else(|| "queryNew() rowData must be an array".to_string())?;

    for row_idx in 0..vm.array_len(row_data_id) {
        let row = vm.array_get(row_data_id, row_idx);
        if vm.is_struct_value(row) {
            add_query_struct_row(vm, query, row)?;
        } else if vm.is_array_value(row) {
            add_query_array_row(vm, query, row)?;
        } else if query.columns.len() == 1 {
            add_query_scalar_row(query, row, vm);
        } else {
            return Err("queryNew() rowData entries must be structs or arrays".to_string());
        }
    }
    Ok(())
}

#[cfg(feature = "bif-datasource")]
fn add_query_struct_row(
    vm: &mut dyn BxVM,
    query: &mut BxQuery,
    row: BxValue,
) -> Result<(), String> {
    let struct_id = row
        .as_gc_id()
        .ok_or_else(|| "queryNew() row struct is invalid".to_string())?;
    for (col_idx, col) in query.columns.iter().enumerate() {
        let bx_val = vm.struct_get(struct_id, &col.name);
        query.data[col_idx].push(bx_to_sql(vm, bx_val));
    }
    query.record_count += 1;
    Ok(())
}

#[cfg(feature = "bif-datasource")]
fn add_query_array_row(vm: &mut dyn BxVM, query: &mut BxQuery, row: BxValue) -> Result<(), String> {
    let row_id = row
        .as_gc_id()
        .ok_or_else(|| "queryNew() row array is invalid".to_string())?;
    for col_idx in 0..query.columns.len() {
        let bx_val = if col_idx < vm.array_len(row_id) {
            vm.array_get(row_id, col_idx)
        } else {
            BxValue::new_null()
        };
        query.data[col_idx].push(bx_to_sql(vm, bx_val));
    }
    query.record_count += 1;
    Ok(())
}

#[cfg(feature = "bif-datasource")]
fn add_query_scalar_row(query: &mut BxQuery, row: BxValue, vm: &dyn BxVM) {
    query.data[0].push(bx_to_sql(vm, row));
    query.record_count += 1;
}

// ─── queryAddRow ─────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_add_row(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryAddRow() expects 2 arguments: (query, dataStruct)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryAddRow() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "addrow", &args[1..])
}

// ─── queryColumnData ─────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_column_data(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryColumnData() expects 2 arguments: (query, columnName)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryColumnData() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "columndata", &args[1..])
}

// ─── queryColumnList ─────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_column_list(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryColumnList() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryColumnList() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "columnlist", &[])
}

// ─── queryClear ──────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_clear(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryClear() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryClear() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "clear", &[])
}

// ─── queryColumnArray ────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_column_array(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryColumnArray() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryColumnArray() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "columnarray", &[])
}

// ─── queryColumnCount ────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_column_count(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryColumnCount() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryColumnCount() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "columncount", &[])
}

// ─── queryColumnExists ───────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_column_exists(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryColumnExists() expects 2 arguments: (query, columnName)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryColumnExists() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "columnexists", &args[1..])
}

// ─── queryKeyExists ──────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_key_exists(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryKeyExists() expects 2 arguments: (query, key)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryKeyExists() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "keyexists", &args[1..])
}

// ─── queryRecordCount ────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_record_count(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryRecordCount() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryRecordCount() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "recordcount", &[])
}

// ─── queryRowData ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_row_data(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryRowData() expects 2 arguments: (query, rowNumber)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryRowData() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "getrow", &args[1..])
}

// ─── queryReverse ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_reverse(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryReverse() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryReverse() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "reverse", &[])
}

// ─── queryRowSwap ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_row_swap(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err("queryRowSwap() expects 3 arguments: (query, source, destination)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryRowSwap() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "rowswap", &args[1..])
}

// ─── querySlice ──────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_slice(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "querySlice() expects at least 2 arguments: (query, offset [, length])".to_string(),
        );
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "querySlice() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "slice", &args[1..])
}

// ─── queryDeleteColumn ───────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_delete_column(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryDeleteColumn() expects 2 arguments: (query, columnName)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryDeleteColumn() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "deletecolumn", &args[1..])
}

// ─── queryDeleteRow ──────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_delete_row(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryDeleteRow() expects 2 arguments: (query, row)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryDeleteRow() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "deleterow", &args[1..])
}

// ─── queryAddColumn ──────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_add_column(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryAddColumn() expects at least 2 arguments: (query, columnName [, datatype, arrayData])".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryAddColumn() first argument must be a query object".to_string())?;
    // Build args for native method: columnName, datatype, arrayData
    let col_name = args.get(1).copied().unwrap_or(BxValue::new_null());
    let datatype = args
        .get(2)
        .copied()
        .unwrap_or_else(|| BxValue::new_ptr(vm.string_new("object".to_string())));
    let array_data = args.get(3).copied().unwrap_or(BxValue::new_null());
    vm.native_object_call_method(query_id, "addcolumn", &[col_name, datatype, array_data])
}

// ─── queryAppend ─────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_append(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryAppend() expects 2 arguments: (query1, query2)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryAppend() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "append", &args[1..])
}

// ─── queryPrepend ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_prepend(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("queryPrepend() expects 2 arguments: (query1, query2)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryPrepend() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "prepend", &args[1..])
}

// ─── queryGetCell ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_get_cell(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "queryGetCell() expects at least 2 arguments: (query, columnName [, rowNumber])"
                .to_string(),
        );
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryGetCell() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "getcell", &args[1..])
}

// ─── querySetCell ────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_set_cell(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err(
            "querySetCell() expects at least 3 arguments: (query, columnName, value [, rowNumber])"
                .to_string(),
        );
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "querySetCell() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "setcell", &args[1..])
}

// ─── queryInsertAt ───────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_insert_at(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err("queryInsertAt() expects 3 arguments: (query, value, position)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryInsertAt() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "insertat", &args[1..])
}

// ─── querySetRow ─────────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_set_row(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err(
            "querySetRow() expects at least 2 arguments: (query, rowData [, rowNumber])"
                .to_string(),
        );
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "querySetRow() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "setrow", &args[1..])
}

// ─── queryCurrentRow ─────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_current_row(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryCurrentRow() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryCurrentRow() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "currentrow", &[])
}

// ─── queryGetResult ──────────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn query_get_result(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("queryGetResult() expects 1 argument: (query)".to_string());
    }
    let query_id = args[0]
        .as_gc_id()
        .ok_or_else(|| "queryGetResult() first argument must be a query object".to_string())?;
    vm.native_object_call_method(query_id, "getresult", &[])
}

// ─── Transaction stubs ───────────────────────────────────────────────────────
#[cfg(feature = "bif-datasource")]
pub fn transaction_begin(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("transactionBegin() is not yet implemented. Transaction support is planned for a future release.".to_string())
}
#[cfg(feature = "bif-datasource")]
pub fn transaction_commit(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("transactionCommit() is not yet implemented. Transaction support is planned for a future release.".to_string())
}
#[cfg(feature = "bif-datasource")]
pub fn transaction_rollback(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("transactionRollback() is not yet implemented. Transaction support is planned for a future release.".to_string())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Coerce a BxValue to the appropriate SqlValue based on a CF SQL type hint.
#[cfg(feature = "bif-datasource")]
fn coerce_cf_sql_type(vm: &mut dyn BxVM, val: BxValue, cf_type: Option<&str>) -> SqlValue {
    match cf_type.map(|s| s.to_uppercase()).as_deref() {
        Some("CF_SQL_BIT") => {
            let s = vm.to_string(val).to_lowercase();
            SqlValue::Bool(s == "true" || s == "1" || s == "yes")
        }
        Some("CF_SQL_INTEGER")
        | Some("CF_SQL_INT")
        | Some("CF_SQL_SMALLINT")
        | Some("CF_SQL_TINYINT")
        | Some("CF_SQL_BIGINT") => {
            let s = vm.to_string(val);
            SqlValue::Int(s.parse::<i64>().unwrap_or(0))
        }
        Some("CF_SQL_FLOAT")
        | Some("CF_SQL_DOUBLE")
        | Some("CF_SQL_DECIMAL")
        | Some("CF_SQL_NUMERIC")
        | Some("CF_SQL_REAL")
        | Some("CF_SQL_MONEY")
        | Some("CF_SQL_SMALLMONEY") => {
            let s = vm.to_string(val);
            SqlValue::Float(s.parse::<f64>().unwrap_or(0.0))
        }
        _ => bx_to_sql(vm, val),
    }
}

#[cfg(feature = "bif-datasource")]
fn parse_query_params(vm: &mut dyn BxVM, val: BxValue) -> Result<Vec<QueryParam>, String> {
    if let Some(arr_id) = val.as_gc_id() {
        let len = vm.array_len(arr_id);
        let mut params = Vec::with_capacity(len);
        for i in 0..len {
            let item = vm.array_get(arr_id, i);
            if let Some(item_id) = item.as_gc_id() {
                if vm.struct_key_exists(item_id, "value") {
                    // CF-style: {value: ..., cfsqltype: "CF_SQL_VARCHAR"}
                    let v = vm.struct_get(item_id, "value");
                    let sql_type_str = {
                        let t = vm.struct_get(item_id, "cfsqltype");
                        if t.is_null() {
                            None
                        } else {
                            Some(vm.to_string(t))
                        }
                    };
                    let sql_val = coerce_cf_sql_type(vm, v, sql_type_str.as_deref());
                    params.push(QueryParam {
                        value: sql_val,
                        sql_type: sql_type_str,
                    });
                } else {
                    // Plain GC value (string, array, etc.)
                    params.push(QueryParam {
                        value: bx_to_sql(vm, item),
                        sql_type: None,
                    });
                }
            } else {
                params.push(QueryParam {
                    value: bx_to_sql(vm, item),
                    sql_type: None,
                });
            }
        }
        Ok(params)
    } else {
        Ok(vec![QueryParam {
            value: bx_to_sql(vm, val),
            sql_type: None,
        }])
    }
}

#[cfg(feature = "bif-datasource")]
fn query_result_to_array(vm: &mut dyn BxVM, query: BxQuery) -> Result<BxValue, String> {
    let arr_id = vm.array_new();
    for row_idx in 0..query.record_count {
        let struct_id = vm.struct_new();
        for (col_idx, col) in query.columns.iter().enumerate() {
            let val = query
                .data
                .get(col_idx)
                .and_then(|col_data| col_data.get(row_idx))
                .cloned()
                .unwrap_or(SqlValue::Null);
            let bx = sql_to_bx(vm, &val);
            vm.struct_set(struct_id, &col.name, bx);
        }
        vm.array_push(arr_id, BxValue::new_ptr(struct_id));
    }
    Ok(BxValue::new_ptr(arr_id))
}

#[cfg(feature = "bif-datasource")]
fn query_result_to_struct(vm: &mut dyn BxVM, query: BxQuery) -> Result<BxValue, String> {
    let outer_id = vm.struct_new();
    for (col_idx, col) in query.columns.iter().enumerate() {
        let arr_id = vm.array_new();
        let col_data = query.data.get(col_idx);
        for row_idx in 0..query.record_count {
            let val = col_data
                .and_then(|d| d.get(row_idx))
                .cloned()
                .unwrap_or(SqlValue::Null);
            let bx = sql_to_bx(vm, &val);
            vm.array_push(arr_id, bx);
        }
        vm.struct_set(outer_id, &col.name, BxValue::new_ptr(arr_id));
    }
    Ok(BxValue::new_ptr(outer_id))
}

#[cfg(feature = "bif-datasource")]
fn parse_col_type(s: &str) -> QueryColumnType {
    match s.to_lowercase().as_str() {
        "varchar" | "string" | "cf_sql_varchar" | "cf_sql_char" | "cf_sql_longvarchar" => {
            QueryColumnType::Varchar
        }
        "integer" | "int" | "cf_sql_integer" | "cf_sql_smallint" | "cf_sql_tinyint" => {
            QueryColumnType::Integer
        }
        "bigint" | "cf_sql_bigint" => QueryColumnType::BigInt,
        "double" | "float" | "cf_sql_double" | "cf_sql_float" | "cf_sql_real" => {
            QueryColumnType::Double
        }
        "decimal" | "numeric" | "cf_sql_decimal" | "cf_sql_numeric" | "cf_sql_money" => {
            QueryColumnType::Decimal
        }
        "boolean" | "bit" | "cf_sql_bit" => QueryColumnType::Boolean,
        "date" | "cf_sql_date" => QueryColumnType::Date,
        "timestamp" | "datetime" | "cf_sql_timestamp" => QueryColumnType::Timestamp,
        "blob" | "binary" | "cf_sql_blob" | "cf_sql_binary" | "cf_sql_varbinary" => {
            QueryColumnType::Blob
        }
        other => QueryColumnType::Other(other.to_string()),
    }
}

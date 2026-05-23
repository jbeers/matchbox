# PRD: MatchBox Remaining Gaps — Complete Feature Parity Roadmap

**Status:** Draft  
**Target:** Identify and plan all remaining work to bring MatchBox to full BoxLang feature parity.

---

## 1. Current State

**Done on `feature/parser` (30 of 34 issues):**
- Custom hand-written lexer (replaced pest) — 34 tokenizer tests
- Recursive-descent + Pratt expression parser covering all BoxLang script syntax
- 11-mode template lexer with script islands, output mode, comments
- Template parser with 15+ tag types
- All statement features (do/while, assert, param, rethrow, include, not)
- All expression features (bitwise, range, xor/eqv, ::, instanceof/castas/contains, spread, power, destructuring, access modifiers)
- 79 integration tests, 0 failures
- VM opcodes: bitwise (7), power/xor/eqv (3), range (1), contains (1), buffer_write (1), array_spread (1)

**Deferred (4 issues — see Section 2.1):**
- C005: Include runtime
- C010: Access modifier enforcement
- C011: Template `#expr#` interpolation
- C012: Template script island compilation

**Completely absent:**
- SQL Query-of-Queries parser and runtime
- Date/time system beyond `now()` and `getTickCount()`
- Several standard BIF categories
- Web scopes, session management, application lifecycle
- Additional database drivers (MySQL, SQLite)
- XML, PDF, spreadsheet, image processing, email, ORM, etc.

---

## 2. Remaining Work — Detailed Issue Breakdowns

---

### 2.1 Tier 1: Compiler Backend (4 issues — deferred from C005-C012)

Parser/compiler features already coded but needing runtime semantics.

#### C011: Template #expr# Interpolation Boundary Tokens

**Effort:** Small | **Issue file:** `c011-template-interpolation-fix.md`

Add `InterpStart`/`InterpEnd` boundary tokens around `#expr#` interpolation in template output mode. The template parser then collects alternating ContentText + expression tokens and builds `StringInterpolation` for `BufferOutput`.

**Acceptance criteria:**
- [ ] InterpStart/InterpEnd tokens emitted around #expr# in output mode
- [ ] Template parser builds StringInterpolation from text + expressions
- [ ] Single BufferOutput per output block (merged text + expressions)
- [ ] Multiple #expr# segments in one output block work
- [ ] `##` produces literal `#`
- [ ] `#expr#` outside output mode is literal text (unchanged)
- [ ] Integration test verifies output buffer content

---

#### C012: Template Script Island Compilation

**Effort:** Small | **Issue file:** `c012-template-script-island-fix.md`

Same boundary token fix enables `<bx:script>` content parsing. Emit `ScriptIslandStart`/`ScriptIslandEnd` tokens. Template parser collects script tokens, parses with script parser, returns statements inline.

**Acceptance criteria:**
- [ ] ScriptIslandStart/End tokens emitted around script island body
- [ ] Template parser compiles script content inline
- [ ] Variables declared in script island accessible to subsequent template tags
- [ ] Multiple script islands work
- [ ] Integration test verifies script island execution

---

#### C005: Include Statement Runtime

**Effort:** Medium | **Issue file:** `c005-include-runtime.md`

**Phase 1 (static includes):** Detect string literal path at compile time, read file, parse, compile as sub-chunk, emit CALL.  
**Phase 2 (dynamic includes):** Add `INCLUDE` VM opcode for runtime file loading.

**Acceptance criteria:**
- [ ] `include "file.bxs"` with string literal loads and executes at compile time
- [ ] Variables set in included file visible to caller
- [ ] File not found produces compile error
- [ ] Integration test passes

---

#### C010: Access Modifier Enforcement

**Effort:** Medium | **Issue file:** `c010-access-modifier-enforcement.md`

Compile-time validation: `ClassRegistry` tracks abstract/final classes. Check `new AbstractClass()`, `extends FinalClass`, `override FinalMethod`, `this`/`super` in static context.

**Acceptance criteria:**
- [ ] Abstract class instantiation → compile error
- [ ] Missing abstract method implementation → compile error
- [ ] Extending final class → compile error
- [ ] Overriding final method → compile error
- [ ] `this` in static context → compile error
- [ ] Integration tests for each error case

---

### 2.2 Tier 2: Core Runtime Gaps (Date/Time, BIFs, Regex, JSON)

These are BoxLang features needed for practical scripting but currently missing.

#### D001: DateTime Type and Core BIFs

**Effort:** Large | **Depends on:** None

Add a `DateTime` type to the VM and implement the core date/time BIFs.

**Scope:**
- DateTime type in VM (chrono-backed, timezone-aware)
- **High:** `createDate(y, m, d)`, `createDateTime(y, m, d, h, min, sec)`, `now()`
- **High:** `dateAdd(part, number, date)`, `dateDiff(part, date1, date2)`
- **Medium:** `dateFormat(date, mask)`, `parseDateTime(str)`
- **Medium:** `day()`, `month()`, `year()`, `hour()`, `minute()`, `second()`
- **Low:** `createTimeSpan()`, `daysInMonth()`, `dayOfWeek()`, `dayOfYear()`, `quarter()`, `week()`

**Acceptance criteria:**
- [ ] DateTime type in GC heap with field access
- [ ] `now()` returns current date/time
- [ ] `createDate(2024, 1, 15)` returns Jan 15 2024
- [ ] `dateAdd("d", 7, someDate)` adds 7 days
- [ ] `dateDiff("d", date1, date2)` returns day count
- [ ] `dateFormat(date, "yyyy-mm-dd")` produces formatted strings
- [ ] `parseDateTime("2024-01-15")` creates DateTime
- [ ] Date comparison operators work
- [ ] Integration tests for each BIF

---

#### D002: Array BIFs

**Effort:** Medium | **Depends on:** None

Add missing array manipulation BIFs. Many can be implemented in the prelude in BoxLang itself.

**Scope:**
- `arrayDeleteAt(arr, position)` — delete element at position (shifts left)
- `arrayInsertAt(arr, position, value)` — insert at position (shifts right)
- `arrayPrepend(arr, value)` — insert at beginning
- `arraySwap(arr, pos1, pos2)` — swap two elements
- `arrayResize(arr, size)` — resize array
- `arrayFind(arr, value)` — find position of value (returns 0 if not found)
- `arrayFindAll(arr, value)` — find all positions
- `arrayContains(arr, value)` — check if contains (already via `contains` opcode)
- `arrayDelete(arr, value)` — delete first occurrence
- `arrayClear(arr)` — remove all elements
- `arrayIsEmpty(arr)` — check if empty
- `arrayToList(arr, delimiter)` — convert to delimited string

**Acceptance criteria:**
- [ ] Each BIF implemented (native or prelude)
- [ ] 1-based indexing throughout
- [ ] Integration tests for each BIF

---

#### D003: Struct BIFs

**Effort:** Medium | **Depends on:** None

Add missing struct manipulation BIFs.

**Scope:**
- `structDelete(st, key)` — remove key
- `structClear(st)` — remove all keys
- `structIsEmpty(st)` — check if empty
- `structFind(st, value)` — find key by value
- `structFindKey(st, key)` — check key existence (already via `contains` opcode)
- `structFindValue(st, value)` — find value
- `structGet(st, key)` — get value with dot-path support
- `structUpdate(st1, st2)` — merge struct2 into struct1

**Acceptance criteria:**
- [ ] Each BIF implemented
- [ ] Case-insensitive key matching
- [ ] Integration tests for each BIF

---

#### D004: String BIFs

**Effort:** Medium | **Depends on:** None

Add missing string manipulation BIFs.

**Scope:**
- `find(substring, string [, start])` — find position (case-sensitive)
- `findNoCase(substring, string [, start])` — find position (case-insensitive)
- `left(string, count)` — left N characters
- `right(string, count)` — right N characters
- `mid(string, start, count)` — extract substring
- `replace(string, find, replace [, scope])` — replace substrings
- `replaceNoCase(string, find, replace)` — case-insensitive replace
- `reverse(string)` — reverse characters
- `lTrim(string)` / `rTrim(string)` / `trim(string)` — whitespace trimming
- `spanExcluding(string, set)` — extract chars up to set
- `spanIncluding(string, set)` — extract chars within set
- `stripCr(string)` — remove carriage returns

**Acceptance criteria:**
- [ ] Each BIF implemented (native or prelude)
- [ ] 1-based string indexing
- [ ] Integration tests for each BIF

---

#### D005: List BIFs

**Effort:** Small | **Depends on:** D004 (string BIFs)

Add list manipulation BIFs (delimited string operations).

**Scope:**
- `listLen(list [, delimiter])` — count items
- `listGetAt(list, position [, delimiter])` — get item at position
- `listAppend(list, value [, delimiter])` — append item
- `listFirst(list [, delimiter])` — first item
- `listLast(list [, delimiter])` — last item
- `listRest(list [, delimiter])` — all but first item
- `listFind(list, value [, delimiter])` — find position
- `listDeleteAt(list, position [, delimiter])` — delete at position
- `listSort(list [, type, direction, delimiter])` — sort items

**Acceptance criteria:**
- [ ] Default delimiter = comma
- [ ] Multi-char delimiters supported
- [ ] Integration tests for each BIF

---

#### D006: Math BIFs

**Effort:** Small | **Depends on:** None

Add missing math functions.

**Scope:**
- `ceiling(number)` — round up
- `floor(number)` — round down
- `int(number)` — truncate to integer
- `log(number)` — natural logarithm
- `log10(number)` — base-10 logarithm
- `exp(number)` — e^x
- `sgn(number)` — sign (-1, 0, 1)
- `sqr(number)` — square root
- `cos(number)` / `sin(number)` / `tan(number)` — trigonometry
- `acos(number)` / `asin(number)` / `atan(number)` — inverse trigonometry
- `pi()` — return pi constant
- `randomize([seed])` — seed random generator

**Acceptance criteria:**
- [ ] Each BIF returns correct mathematical value
- [ ] Integration tests for each BIF

---

#### D007: Regex BIFs

**Effort:** Medium | **Depends on:** None (regex crate already available)

Add full regex BIFs (only `reMatch` exists).

**Scope:**
- `reMatch(pattern, string)` — exists, keep
- `reMatchNoCase(pattern, string)` — case-insensitive match
- `reFind(pattern, string [, start, returnSubExpressions])` — find pattern, return struct with pos/len/match/subexpressions
- `reFindNoCase(pattern, string [, start])` — case-insensitive find
- `reReplace(string, pattern, replacement [, scope])` — replace matches
- `reReplaceNoCase(string, pattern, replacement)` — case-insensitive replace

**Acceptance criteria:**
- [ ] Each BIF implemented using `regex` crate
- [ ] Scope parameter: "ONE" (first) or "ALL" (all matches)
- [ ] Integration tests with pattern matching

---

#### D008: JSON BIFs

**Effort:** Medium | **Depends on:** None (serde_json already available)

Improve JSON BIFs (basic support exists in JS interop).

**Scope:**
- `serializeJSON(value [, serializeQueryByColumns])` — encode to JSON
- `deserializeJSON(jsonString [, strictMapping])` — decode from JSON
- `isJSON(string)` — validate JSON string
- Handle BoxLang-specific types: queries, structs (case-insensitive keys), dates

**Acceptance criteria:**
- [ ] serializeJSON handles structs, arrays, numbers, strings, booleans, null
- [ ] deserializeJSON reconstructs BoxLang values correctly
- [ ] Case-insensitive struct keys preserved
- [ ] Integration tests for round-trip serialization

---

#### D009: Utility BIFs

**Effort:** Medium | **Depends on:** D001, D008

Add general utility and type-checking BIFs.

**Scope:**
- `writeOutput(value)` — write to output buffer (used by templates)
- `writeDump(value [, expand])` — debug output
- `createUUID()` — generate UUID v4
- `sleep(milliseconds)` — pause execution
- `isNull(value)` — null check
- `isDefined(variableName)` — variable existence check
- `isNumeric(value)` / `isBoolean(value)` / `isDate(value)` — type checks
- `isStruct(value)` / `isArray(value)` / `isObject(value)` / `isSimpleValue(value)`
- `duplicate(value)` — deep copy
- `evaluate(expressionString)` — evaluate expression string
- `getTickCount()` — exists, keep

**Acceptance criteria:**
- [ ] writeOutput writes to output buffer (uses BUFFER_WRITE opcode)
- [ ] createUUID produces valid UUID v4
- [ ] Type-checking BIFs return correct booleans
- [ ] duplicate deep-copies structs and arrays
- [ ] Integration tests for each BIF

---

#### D010: Crypto BIFs

**Effort:** Low | **Depends on:** None

Extend crypto BIFs beyond `hash` (SHA-256).

**Scope:**
- `hash(value [, algorithm, encoding, iterations])` — extend with algorithm parameter
- Supported algorithms: MD5, SHA-1, SHA-256, SHA-384, SHA-512
- `hmac(value, key [, algorithm, encoding])` — HMAC
- `encrypt(value, key [, algorithm, encoding])` — symmetric encryption (deferred)
- `decrypt(value, key [, algorithm, encoding])` — symmetric decryption (deferred)
- `generateSecretKey(algorithm [, keySize])` — key generation (deferred)

**Acceptance criteria:**
- [ ] hash supports multiple algorithms via parameter
- [ ] hmac produces correct MAC
- [ ] Integration tests for each supported algorithm

---

### 2.3 Tier 3: SQL Query-of-Queries

BoxLang's QoQ supports running SQL SELECT against in-memory Query objects. This is a complete subsystem with its own ANTLR grammar (SQLLexer.g4 + SQLGrammar.g4), AST (34 node types), and execution engine.

#### Q001: SQL Lexer

**Effort:** Large | **Depends on:** None

Implement a hand-written SQL lexer based on the SQLite grammar subset used in BoxLang's `SQLLexer.g4`.

**Scope:**
- Case-insensitive keyword matching (SELECT, FROM, WHERE, JOIN, etc.)
- Operators: arithmetic, comparison, boolean, bitwise, string concat (`||`)
- Literals: integers, decimals, strings (single-quoted, with `''` escaping), booleans, NULL
- ODBC date/time literals: `{d 'yyyy-mm-dd'}`, `{t 'hh:mm:ss'}`, `{ts 'yyyy-mm-dd hh:mm:ss'}`
- Identifiers: regular, bracketed `[name]`, backtick `` `name` ``, double-quoted `"name"`
- Bind parameters: `?` and `:name`
- Line comments (`--`) and block comments (`/* */`) → skip
- Whitespace → skip

**Token kinds needed:** 80+ (SELECT, FROM, WHERE, JOIN, LEFT, RIGHT, INNER, OUTER, CROSS, FULL, ON, GROUP, BY, HAVING, ORDER, ASC, DESC, LIMIT, TOP, DISTINCT, ALL, AS, UNION, CASE, WHEN, THEN, ELSE, END, BETWEEN, IN, LIKE, ESCAPE, IS, NULL, NOT, AND, OR, XOR, CAST, CONVERT, COUNT, SUM, AVG, MIN, MAX, COALESCE, UPPER, LOWER, LENGTH, TRIM, function_name, identifier, string_literal, numeric_literal, plus, minus, star, slash, percent, concat, eq, neq, lt, gt, le, ge, bang, bitand, bitor, bitxor, bitnot, comma, dot, lparen, rparen, semicolon, etc.)

**Acceptance criteria:**
- [ ] All SQL keywords tokenized case-insensitively
- [ ] String literals with `''` escape
- [ ] Bracket/backtick/quoted identifiers
- [ ] ODBC date literals
- [ ] Comments skipped
- [ ] 40+ tokenizer unit tests

---

#### Q002: SQL Parser

**Effort:** Large | **Depends on:** Q001

Implement a recursive-descent SQL parser producing SQL AST nodes.

**Scope — SELECT statement:**
- Result columns: `*`, `table.*`, `expr AS alias`, `expr alias`
- DISTINCT, TOP n
- FROM clause: table names, `schema.table`, `(subquery) AS alias`, comma-separated tables
- JOIN clause: INNER/LEFT/RIGHT/FULL/CROSS JOIN with ON
- WHERE clause with arbitrary boolean expressions
- GROUP BY with multiple expressions
- HAVING clause
- ORDER BY with ASC/DESC
- LIMIT n
- UNION [ALL | DISTINCT]

**Scope — Expressions:**
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Bitwise: `&`, `|`, `^`, `~`
- String: `||`
- Comparison: `=`, `!=`, `<>`, `<`, `<=`, `>`, `>=`
- Boolean: AND, OR, NOT (`!`)
- Predicates: BETWEEN x AND y, IN (list), IN (subquery), LIKE pattern, IS NULL / IS NOT NULL
- Subqueries in FROM and IN
- CASE WHEN expr THEN expr [WHEN ...] [ELSE expr] END
- Functions: `FUNC(args)` — generic dispatch
- Aggregates: `COUNT(*)`, `COUNT(DISTINCT col)`, `SUM`, `AVG`, `MIN`, `MAX`
- Scalar functions: `UPPER`, `LOWER`, `LENGTH`, `TRIM`, `COALESCE`, `CAST(expr AS type)`, `CONVERT(type, expr)`

**Acceptance criteria:**
- [ ] Full SELECT statement parsing
- [ ] All JOIN types with ON clauses
- [ ] Subqueries in FROM and IN
- [ ] UNION parsing
- [ ] CASE expressions
- [ ] Aggregate and scalar function calls
- [ ] Parse errors with source location
- [ ] 30+ parser unit tests

---

#### Q003: SQL AST

**Effort:** Large | **Depends on:** Q002

Define the SQL AST node types in MatchBox's type system.

**AST node hierarchy:**
```
SQLStatement (root)
 ├── SQLSelect (columns, table, joins, where, groupBy, having, orderBy, limit)
 ├── SQLTable (abstract)
 │    ├── SQLTableVariable (named query reference)
 │    └── SQLTableSubQuery (nested SELECT)
 ├── SQLJoin (type: INNER|LEFT|RIGHT|FULL|CROSS, table, on)
 ├── SQLUnion (select, type: ALL|DISTINCT)
 ├── SQLResultColumn (expression, alias, position)
 ├── SQLExpression (abstract)
 │    ├── SQLColumn (table name, column name)
 │    ├── SQLStarExpression (* or table.*)
 │    ├── SQLBinaryOperation (left, operator, right)
 │    ├── SQLUnaryOperation (operator, expr)
 │    ├── SQLBetweenOperation (expr, low, high)
 │    ├── SQLInOperation (expr, list)
 │    ├── SQLInSubQueryOperation (expr, subquery)
 │    ├── SQLCase (expression, when/then pairs, else)
 │    │    └── SQLCaseWhenThen (when, then)
 │    ├── SQLFunction (name, args, distinct?)
 │    ├── SQLOrderBy (expr, ASC|DESC)
 │    └── SQL literals (Number, String, Boolean, Null, Date)
```

**Acceptance criteria:**
- [ ] All node types defined with Debug, Clone, PartialEq
- [ ] Source span tracking on each node
- [ ] SQLVisitor trait for traversing the AST

---

#### Q004: QoQ Execution Engine

**Effort:** Very Large | **Depends on:** Q003

Execute parsed SQL AST against in-memory `Query` objects loaded from the BoxLang variable scope.

**Phases of execution:**
1. **Table resolution** — Load named queries from variable scope. Execute subquery SELECTs recursively.
2. **Intersection generation** — Create Cartesian product of table rows. Apply JOIN ON filters. Handle LEFT/RIGHT/FULL outer join semantics (include unmatched rows with NULL padding).
3. **WHERE filtering** — Evaluate WHERE expression against each row intersection. Remove non-matching rows.
4. **GROUP BY aggregation** — Partition rows by GROUP BY expressions. Evaluate aggregate functions (COUNT, SUM, AVG, MIN, MAX) per partition. Apply HAVING filter.
5. **Row projection** — Evaluate SELECT column expressions for each row. Build result column arrays.
6. **Post-processing** — DISTINCT deduplication. ORDER BY sorting. LIMIT truncation. UNION merging.

**Built-in functions:**
- Aggregates: COUNT, SUM, AVG, MIN, MAX
- Scalars: UPPER, LOWER, LENGTH, TRIM, COALESCE, CAST, CONVERT
- Math: ABS, CEILING, FLOOR, EXP, SQRT, COS, SIN, TAN, ACOS, ASIN, ATAN, MOD, POWER
- String: CONCAT, LEFT, RIGHT, LTRIM, RTRIM

**Acceptance criteria:**
- [ ] Named queries loaded from variable scope
- [ ] CROSS JOIN / INNER JOIN with ON filtering
- [ ] LEFT/RIGHT/FULL outer joins with NULL padding
- [ ] WHERE clause filtering
- [ ] GROUP BY with aggregate functions
- [ ] HAVING filtering
- [ ] ORDER BY with ASC/DESC
- [ ] DISTINCT deduplication
- [ ] LIMIT truncation
- [ ] UNION merging
- [ ] Subquery execution
- [ ] CASE expression evaluation
- [ ] 15+ QoQ integration tests

---

#### Q005: QoQ BIF Integration

**Effort:** Medium | **Depends on:** Q004

Wire QoQ into `queryExecute()` BIF when `{dbtype: "query"}` option is passed.

**Scope:**
- Detect `dbtype === "query"` in `queryExecute` options
- Route to QoQ parser + execution engine instead of database driver
- Return result as Query object
- Support parameterized queries with `?` placeholders and named params

**Acceptance criteria:**
- [ ] `queryExecute("SELECT * FROM myQuery WHERE age > ?", [18], {dbtype: "query"})` works
- [ ] Result is a Query object
- [ ] Parameterized queries work with positional and named params
- [ ] Integration test with in-memory query data

---

### 2.4 Tier 4: Database & Web Platform Features

#### W001: MySQL/MariaDB Driver

**Effort:** Medium | **Depends on:** None (mysql crate)

Add MySQL database driver alongside existing PostgreSQL driver.

**Scope:**
- Implement `DbDriver` trait for MySQL
- Connection pooling via r2d2
- Column type mapping (INT, BIGINT, FLOAT, DOUBLE, DECIMAL, VARCHAR, TEXT, DATE, DATETIME, TIMESTAMP, BLOB, BOOL)
- Parameterized queries with `?` placeholders → MySQL `?` syntax
- Register in datasource registry: `datasourceRegister("mydb", {driver: "mysql", ...})`

**Acceptance criteria:**
- [ ] MySQL SELECT/INSERT/UPDATE/DELETE queries
- [ ] Parameterized queries with type hints
- [ ] Datasource config via matchbox.toml
- [ ] Integration tests

---

#### W002: SQLite Driver

**Effort:** Medium | **Depends on:** None (rusqlite crate)

Add embedded SQLite driver. Unlike PostgreSQL/MySQL, SQLite is serverless — useful for desktop/embedded apps.

**Scope:**
- Implement `DbDriver` trait for SQLite
- In-memory and file-based databases
- Column type mapping (INTEGER, REAL, TEXT, BLOB, NUMERIC)
- Register in datasource registry: `datasourceRegister("local", {driver: "sqlite", database: "data.db"})`

**Acceptance criteria:**
- [ ] SQLite file-based and in-memory databases
- [ ] Parameterized queries
- [ ] Datasource config via matchbox.toml
- [ ] Integration tests

---

#### W003: Transaction Support

**Effort:** Medium | **Depends on:** W001, W002, existing PostgreSQL driver

Implement `transactionBegin`, `transactionCommit`, `transactionRollback` (currently stubs).

**Scope:**
- Track active transaction per connection
- Nested transaction support (savepoints)
- Auto-rollback on error
- Thread-safe transaction state

**Acceptance criteria:**
- [ ] transactionBegin/Commit/Rollback work
- [ ] Nested transactions via savepoints
- [ ] Auto-rollback on unhandled errors
- [ ] Integration tests

---

#### W004: Web Scopes

**Effort:** Large | **Depends on:** None

Implement `url`, `form`, `cookie`, `session`, `request`, `cgi` scopes as global structs accessible from BoxLang code in web contexts.

**Scope:**
- `url` scope: query string parameters (parsed in server, exposed as struct)
- `form` scope: POST body parameters
- `cookie` scope: read/write HTTP cookies
- `session` scope: server-side session storage with expiry
- `cgi` scope: server environment variables (SERVER_NAME, REQUEST_METHOD, etc.)
- `request` scope: per-request scratch space
- `server` scope: server-level configuration

**Acceptance criteria:**
- [ ] `url.name` in template returns query param
- [ ] `form.field` in template returns form field
- [ ] `cookie.MBX_SESSION_ID` reads cookie
- [ ] `session.visitCount = session.visitCount + 1` persists across requests
- [ ] Integration tests for each scope

---

#### W005: Application Lifecycle

**Effort:** Large | **Depends on:** W004

Support `Application.bx` with lifecycle events.

**Scope:**
- `onApplicationStart()` — called once when app starts
- `onRequestStart(targetPage)` — called before each request
- `onRequest(targetPage)` — default request handler
- `onRequestEnd()` — called after each request
- `onSessionStart()` / `onSessionEnd()` — session lifecycle
- `onError(exception, eventName)` — error handler
- `this.name` — application name
- `this.sessionManagement` — enable sessions
- `this.sessionTimeout` — session timeout
- `this.applicationTimeout` — application timeout

**Acceptance criteria:**
- [ ] Application.bx loaded from web root
- [ ] Lifecycle events called in correct order
- [ ] Application timeout and reload
- [ ] Integration test covering full request lifecycle

---

#### W006: File Upload

**Effort:** Medium | **Depends on:** W004 (form scope)

Add `fileUpload` and `fileUploadAll` BIFs for handling multipart form uploads.

**Scope:**
- Parse multipart/form-data requests
- `fileUpload(destination [, fileField, accept, nameConflict])` — save single upload
- `fileUploadAll(destination [, accept, nameConflict])` — save all uploads
- Return struct with upload metadata (clientFileName, serverFile, fileSize, contentType)
- Name conflict strategies: ERROR, SKIP, OVERWRITE, MAKEUNIQUE

**Acceptance criteria:**
- [ ] Single file upload works
- [ ] Multiple file upload works
- [ ] Name conflict strategies honored
- [ ] Integration test with multipart form

---

### 2.5 Tier 5: Subsystem Parity (Long-term)

These are large BoxLang subsystems that would need dedicated PRDs if pursued. Listed here for completeness.

#### S001: XML Processing
**Effort:** Very Large  
XML parsing (`xmlParse`), XPath search (`xmlSearch`), XSLT transform (`xmlTransform`), XML document creation (`xmlNew`), DOM manipulation.

#### S002: Email (SMTP/POP3/IMAP)
**Effort:** Large  
`mailSend()`, `mailRead()`, SMTP with TLS/SSL, POP3, IMAP, attachments, HTML email.

#### S003: PDF Generation
**Effort:** Very Large  
`cfdocument` equivalent, HTML-to-PDF rendering, page formatting, headers/footers.

#### S004: Spreadsheet (Excel)
**Effort:** Very Large  
`spreadsheetNew()`, `spreadsheetRead()`, cell formatting, formulas, charts.

#### S005: Image Processing
**Effort:** Very Large  
`imageNew()`, `imageRead()`, `imageWrite()`, resize, crop, rotate, filters, text drawing.

#### S006: Scheduling / Tasks
**Effort:** Large  
Cron-like task scheduler, `cfschedule`, background task execution.

#### S007: Caching (User-facing)
**Effort:** Large  
`cachePut()`, `cacheGet()`, `cacheRemove()`, in-memory/disk/Ehcache backends.

#### S008: Logging Framework
**Effort:** Medium  
`writeLog()`, log levels, file/console/network appenders.

#### S009: Internationalization (i18n)
**Effort:** Large  
Resource bundles, `i18nGetResource()`, locale-aware formatting.

#### S010: Validation Framework
**Effort:** Medium  
Server-side validation, required/type/range/regex validators, error messages.

#### S011: ColdBox-style Interceptors/Events
**Effort:** Large  
Event-driven architecture, interceptor chains, `announceInterception()`.

#### S012: LDAP Integration
**Effort:** Medium  
LDAP query, authentication, directory search.

#### S013: Charting / Graphing
**Effort:** Very Large  
Chart generation (bar, line, pie, etc.), image output, data binding.

---

## 3. Out of Scope (Permanently)

- CFML compatibility (`CFLexer.g4` / `CFGrammar.g4`)
- JVM/JAR integration
- Hibernate ORM (JVM-dependent)
- Any subsystem requiring a JVM

---

## 4. Issue Summary

| Tier | Issues | Count | Total Effort |
|---|---|---|---|
| **Tier 1** | C005, C010, C011, C012 | 4 | 0.5-1 week |
| **Tier 2** | D001-D010 | 10 | 2-3 weeks |
| **Tier 3** | Q001-Q005 | 5 | 3-4 weeks |
| **Tier 4** | W001-W006 | 6 | 2-3 weeks |
| **Tier 5** | S001-S013 | 13 | 12+ weeks (long-term) |
| **Total** | | **38 issues** | |

---

## 5. Success Metrics Per Tier

| Tier | Tests | Feature Flags |
|---|---|---|
| Tier 1 | 4 new integration tests | None |
| Tier 2 | 30+ BIF integration tests | None (core) |
| Tier 3 | 30+ SQL parser tests, 15+ QoQ tests | `qoq` feature flag |
| Tier 4 | 15+ database/web integration tests | `bif-datasource` |
| Tier 5 | Per-subsystem suites | Per-subsystem feature flags |

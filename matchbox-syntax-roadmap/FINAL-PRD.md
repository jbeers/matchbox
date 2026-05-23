# PRD: MatchBox Remaining Gaps — Complete Feature Parity Roadmap

**Status:** Draft  
**Target:** Identify and plan all remaining work to bring MatchBox to full BoxLang feature parity.

---

## 1. Current State

**Done on `feature/parser`:**
- Custom hand-written lexer (replaced pest) with 34 tokenizer tests
- Recursive-descent + Pratt expression parser covering all BoxLang script syntax
- 11-mode template lexer with script islands, output mode, comments
- Template parser with 15+ tag types
- All statement features (do/while, assert, param, rethrow, include, not)
- All expression features (bitwise, range, xor/eqv, ::, instanceof/castas/contains, spread, power, destructuring, access modifiers)
- 79 integration tests, 0 failures
- VM opcodes: bitwise (7), power/xor/eqv (3), range (1), contains (1), buffer_write (1), array_spread (1)

**Deferred from compiler backend:**
- C005: Include runtime — needs VM file I/O or pre-compiled constant embedding
- C010: Access modifier enforcement — needs class metadata tracking
- C011: Template `#expr#` interpolation — needs boundary tokens between template/expression lexer modes
- C012: Template script island compilation — same boundary token issue

**Completely absent:**
- SQL Query-of-Queries parser and runtime
- Date/time BIFs (beyond `now()` and `getTickCount()`)
- Several standard BIFs
- Web scopes, session management, application lifecycle
- XML, PDF, spreadsheet, image processing, email
- Other database drivers (MySQL, SQLite)
- Full regex, crypto, logging, scheduling, caching, ORM, file upload, i18n

---

## 2. Gap Categories

### 2.1 Compiler Backend (4 issues — deferred from C005-C012)

These are parser/compiler features already coded but needing runtime semantics.

| Issue | What's Needed | Effort |
|---|---|---|
| **Include runtime** | VM-level file loading (read file → parse → compile → execute inline). Static includes can use pre-compiled constants. Dynamic includes need VM file I/O. | Medium |
| **Access modifier enforcement** | Compiler tracks abstract/final class metadata. Check `new AbstractClass()` at compile time. Check `extends FinalClass`. Check `override FinalMethod`. | Medium |
| **Template #expr# interpolation** | Lexer needs boundary tokens (InterpStart/InterpEnd) around expression content in template output mode. Parser collects alternating text + expression tokens into StringInterpolation. | Small |
| **Template script island** | Same boundary token fix enables proper script content parsing within `<bx:script>` islands. | Small |

### 2.2 SQL Query-of-Queries (0% done)

BoxLang supports running SQL SELECT statements against in-memory `Query` objects. This is a complete subsystem with its own ANTLR grammar (SQLLexer.g4 + SQLGrammar.g4), AST (34 node types), and execution engine.

**What's needed:**
- **SQL parser** — hand-written recursive-descent SQL parser supporting SELECT with:
  - Result columns, `*`, aliases, DISTINCT, TOP
  - FROM with table variables, subqueries, schema-qualified names
  - All JOIN types (INNER, LEFT, RIGHT, FULL, CROSS)
  - WHERE, GROUP BY, HAVING, ORDER BY, LIMIT
  - Expressions: arithmetic, comparison, boolean, bitwise, string concat
  - Predicates: BETWEEN, IN, LIKE, IS NULL, IS NOT NULL
  - Subqueries in FROM and IN clauses
  - UNION [ALL | DISTINCT]
  - CASE expressions
  - Aggregates: COUNT, SUM, AVG, MIN, MAX
  - Scalar functions: UPPER, LOWER, LENGTH, TRIM, COALESCE, CAST, CONVERT, etc.
- **SQL AST** — Complete AST node tree (select, table, join, expression, literal nodes)
- **QoQ execution engine** — Execute parsed SQL against in-memory Query objects:
  - Load tables from variable scope by name
  - Execute recursive subqueries
  - Intersection generation for joins
  - WHERE filtering
  - GROUP BY aggregation with HAVING
  - ORDER BY sorting
  - DISTINCT deduplication
  - LIMIT truncation
  - UNION merging
  - Function dispatch (scalar + aggregate)
- **BIF integration** — `queryExecute(sql, params, {dbtype: "query"})` routes to QoQ

**Effort:** Very Large (3-4 weeks for a full implementation)

### 2.3 Date/Time System

MatchBox has only `now()` and `getTickCount()`. BoxLang has a rich `DateTime` object with dozens of BIFs.

| What's Needed | Priority |
|---|---|
| DateTime type in VM | High |
| `createDate(year, month, day)` | High |
| `createDateTime(y, m, d, h, min, sec)` | High |
| `now()` — already exists | Done |
| `dateAdd(part, number, date)` | Medium |
| `dateDiff(part, date1, date2)` | Medium |
| `dateFormat(date, mask)` | Medium |
| `parseDateTime(str)` | Medium |
| `day(date)` / `month(date)` / `year(date)` | Medium |
| `hour(date)` / `minute(date)` / `second(date)` | Medium |
| `createTimeSpan(days, hours, min, sec)` | Low |
| `daysInMonth(date)` / `daysInYear(date)` | Low |
| `dayOfWeek(date)` / `dayOfYear(date)` | Low |
| `firstDayOfMonth(date)` | Low |
| `quarter(date)` / `week(date)` | Low |

### 2.4 Standard BIFs

Missing BIFs beyond date/time:

| Category | Missing | Priority |
|---|---|---|
| Array | `arrayDeleteAt`, `arrayInsertAt`, `arrayPrepend`, `arraySwap`, `arrayResize`, `arrayFind`, `arrayFindAll`, `arrayContains`, `arrayDelete`, `arrayClear`, `arrayIsEmpty`, `arrayToList` | Medium |
| Struct | `structDelete`, `structClear`, `structIsEmpty`, `structFind`, `structFindKey`, `structFindValue`, `structGet`, `structUpdate` | Medium |
| String | `lCase`, `uCase` (exist as BIFs via prelude), `find`, `findNoCase`, `left`, `right`, `mid`, `replace`, `replaceNoCase`, `reverse`, `spanExcluding`, `spanIncluding`, `stripCr`, `lTrim`, `rTrim`, `trim`, `listLen`, `listGetAt`, `listAppend`, `listFirst`, `listLast`, `listRest` | Medium |
| Regex | `reMatch` (exists), `reMatchNoCase`, `reReplace`, `reReplaceNoCase`, `reFind`, `reFindNoCase` | Medium |
| Math | `abs`, `min`, `max`, `round`, `randRange` (exist in prelude); missing: `ceiling`, `floor`, `int`, `log`, `log10`, `exp`, `sgn`, `sqr`, `cos`, `sin`, `tan`, `acos`, `asin`, `atan`, `pi`, `randomize` | Medium |
| Conversion | `toString`, `toNumeric`, `toBoolean`, `toBase64`, `toBinary`, `charsetEncode`, `charsetDecode` | Low |
| File I/O | `fileRead`, `fileWrite`, `fileAppend`, `fileDelete`, `fileExists`, `fileMove`, `fileCopy`, `directoryCreate`, `directoryDelete`, `directoryExists`, `directoryList` | Low |
| JSON | `serializeJSON`, `deserializeJSON` (basic support exists in interop), `isJSON` | Medium |
| Crypto | `hash` (SHA-256 exists), missing: `hash` with other algorithms, `encrypt`, `decrypt`, `generateSecretKey`, `hmac` | Low |
| Other | `writeOutput`, `writeDump`, `getTickCount` (exists), `createUUID`, `sleep`, `throw` (exists), `isNull`, `isDefined`, `isNumeric`, `isBoolean`, `isDate`, `isStruct`, `isArray`, `isObject`, `isSimpleValue`, `getMetadata`, `duplicate`, `evaluate`, `invoke` | Medium |

### 2.5 Template Runtime Features

Beyond the lexer/parser (done), the template runtime needs:

| Feature | Status |
|---|---|
| Buffer output execution | Done (`BUFFER_WRITE` opcode) |
| `<bx:output>` body with `#expr#` | Deferred (C011) |
| `<bx:script>` island execution | Deferred (C012) |
| `<bx:loop>` with array/collection attributes | Parser done, compiler basic |
| `<bx:include template="...">` | Parser done, runtime deferred |
| `<bx:function>` template UDFs | Parser stub, not compiled |
| Template caching | Not started |
| Error handling in templates (`<bx:try>/<bx:catch>`) | Parser done, compiler basic |
| Source maps for templates | Not started |

### 2.6 Database Drivers

Only PostgreSQL exists. BoxLang supports multiple database types.

| Driver | Status |
|---|---|
| PostgreSQL | Done |
| MySQL/MariaDB | Not started |
| SQLite (embedded) | Not started |
| MSSQL | Not started |
| Oracle | Not started |
| Connection pooling abstraction | PostgreSQL-only |

### 2.7 Web Features

| Feature | Status |
|---|---|
| Web server (static + app) | Done |
| URL/form/cookie scopes | Server parses internally, no scope BIFs |
| Session management | Stubs only |
| Application lifecycle | Not started |
| WebSocket (server) | Done |
| REST route definitions | Not started |
| File upload | Not started |

### 2.8 Large Subsystems (Not Planned)

These are entire BoxLang subsystems with no MatchBox equivalent:

- **XML processing** (parse, search, transform)
- **PDF generation**
- **Spreadsheet** (read/write Excel)
- **Image processing**
- **Email** (SMTP/POP3/IMAP)
- **ORM / Hibernate**
- **Scheduling / Tasks**
- **Caching** (user-facing)
- **Logging** framework
- **Internationalization** (i18n)
- **ColdBox-style** interceptors/events
- **LDAP** integration
- **Validation** framework
- **Charting / Graphing**

---

## 3. Prioritized Implementation Plan

### Tier 1: Complete What's Started (This Session)

| Issue | Effort |
|---|---|
| C011: Template `#expr#` boundary tokens | Small |
| C012: Template script island | Small |
| C005: Include runtime (static only) | Medium |
| C010: Access modifier enforcement | Medium |

### Tier 2: Core Runtime Gaps (Next Sprint)

| Feature | Effort |
|---|---|
| Date/Time type + BIFs | Large |
| Missing array/struct/string BIFs | Medium |
| Regex BIFs | Medium |
| JSON BIF improvements | Medium |
| Output/utility BIFs (writeDump, isDefined, etc.) | Medium |

### Tier 3: SQL Query-of-Queries (Major Project)

| Feature | Effort |
|---|---|
| SQL lexer (hand-written, matching SQLite grammar) | Large |
| SQL parser (recursive-descent, 30+ production rules) | Large |
| SQL AST (34 node types) | Large |
| QoQ execution engine (intersection, filter, aggregate, sort) | Very Large |
| QoQ BIF integration | Medium |

### Tier 4: Database & Web (Platform Maturity)

| Feature | Effort |
|---|---|
| MySQL driver | Medium |
| SQLite driver | Medium |
| Transaction support | Medium |
| Web scopes (session, cookie, request) | Large |
| Application lifecycle | Large |
| File upload | Medium |

### Tier 5: Subsystem Parity (Long-term)

| Feature | Effort |
|---|---|
| XML processing | Very Large |
| Email | Large |
| PDF/Spreadsheet/Image | Very Large |
| ORM | Very Large |
| Caching, Scheduling, Logging | Large |
| i18n | Large |

---

## 4. Out of Scope (Permanently)

- CFML compatibility (`CFLexer.g4` / `CFGrammar.g4`)
- JVM/JAR integration
- Hibernate ORM (JVM-dependent)
- Any subsystem that requires a JVM

---

## 5. Success Metrics Per Tier

| Tier | Tests | Feature Flags |
|---|---|---|
| Tier 1 | 4 new integration tests | None (all parser/compiler) |
| Tier 2 | 20+ new BIF integration tests | N/A |
| Tier 3 | 30+ SQL parser unit tests, 15+ QoQ integration tests | Feature flag recommended |
| Tier 4 | 10+ database integration tests | `bif-datasource` existing |
| Tier 5 | Per-subsystem test suites | Feature flag per subsystem |

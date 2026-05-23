# PRD: MatchBox Compiler Backend — VM Opcodes for New Syntax

**Status:** Draft  
**Target:** Implement compiler bytecode generation and VM opcodes for all syntax features added in the parser roadmap (#001-#018 and T001-T016).

---

## 1. Current State

The parser correctly handles all BoxLang-native script syntax and template syntax. The AST has variants for every construct. The compiler emits *basic* bytecode for each, but many operations are implemented as no-ops or passthroughs. They parse without error but don't execute correctly.

## 2. What Needs Compiler Work

### 2.1 Bitwise Operators

**Parsed:** `b|`, `b&`, `b^`, `b~`, `b<<`, `b>>`, `b>>>`  
**Current compiler:** Operator string passed through as generic `Binary` operation. The VM has no bitwise opcodes.  
**Needed:** Add VM opcodes (BIT_OR, BIT_AND, BIT_XOR, BIT_NOT, BIT_SHL, BIT_SHR, BIT_USHR). Implement in VM interpreter. Update compiler to emit correct opcodes for each bitwise operator.

### 2.2 Range Operators

**Parsed:** `..`, `>..`, `..<`, `>..<`  
**Current compiler:** No range handling. Ranges should create range objects usable in `for-in` loops.  
**Needed:** Emit bytecode to construct a range value (struct with start/end/inclusivity). The `for-in` loop already handles iterating over collections — ranges should implement the same iteration protocol or be treated as a simple value.

### 2.3 Power Operator `^`

**Parsed:** `a ^ b` (right-associative)  
**Current compiler:** Treated as generic binary op, VM has no pow opcode.  
**Needed:** Add POW opcode or emit call to `math.pow()` runtime function.

### 2.4 XOR / EQV

**Parsed:** `a XOR b`, `a EQV b`  
**Current compiler:** Treated as generic binary op, VM has no XOR/EQV opcodes.  
**Needed:** Add XOR and EQV opcodes (logical exclusive-or and equivalence).

### 2.5 castAs / instanceOf / contains

**Parsed:** `expr castAs "Type"`, `expr instanceOf Type`, `expr contains expr`  
**Current compiler:** No handling.  
**Needed:** For MVP, emit runtime function calls (`castAs()`, `instanceOf()`, `contains()`) — these don't need dedicated opcodes if runtime BIFs exist.

### 2.6 Functional BIF `::method`

**Parsed:** `::ucase` — reference to built-in function.  
**Current compiler:** Treated as plain Identifier.  
**Needed:** Resolve the BIF name at compile time and emit a `CONSTANT(FunctionRef)` or similar. The VM needs to treat BIF references as callable values.

### 2.7 Spread Expression `...`

**Parsed:** `...expr` in function args, array literals, struct literals.  
**Current compiler:** No desugaring.  
**Needed:** At compile time, desugar spread into iteration + push operations. For `func(...arr)`: iterate arr, push each element as argument. For `[...arr]`: iterate arr, push each element into array. For `{...obj}`: iterate obj keys, push key-value pairs.

### 2.8 Destructuring

**Parsed:** `{ a, b } = obj`, `[ a, b ] = arr`  
**Current compiler:** Basic bytecode for object destructuring via member access. Array destructuring not implemented. Rename (`a: localA`) and rest (`...rest`) not supported.  
**Needed:** Array destructuring via index access. Rename support. Rest binding support.

### 2.9 Access Modifiers

**Parsed:** `remote`, `package`, `static`, `abstract`, `final` on functions/classes.  
**Current compiler:** Mostly ignored. Abstract functions skipped.  
**Needed:** Static function dispatch (no `this` context). Final class/function enforcement at compile time. Abstract class validation. Remote/package are mostly metadata for now.

### 2.10 Statement Semantics

Several statements have incomplete runtime semantics:

| Statement | Parser | Compiler | Gap |
|---|---|---|---|
| `assert` | OK | Emits JUMP_IF_FALSE | **Should throw AssertError on failure** |
| `param` | OK | DEFINE_GLOBAL with null | **Should only set if var is undefined/null; throw if no default and undefined** |
| `rethrow` | OK | Emits THROW | **Should re-throw caught exception preserving stack** |
| `include` | OK | Evaluates expression + POP | **Should load, parse, compile, and execute another file** |
| `not` (statement) | OK | Compiles expr + POP | **Functionally correct (no-op), but should verify it doesn't produce side-effect surprises** |

### 2.11 Template Compiler

| Feature | Parser | Compiler | Gap |
|---|---|---|---|
| `BufferOutput` | OK | Emits PRINT | **Should write to output buffer, not stdout** |
| `<bx:output>` body | Partial | N/A | **`#expr#` interpolation in body needs string concatenation + output** |
| `<bx:loop>` | OK (desugared to WhileLoop) | Uses WhileLoop bytecode | **Loop variable scoping and iteration semantics** |
| `<bx:script>` | OK | N/A | **Script island statements should compile inline in template scope** |
| `<bx:include>` | OK | Evaluates path | **Should load and inline another template** |
| `<bx:function>` | Partial | N/A | **Template UDFs need compilation as callable functions** |
| Generic components | OK | N/A | **Component invocation at runtime** |
| `#expr#` in attr values | Partial | N/A | **Expression attributes should be evaluated** |

### 2.12 Output Buffer

**Context:** Templates need to write to an output buffer (captured for HTTP response).  
**Current state:** `BufferOutput` emits `PRINT` opcode which writes to stdout.  
**Needed:** Add `BUFFER_WRITE` opcode that appends to `vm.output_buffer`. If `output_buffer` is `None`, fall back to stdout. The web server sets `vm.output_buffer = Some(String::new())` before template execution.

---

## 3. VM Opcodes to Add

| Opcode | Purpose | Priority |
|---|---|---|
| BIT_OR, BIT_AND, BIT_XOR, BIT_NOT | Bitwise ops | High |
| BIT_SHL, BIT_SHR, BIT_USHR | Bitwise shifts | High |
| POW | Exponentiation | High |
| XOR, EQV | Logical XOR/EQV | Medium |
| RANGE | Create range object | High |
| BIF_REF | Reference to built-in function | Medium |
| BUFFER_WRITE | Write to output buffer | High |
| RETHROW | Re-throw caught exception | Medium |
| INCLUDE | Load + execute another file | Medium |
| PARAM | Conditional variable default | Medium |
| ASSERT | Assert with message | Medium |

---

## 4. Runtime BIFs Needed

| BIF | Purpose |
|---|---|
| `castAs(value, type)` | Type casting |
| `instanceOf(value, type)` | Type checking |
| `contains(collection, item)` | Containment check |

---

## 5. Implementation Phases

### Phase 1: Critical VM Ops (High priority)
Bitwise operators, power, ranges, buffer output. These are core language features that scripts will use.

### Phase 2: Statement Semantics (Medium)
assert, param, rethrow, include — complete the runtime behavior.

### Phase 3: Template Execution (Medium)
BufferOutput → BUFFER_WRITE, script island compilation, `<bx:loop>` proper compilation, `#expr#` interpolation.

### Phase 4: Advanced Features (Low)
Spread desugaring, full destructuring, access modifier enforcement, functional BIF refs, generic components.

---

## 6. Out of Scope

- SQL Query-of-Queries (needs full SQL parser + runtime)
- CFML compatibility
- Template component caching
- JIT compilation of new opcodes
- Source maps for template output

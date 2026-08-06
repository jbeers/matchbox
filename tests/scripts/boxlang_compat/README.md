# BoxLang Compatibility Transfer

Every BoxLang JVM test (`../BoxLang/src/test/java/ortus/boxlang/...`) has a
1:1 black-box counterpart as a MatchBox `.bxs` script in this directory.
`tests/boxlang_compat_tests.rs` registers and executes them all.

This is **transfer-first**. Getting the equivalent behavior green is a separate
follow-up task that uses only MatchBox/Rust idiomatic solutions — NOT a rewrite
of VM internals to mirror the JVM.

## Mapping conventions

- Directory structure mirrors `../BoxLang/src/test/java/ortus/boxlang/...`.
- One `.bxs` per BoxLang test file.
- Each JVM `@Test` method becomes a block using `assert <expr> : "<TestName>"`.
- Every file header records the JVM test it ties back to.

## Categorizing tests

Only tests that capture BoxLang runtime behavior are transferred.

- BoxLang source evaluated directly → transferred verbatim.
- Unit tests focused on Java behavior or internal BoxLang behavior → skipped,
  with the reason recorded in the file header and in the registry below.

## Status per category

| Category | Path | Status |
| :--- | :--- | :--- |
| operators | `runtime/operators/` | transferred (38 files) |
| scopes | `runtime/scopes/` | transferred (6 files) |
| casters/dynamic | `runtime/dynamic/` | transferred (25 files) |
| types | `runtime/types/` | transferred (26 files) |
| bifs/* | `runtime/bifs/global/` | transferred (477 files across all BIF categories) |

## Running

```bash
cargo test --test boxlang_compat_tests
```

Every transferred test is registered and RUNS. Tests whose behavior MatchBox
does not implement yet **fail (red)** — that is the signal the follow-up
compatibility work consumes. No transferred test is skipped or ignored. Only
tests that cannot be expressed in BoxLang at all (JVM-only infrastructure) are
skipped, and those are documented in the `.bxs` header and the registry below.

## Skipped tests (hard requirements, non-JVM)

A test is skipped only when it cannot be expressed in MatchBox (JVM-only
infrastructure such as Java reflection, SOAP wire format, JDBC driver
internals). Functionality that CAN be expressed (e.g. configuring a
datasource + running queries) is transferred. The skip reason is documented
in the corresponding `.bxs` file header.

### runtime/operators

| JVM test | Reason |
| :--- | :--- |
| `CastAsTest.java` | asserts Java result class via `.getClass().getName()` (reflection) |
| `InstanceOfTest.java` | Java Class objects, Java collections and BoxLang class files (expressible `isInstanceOf` subset is transferred) |

## Transferred, currently failing (red)

These files contain faithful transfers that currently fail because MatchBox
does not implement the behavior yet. The `.bxs` header notes the gap.

- `==` compares int results of the specialized `ADD_INT` / `SUB_INT` /
  `MULTIPLY_INT` opcodes against numeric literals as unequal.
- String truthiness: JVM treats `"false"` and `"no"` as falsy; MatchBox treats
  every non-empty string as truthy (`AndTest`, `NotTest`, `OrTest`,
  `TernaryTest`).
- String `<` / `>` / `<=` / `>=` ordering is case-sensitive in MatchBox; JVM
  BoxLang is case-insensitive (`CompareTest`, `LessThanEqualTest`).
- `contains` is case-sensitive in MatchBox (`ContainsTest`).
- Numeric-string equality/ordering not implemented (`"1.5" == "1.500"`,
  `EqualsEqualsTest`, `GreaterThanEqualTest`).
- The `-`, `/`, `%`, `^` and unary `-` operators do not coerce numeric strings
  (`MinusTest` minus path, `DivideTest`, `ModulusTest`, `PowerTest`,
  `NegateTest`, `DecrementTest`).
- Operators whose files fail to compile or panic, so they show red:
  `b~` complement not parsed (`BitwiseComplementTest`, part of
  `BitwiseOperatorsTest`), `b>>>` on negatives not implemented
  (`BitwiseUnsignedRightShiftTest`), negative `b>>` shift counts panic
  (`BitwiseSignedRightShiftTest`), `===` strict equality not parsed
  (`EqualsEqualsEqualsTest`), `\` integer divide not parsed
  (`IntegerDivideTest`), `.toSet()` / Set operators not implemented
  (`SetOperatorsTest`), `isInstanceOf` BIF missing (`InstanceOfTest`).
- Value casters transferred via `cast( value, "type" )` — MatchBox has no
  `cast` BIF yet (`BooleanCasterTest`, `NumberCasterTest`, `StringCasterTest`,
  `DoubleCasterTest`).
- Array behavior (`runtime/types/ArrayTest.bxs`): negative indexing and
  out-of-bounds / non-integer reads return null instead of throwing.

## Skipped per category (JVM-only / internal-BoxLang, hard requirements)

Skipped files still exist 1:1 under the category path; their header records the
reason. These tests drive internal Java type/scope/caster APIs directly with no
BoxLang source, so there is nothing to transfer as a black-box script.

- **scopes (6):** ArgumentsScopeTest, BaseScopeTest, IntKeyTest, KeyTest,
  ScopeWrapperTest, ServerScopeTest — internal scope/Key mechanics.
- **dynamic (21):** AttemptTest, ExpressionInterpreterTest,
  FunctionalInterfaceTest, ReferencerTest, and the caster unit tests for Java
  collections/iterables/java.time/Query/XML/exceptions/Key/StringBuilder
  (ArrayCasterTest, AssignableArrayCasterTest, BigIntegerCasterTest,
  CollectionCasterTest, DateTimeCasterTest, FunctionCasterTest,
  IterableCasterTest, KeyCasterTest, ModifiableArrayCasterTest,
  ModifiableStructCasterTest, QueryCasterTest, StringBuilderCasterTest,
  StructCasterTest, ThrowableCasterTest, TimeCasterTest, VariableNameCasterTest,
  XMLCasterTest).
- **types (25):** ArgumentTest, BoxSetTest, BoxStringBuilderTest,
  ChunkedArrayListTest, ClosureTest, DateTimeTest, DelimitedArrayTest,
  DynamicFunctionTest, FileTest, FunctionTest, LambdaTest, QueryColumnTypeTest,
  QueryTest, StructTest, XMLTest, `meta/*` (4), `unmodifiable/*` (2),
  `util/*` (4). Revisit when the underlying type lands in MatchBox and becomes
  observable from BoxLang source.

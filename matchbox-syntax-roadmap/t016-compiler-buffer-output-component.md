# T016: Compiler — BufferOutput and Component Invocation

**Type:** AFK  
**Blocked by:** T015 (Generic components + wire up)

## What to build

Implement bytecode generation for template-specific AST nodes: `BufferOutput`, `Component`, `ScriptIsland`, `TemplateIsland`.

### BufferOutput compilation
- Compile the expression (string literal or StringInterpolation)
- Emit bytecode that writes to the output buffer
- The VM needs an output buffer or a `writeOutput()` built-in
- For now, use `writeOutput()` function call if available, or emit a `PRINT` opcode

### Component compilation
- `StatementKind::Component { name, attributes, body }`
- For known components (output, set, etc.), compile to dedicated bytecode
- For generic components, emit a runtime component invocation
- Component attributes are compiled as expression evaluations

### ScriptIsland compilation
- Statements are compiled inline in the current scope
- No special wrapping needed — they're just regular script statements

### TemplateIsland compilation
- Same as ScriptIsland — compile inline
- Template statements are already desugared to regular AST nodes

### Output buffer
- The VM needs an `output_buffer` field or a `writeToBuffer(value)` mechanism
- At minimum, `BufferOutput` should emit text that can be captured
- For the web server, the output buffer becomes the HTTP response body

## Delivery

Changes: `crates/matchbox-compiler/src/compiler/mod.rs`
May need: `crates/matchbox-vm/src/vm/mod.rs` (output buffer support)

## Acceptance criteria

- [ ] `BufferOutput("hello")` compiles to bytecode that produces "hello"
- [ ] `BufferOutput` with `StringInterpolation` concatenates parts correctly
- [ ] `Component` with known name compiles to appropriate bytecode
- [ ] `ScriptIsland` statements execute in template scope
- [ ] Template output can be captured (for HTTP response)
- [ ] Integration test: template renders to string output

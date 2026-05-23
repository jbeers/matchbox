# C006: Add Output Buffer Support for Templates

**Type:** AFK  
**Blocked by:** None

## What to build

Add `BUFFER_WRITE` VM opcode and use it for template `BufferOutput` statements instead of `PRINT` (stdout).

### VM opcode
`BUFFER_WRITE` (79) — pops value from stack. If `vm.output_buffer` is `Some(ref mut buf)`, converts value to string and appends to buffer. If `output_buffer` is `None`, writes to stdout (via PRINT).

### Compiler changes
Change `BufferOutput` compilation from `PRINT` to `BUFFER_WRITE`. This ensures template output is captured in the buffer rather than printed to stdout.

### Test
```
var vm = VM::new();
vm.output_buffer = Some(String::new());

// Compile and run a template that outputs text
// Verify output_buffer contains the expected text
```

## Acceptance criteria
- [ ] BUFFER_WRITE opcode appends to output_buffer when set
- [ ] BUFFER_WRITE falls back to stdout when output_buffer is None
- [ ] Template BufferOutput statements use BUFFER_WRITE
- [ ] Web server template rendering captures output correctly
- [ ] Integration test passes

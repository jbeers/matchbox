# N029: ESP32 Route Execution Without Per-Request Chunk Clone

**Type:** ESP32 runtime / VM memory
**Priority:** High

## What to build

Make embedded ESP32 web route execution avoid cloning the full route `Chunk` on
every request.

The current ESP32 route path was intended to use borrowed route bytecode, but it
still clones the route chunk before execution:

```rust
VM::interpret_chunk_borrowed_current_task
  -> Chunk::clone_without_runtime_caches
  -> constants.clone()
```

On ESP32-S3 web + Wi-Fi + USB HID builds, heap after web startup can be around
50 KB free with a largest internal block around 31 KB. A route with enough
literal constants to describe a non-trivial drawing can OOM before the first
BoxLang statement executes.

The goal is for preloaded embedded routes to execute from their already-loaded
chunk data without duplicating `code`, `constants`, `lines`, `filename`, or
`source` per request.

## Why

The mouse-jiggler demo needs a `/draw` route that can drive a USB HID mouse to
draw the BoxLang logo. A generated route with SVG-derived movement instructions
currently crashes before execution due to route chunk cloning, not HID behavior.

Fresh coredump evidence from the demo device:

```text
Crashed task: httpd
Panic reason: abort()
Root cause: Rust OOM

std::alloc::rust_oom
alloc::raw_vec::handle_error
Vec<matchbox_vm::types::Constant>::clone
matchbox_vm::vm::chunk::Chunk::clone_without_runtime_caches
matchbox_vm::vm::VM::interpret_chunk_borrowed_current_task
matchbox_esp32_runner::web::execute_embedded_route
```

Tiny routes such as square drawing work because their constant tables are small.
Non-trivial but still reasonable ESP web routes should not fail simply because
the VM duplicates immutable bytecode and constants for each request.

## Design constraints

- Preserve mutable runtime caches safely. Inline caches must not introduce data
  races or request-to-request corruption.
- Avoid broad app-specific shortcuts such as native BIFs for one demo drawing.
  The fix should make the MatchBox ESP runtime capable of richer routes.
- Keep allocation pressure low in the HTTP task. ESP route startup should not
  require allocating another copy of the route program.
- Maintain existing host behavior unless the same borrowed execution path is
  useful and covered by tests.
- Be careful with function literals and closures: `BxCompiledFunction` currently
  owns a `Chunk`, so any no-clone design must account for script functions and
  nested compiled functions without reintroducing large per-request clones.

## Candidate implementation directions

1. **Shared immutable chunk data plus separate runtime caches**
   Split `Chunk` into immutable program data and mutable runtime execution
   state. Route tables can hold `Arc<ChunkProgram>`, while each execution gets
   only the lightweight cache/state it must mutate.

2. **Borrowed/root chunk frame for ESP routes**
   Add an execution path where the root call frame references the preloaded
   route chunk directly and keeps mutable per-run state outside the chunk. This
   is smaller than a full chunk split but may require more lifetime/refactor work
   around `CallFrame`.

3. **ESP-specific clone profile**
   As an interim step, avoid cloning `source`, `filename`, or other debug-only
   fields on ESP32. This may reduce pressure, but it does not solve the
   `constants.clone()` failure for large literal tables.

The preferred fix is option 1 or 2. Option 3 is only a fallback if it is paired
with a follow-up that removes `constants.clone()` from the ESP route path.

## Acceptance criteria

- [ ] Embedded ESP32 route execution does not clone the full `Chunk` per request
- [ ] `code`, `constants`, `lines`, `filename`, and `source` are not duplicated
      when an ESP route starts
- [ ] Runtime caches remain correct across repeated requests to the same route
- [ ] A route with many numeric literals can start and execute on ESP32 without
      OOMing in `Chunk::clone_without_runtime_caches`
- [ ] The mouse-jiggler `/draw` route can remain implemented in BoxLang rather
      than moving logo drawing into a native app-specific BIF
- [ ] Add a regression test or ESP smoke fixture that exercises a route with a
      large constant table
- [ ] Document the ESP memory motivation in the relevant runner or VM comments

## Verification notes

Use the mouse-jiggler app as the hardware reproduction case:

1. Flash the ESP32-S3 web/HID runner.
2. Connect to the device AP and load the app.
3. Register USB HID.
4. Call `/draw` with a route containing a non-trivial movement program.
5. Confirm no coredump is produced and the route reaches HID movement code.

Before the fix, the coredump should show OOM in
`Chunk::clone_without_runtime_caches`. After the fix, failures in this area
should move to ordinary BoxLang route errors or HID errors, not route startup
allocation.

## Notes

- The current ESP route table already stores executable route chunks in memory
  as `Arc<Chunk>`.
- The misleading method name `interpret_chunk_borrowed_current_task` should be
  revisited; it currently borrows at the API boundary but clones internally.
- This issue is not asking for a more compact SVG/logo format. Compact route
  data may still be useful, but the platform should not require it just to avoid
  duplicating immutable route bytecode at request time.

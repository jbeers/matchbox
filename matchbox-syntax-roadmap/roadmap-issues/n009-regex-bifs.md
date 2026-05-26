# N009: Regex BIFs

**Type:** Runtime
**Priority:** Medium

## What to build

Round out the regex helpers beyond the current `reMatch()` support.

## Completed

- `reMatch()` and `reMatchNoCase()` return the expected match arrays
- `reFind()` and `reFindNoCase()` return positions and `len`/`match`/`pos` structures
- `reReplace()` and `reReplaceNoCase()` support `one` and `all` scope
- String member forms are wired for the new regex helpers

## Acceptance criteria

- [x] Case-sensitive and case-insensitive match helpers work
- [x] Find helpers return position/length/result structures correctly
- [x] Replace helpers support one/all scope

# N010: JSON BIFs

**Type:** Runtime
**Priority:** Medium

## What to build

Implement stable JSON encode/decode helpers for BoxLang values.

## Completed

- `serializeJSON()` and `jsonSerialize()` convert arrays, structs, scalars, and null to JSON
- `deserializeJSON()` and `jsonDeserialize()` reconstruct arrays, structs, scalars, and null
- `isJSON()` validates JSON input
- `toJSON()` and `fromJSON()` member aliases work on string and collection receivers

## Acceptance criteria

- [x] `serializeJSON()` handles structs, arrays, scalars, and null
- [x] `deserializeJSON()` reconstructs BoxLang values correctly
- [x] `isJSON()` validates JSON input

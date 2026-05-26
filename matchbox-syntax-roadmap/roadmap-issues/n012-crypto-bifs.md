# N012: Crypto BIFs

**Type:** Runtime
**Priority:** Low

## What to build

Extend the current crypto coverage beyond the default hash path.

## Acceptance criteria

- [x] `hash()` supports the listed algorithms
- [x] `hmac()` works
- [x] Unsupported encrypt/decrypt features stay explicitly deferred

## Completed

Implemented the crypto helper surface that is in scope for this roadmap slice:

- `hash()` supports MD5, SHA-1, SHA-224, SHA-256, SHA-384, SHA-512, QUICK, and `bxmX_COMPAT`
- `hash()` accepts byte values, string values, and iterated hashing
- `hmac()` works for MD5, SHA-1, SHA-224, SHA-256, SHA-384, and SHA-512
- `hash()` and `hmac()` are available as string member methods
- symmetric `encrypt()` / `decrypt()` and key generation remain deferred outside this ticket

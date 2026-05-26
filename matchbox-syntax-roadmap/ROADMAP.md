# MatchBox Roadmap From The Current Checkpoint

This file replaces the old planning view for active work. The review backlog
(`review-issues/`) is complete and archived. The active backlog now starts here.

## What Is Done

- Parser review backlog `R001-R012` is complete.
- CST foundation work is complete:
  - lossless script and template CST
  - stable node ids and traversal helpers
  - typed statement nodes
  - shallow expression nodes
  - interpolation and script-island template nodes
  - edge trivia helpers for formatter consumers
- The runtime/compiler fixes from the review backlog are complete.

## What Is Still Active

### Phase 1: Core Runtime Parity

- `C005` Include runtime
- `C010` Access modifier enforcement

### Phase 2: Core BIF Coverage

- `D001` DateTime type and core date/time BIFs
- `D002` Array BIFs
- `D003` Struct BIFs
- `D004` String BIFs
- `D005` List BIFs
- `D006` Math BIFs
- `D007` Regex BIFs
- `D008` JSON BIFs
- `D009` Utility BIFs
- `D010` Crypto BIFs

### Phase 3: Query-of-Queries

QoQ is planned as an optional feature flag (`qoq`) so streamlined deployments
can leave it out entirely.

- `Q001` SQL lexer
- `Q002` SQL parser
- `Q003` SQL AST
- `Q004` QoQ execution engine
- `Q005` QoQ BIF integration

### Phase 4: Database And Web Platform

- `W001` MySQL/MariaDB driver
- `W002` SQLite driver
- `W003` Transaction support
- `W004` Web scopes
- `W005` Application lifecycle
- `W006` File upload

## Not Yet Scheduled

The long-term subsystem parity items from `FINAL-PRD.md` remain deferred:
XML, email, PDF, spreadsheet, image processing, scheduling, caching, logging,
i18n, validation, interceptors, LDAP, and charting.

## Active Backlog

Use [roadmap-issues/README.md](roadmap-issues/README.md) as the working index.

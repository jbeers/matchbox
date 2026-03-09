# Basic BoxLang Project

This example shows a minimal multi-file BoxLang project with a script entry point
and model classes split across separate files — the most common project structure
for a BoxLang application.

## Project Layout

```
basic_project/
├── main.bxs            ← entry point (run this)
└── models/
    ├── ISpeakable.bxs  ← interface definition
    ├── Animal.bxs      ← base class
    └── Dog.bxs         ← subclass (extends Animal, implements ISpeakable)
```

## Running

Run the project directly with the MatchBox interpreter from this directory:

```bash
matchbox main.bxs
```

Expected output:

```
=== MatchBox Basic Project ===

--- Descriptions ---
I am Whiskers and I make the sound 'Meow'
I am Rex and I make the sound 'Woof'

--- Speak ---
Meow!
Rex barks: Woof! Woof!

--- Dog Only ---
Rex fetches the ball!
Rex fetches the frisbee!

--- Roster ---
  Whiskers — Cat
  Rex — Dog

All done!
```

## Compile to Bytecode

Compile to a portable `.bxb` bytecode file for faster repeated execution:

```bash
matchbox --build main.bxs
matchbox main.bxb
```

## Compile to a Native Binary

Produce a single self-contained executable (~500 KB, no runtime required):

```bash
matchbox --target native main.bxs
./main
```

## Key Concepts

| Concept | Where to look |
|---|---|
| Class definition | `models/Animal.bxs` |
| Inheritance | `models/Dog.bxs` (`extends Animal`) |
| Interface | `models/ISpeakable.bxs` |
| Multi-file imports | `main.bxs` (`import models.Animal`) |
| Struct literals | `main.bxs` (roster array) |
| For-in loop | `main.bxs` (roster iteration) |

## How Multi-File Imports Work

MatchBox resolves imports using dot-separated paths relative to the directory
where `matchbox` is invoked. `import models.Animal` maps to `models/Animal.bxs`
relative to the current working directory.

Always run `matchbox main.bxs` from the `basic_project/` directory so that the
`models.*` imports resolve correctly.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Capybara-Lang is an experimental programming language written in Rust with first-class support for Korean (Hangul) identifiers and strings. Programs are written in `.bara` files and executed by the interpreter. Only dependency is `serde_json` for native JSON value support.

## Commands

### Build & Run
```bash
cargo build                                    # debug build → target/debug/capybara-lang
cargo build --release                          # optimized build
cargo check                                    # fast type/syntax check without linking
./target/debug/capybara-lang <file.bara>       # run a .bara program
```

### Testing
There is no automated test suite. Tests are run by executing the `.bara` files in the repository root:
```bash
cargo build && ./target/debug/capybara-lang test_comprehensive.bara
./target/debug/capybara-lang test_conditionals.bara
./target/debug/capybara-lang test_booleans.bara
```

### Documentation site (Docusaurus)
```bash
cd docs && npm ci          # install deps
npm run build              # build static site → docs/build/
npm start                  # local dev server
```

## Architecture

The entire interpreter lives in `src/main.rs` (~928 lines) as a classic three-stage pipeline:

```
.bara source  →  Lexer  →  Vec<Token>  →  Parser  →  Vec<Statement>  →  Interpreter
```

**Lexer** (`struct Lexer`, line 116): character-by-character scan. Key detail: `<-` is a single `BlockStart` token and `->` is a single `BlockEnd` token — they are consumed together, so angle-bracket logic must look ahead one character. The `IsHangul` trait (line 369) extends `char` to detect Korean Unicode blocks (AC00–D7AF, 1100–11FF, 3130–318F), allowing Hangul identifiers.

**Parser** (`struct Parser`, line 385): hand-written recursive-descent. Entry point is `parse()` → `parse_statement()`. The two variable-declaration forms have different token shapes:
- `set` (untyped): `set ; [ "name" ] : <expr>`
- `var` (typed):   `var ; [ "name" ] : < type > ; < ( value ) >`

`if`/`fi`/`el` (if/elif/else) are parsed in `parse_if_statement()`. Blocks are delimited by `BlockStart`/`BlockEnd` tokens. The only binary operator is `=` (equality comparison).

**Value system** (`enum Value`, line 54): `String`, `Number(f64)`, `Boolean`, `Json(serde_json::Value)`, `Null`. JSON is a native first-class type parsed directly into `serde_json::Value` at parse time.

**Environment** (`struct Environment`, line 738): a flat `HashMap<String, Value>` — there is no scoping; all variables are global.

**Interpreter** (`struct Interpreter`, line 758): walks the AST. Holds a reference to `Environment` and `last_input: Option<Value>` for the `pop` built-in. Built-in functions: `print`, `input`, `pop`.

## Capybara Language Syntax Reference

| Construct | Syntax |
|-----------|--------|
| Comment (inline) | `` `comment text` `` |
| Comment (multi-line) | ` ```comment``` ` |
| Untyped variable | `set;["name"]:(value)` |
| Typed variable | `var;["name"]:<str>;<("value")>` |
| Print | `<print:(value)>` |
| Input + prompt | `<input;print:("prompt")>` |
| Last input | `<pop>` |
| If | `if {condition} <- body ->` |
| Elif | `fi {condition} <- body ->` |
| Else | `el {} <- body ->` |
| Equality | `x = y` (inside `{}` condition or expression) |
| JSON literal | `({"key": "value"})` |
| Boolean | `(true)` / `(false)` — case-insensitive |

Variable names and string contents may use Korean characters. The `=` token is equality comparison only (no assignment operator exists separately from the declaration syntax).

## Known Limitations / Active Gaps

- `read_json()` method on `Lexer` is implemented but never called — JSON in source is parsed by the parser via `serde_json`, not the lexer.
- `Parser::peek()` is defined but unused.
- No loops, no user-defined functions, no scoping.
- Only one operator: `=` (equality). No arithmetic, no string concatenation.
- Type annotations (`var;["x"]:<str>`) are parsed and stored but not enforced at runtime.

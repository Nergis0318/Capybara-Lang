# AGENTS.md

This file provides guidance to OpenCode when working with code in this repository.

## Project Overview

Capybara-Lang is an experimental programming language interpreter written in Rust. The entire interpreter lives in a single file: `src/main.rs`. Only dependency is `serde_json`.

## Commands

```bash
cargo build                              # debug build → target/debug/capybara-lang
cargo check                              # fast type/syntax check without linking
./target/debug/capybara-lang <file.bara> # run a .bara program
```

## Testing

**There is no automated test suite (`cargo test` does nothing).** Tests are `.bara` files in the repo root executed manually:

```bash
cargo build && ./target/debug/capybara-lang test_comprehensive.bara
./target/debug/capybara-lang test_conditionals.bara
```

After any change to the interpreter, run at least `test_comprehensive.bara` to verify nothing broke.

## Architecture

```
.bara source → Lexer → Vec<Token> → Parser → Vec<Statement> → Interpreter
```

All three stages are in `src/main.rs`. There is no `lib.rs`, no modules, no test directory.

### Key implementation details

- **Lexer**: `<-` and `->` are single tokens (`BlockStart`/`BlockEnd`), consumed via 2-character lookahead. Angle-bracket logic must `peek(1)` ahead.
- **Parser**: hand-written recursive-descent with precedence climbing. Entry point: `parse()` → `parse_statement()`.
- **Environment**: stack of `HashMap<String, Value>` scopes. `if`/`fi`/`el`/`wh` blocks each create their own scope.
- **Interpreter**: walks AST directly. Built-in functions: `print`, `input`, `pop`. No user-defined functions exist.

### Dead code (do not rely on)

- `Lexer::read_json()` — implemented but never called. JSON literals are parsed by the Parser via `serde_json::from_str`, not the Lexer.
- `Parser::peek()` — defined but unused.

### Known bugs / failing tests

- **JSON literals are broken**: `test_json.bara`, `test_simple_json.bara`, and `test_comprehensive.bara` fail with `Unexpected token: LeftBrace`. The lexer emits `Token::LeftBrace` for `{` but the parser does not handle it — `read_json()` should be called instead when `{` follows `(`.
- **`test_variables.bara`** has a missing closing `>` on line 10 — the file itself is malformed.
- Run `test_conditionals.bara`, `test_loops.bara`, `test_booleans.bara`, and `test_arithmetic.bara` for reliable verification.

## Language Syntax (non-obvious)

| Gotcha | Detail |
|--------|--------|
| Equality operator | `=` (not `==`) |
| Boolean literals | `(true)` / `(false)` — **case-insensitive**: `True`, `FALSE`, `tRuE` all work |
| Assignment/declaration | `set` is both declaration and reassignment; there is no separate assignment operator |
| Typed vars | `var;["name"]:<type>;<(value)>` — type is parsed and stored but **never enforced at runtime** |
| Block delimiters | `<-` opens, `->` closes (NOT `<` and `>` individually) |
| Condition wrapping | Conditions use braces: `if {x = 5} <-` (not `<` and `>`) |
| Input syntax | `<input;print:("prompt")>` — the `print` keyword inside input is part of the syntax |
| Variable names | Korean (Hangul) identifiers are allowed via `IsHangul` trait |

## Documentation site

```bash
cd docs && npm ci   # install deps
npm start           # local dev server (Docusaurus)
```

CI auto-deploys `docs/` to GitHub Pages on pushes to `main` that touch files under `docs/`.

## References

- `CLAUDE.md` — same content, targeted at Claude Code
- `SYNTAX.md` — Korean-language syntax reference with examples

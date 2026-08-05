# lumesh Recent Release Update (0.16.13 → 0.17.5)

---

Following up on the previous report (0.14.x–0.16.12: editor rewrite, tokenizer refactor, History/Slash Commands, AI integration), this release shifts focus to the core language layer — a **deep refactor of the string/quoting system**, the **introduction of the `Bytes` type**, the **establishment of a background job management system**, and a series of performance refinements throughout.

---

### Strings and Quote Semantics: The Core Change of This Release

This is the heaviest-weight set of changes in this release, spanning three versions (0.16.13 → 0.17.0 → 0.17.5) of continuous evolution, ultimately converging into a complete and self-consistent escaping semantics system:

- **0.16.13**: `rewrite unescape for all string` — rewrote the unescaping logic for all string types, paving the way for the subsequent fine-grained differentiation of quote semantics.
- **0.17.0**: **Introduced `StringSafe`** (`s'...'`), purpose-built for command-concatenation scenarios, eliminating injection risk at the language level:
  ```bash
  let s = s'(rm -rf /)'
  let s2 = into.safe 'mkfs /dev/sda'
  eval_str(`echo $s $s2`)  # never unsafe eval
  ```
  In the same release, **`SymbolRaw` was introduced**, extending the applicability of the `^` escape character from command position to argument position as well.
- **0.17.5**: **Introduced hashed raw quotes** (the `#`-delimited syntax), formally establishing a multi-tier escaping gradient:
  ```rust
  r#'....'#        // fully unescaped
  r##'....'##      // supports any number of #'s (for content that itself contains #)

  r#"...."#        // quotes are not escaped, but ansi and unicode sequences are
  t#"...."#        // no escaping for the g/t/s/b prefixes
  ```
  **Breaking change**: `r'....'` no longer serves as the regex prefix — it is now narrowed to a purely **raw string** semantic; the former **regex prefix has been officially renamed to `g'....'`**.

  This design cleverly uses the `#` delimiter to resolve the inherent conflict between "boundary character vs. quote characters appearing in content," while decoupling "whether to escape ANSI/unicode" from "whether to escape quotes" into two independent, composable dimensions — ultimately producing a clean semantic gradient: from `"..."` (fully escaped), through `r#"..."#` (partially escaped — quotes left raw, ANSI/unicode still escaped), to `r#'...'#` (fully unescaped), each with a clearly defined role.

---

### `Bytes` Type Introduced (0.17.0)

- Literal support: `b'\x41'`
- Supports slicing, comparison, and operator overloading
- Supports printing, piping, and use as command arguments
- **0.17.5**: Syntax further expanded with `b"..."` (double-quote form) and `b#"..."#` (`#`-delimited form), keeping the writing style consistent with the escaping semantics of the string system.

---

### Numeric Literal Enhancements (0.17.0)

- Introduced **radix numbers**: `0b100_100`, `0o170`, `0xff`
- All numeric types now uniformly support `_` separators: `999_999`, `999_999.999_999`
- **0.17.5**: Further optimized radix number parsing performance.

---

### Background Jobs and Terminal Integration (from 0.17.3)

- **0.17.3**: Added the `jobs` command for unified management of background tasks.
- **0.17.5**:
  - Performance and stability improvements related to PTY (pseudo-terminal)
  - Fixed leftover screen artifacts when piping output to `vi`
  - Added `Ctrl+Z` support for non-PTY scenarios

---

### Syntax/Tokenizer Refinements (0.17.3)

- Adjusted the categorization of custom unary operators
- Improved tokenization precision for `.`, `^`, `+`
- Removed the index operator `@`
- Unified tokenization logic for `-`/`!` prefixes
- Improved `<<`, adding byte-stream support
- **`?&` renamed to `&:`**, continuing the short-circuit logic writing style introduced in 0.16.2
- **`let` disables CFM by default**: prevents statements like `let a=1` from being misparsed as commands under CFM, while preserving genuine CFM usage such as `dd if=/dev/sda`
- Improved whitespace recognition around `^` and `:`
- Fixed `symof`: `assert(symof(1+2), 'BinaryOp')`

---

### Standard Library Enhancements (0.17.3)

- `fs.ls` supports specifying files directly (`fs.ls file(s)`)
- Built-in libraries now support wildcard expansion

### Prompt Integration (0.17.3)
- Integrated with starship, improving shell prompt customizability

---

### Release Cadence Summary

| Phase | Core Focus |
|------|----------|
| 0.16.13 | Full rewrite of string escaping logic; CFM/custom operator fixes |
| 0.17.0 | `Bytes` type, `StringSafe`/`SymbolRaw`, radix numbers, function renames (Breaking Change) |
| 0.17.1–0.17.2 | Library function/hint optimizations, `cfm auto` mode, `&:` short-circuit syntax |
| 0.17.3 | `jobs` background task management, starship integration, tokenizer refinements (Breaking Change: `?&` → `&:`) |
| 0.17.5 | **Hashed raw quote system landed** (`r#'..'#`/`r#"..."#`/`g#'..'#`, etc.), PTY/signal handling optimizations (**Breaking Change: `r'..'` narrowed to raw string; regex prefix changed to `g'..'`**) |

---

# lumesh Recent Updates (0.14.x → 0.16.x)

### Complete Editor Rewrite (0.15.0)

Version 0.15.0 was a major milestone: **migrated from rustyline to crossterm, editor rewritten from scratch**. Subsequent versions continued refinement:

- **0.15.1**: Editor themes, buffer optimization, custom hotkey bindings
- **0.15.2**: Multi-line editing mode, input validation, cursor and hint position fixes
- **0.16.0**: Fixed ONLCR issue, path completion sorting optimization
- **0.16.2**: Added `ui.editor`, `ui.date_pick`, fixed CapsLock recognition, hotkey modifier mapping, and other bugs
- **0.16.10**: Editor scrolling support

---

### Tokenizer Refactoring (0.15.5 onwards)

- **0.15.5**: **Restructured tokenizer with dispatch mechanism**, dispatching token parsing by priority for clearer and more maintainable parsing logic
- **0.15.6**: Improved CFM (Command First Mode) symbol handling, unified highlighting logic
- **0.15.7**: Fixed tokenizer handling of trailing `&`
- **0.16.2**: Fixed module call tokenizer
- **0.16.7**: CFM mode enhanced to take whole words (avoiding misparsing `1.1.1.1` as float)
- **0.16.8**: Switched to static regex for improved tokenization performance

---

### History System Evolution

- **0.16.5**: Added history hint, ESC moves to end in multiline mode and clears hint
- **0.16.8**: **Introduced slash commands** (`/h`, `/hh`, `/hm` and other slash command system)
- **0.16.9**: Smarter history weighting and sorting, added `/q` quick exit command
- **0.16.10**:
  - Optimized `Ctrl+R` long history display
  - Multi-line commands automatically ignored from `/h...` history (avoiding screen clutter)
  - `/h`, `/hh`, `/hm`, `/history` support prefix filtering

---

### Completion System Enhancements (0.15.3–0.16.3)

- Path completion, external command completion, parameter-aware completion progressively improved
- Support for lumesh scripts as completion data sources
- Completion colors and context awareness
- `ui.pick`/`ui.multi_pick` support `table`/`map` type input
- `ui.float` supports custom decimal places

---

### AI Integration Deepening

- **0.15.9**: `ALT+i` triggers AI hint
- **0.16.0**: `ALT+Enter` / `ALT+o` triggers AI generation
- **0.16.3**: ai-tls enabled by default, ai-https separated as optional feature
- **0.16.8**: Updated AI skill configuration
- **0.16.10**: Updated AI documentation

---

### Language Features

- **0.14.0**: Introduced `table` expression and built-in `table` library
- **0.14.3**: Improved quote semantics (`''` raw string / `""` normal escape / ` `` ` full escape + variable interpolation)
- **0.16.5**: Added `continue` statement; `match` arrow supports line breaks; **Breaking change: only allow single value in declarations**
- **0.16.7**: `~` auto-expands in symbols, normal mode supports prefix matching

---

### Standard Library Restructuring (0.16.10)

Module responsibilities reorganized:

| Change | Description |
|--------|-------------|
| `fs.dirs` → `sys` | Directory-related functions moved to system library |
| `sys.print_tty` / `sys.discard` → `console` | Terminal output control moved to console library |
| Delete `sys.cds` | Cleaned up redundant interfaces |
| Fix float file size display | `float filesize` formatting fix |

---

### Phase Summary

| Phase | Core Direction |
|-------|----------------|
| 0.14.x | Language feature refinement (table, quotes, bug fixes) |
| 0.15.x | Editor rewrite + tokenizer refactoring + completion system |
| 0.16.x | Slash commands system + History intelligence + AI integration + module reorganization |

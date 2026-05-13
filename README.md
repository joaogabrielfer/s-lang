# SLUR (Stack-based Lang-Uage in Rust)

SLUR is a minimalistic, yet powerful stack-oriented programming language implemented in Rust. It combines traditional concatenative programming with modern features like deep pattern matching, variadic dispatch, and move-inspired variable handling.

## Table of Contents
- [Quick Start](#quick-start)
- [Fundamentals](#fundamentals)
- [Data Types](#data-types)
- [Stack Manipulation](#stack-manipulation)
- [Arithmetic & Logic](#arithmetic--logic)
- [Strings & Lists](#strings--lists)
- [Variables & Functions](#variables--functions)
- [Pattern Matching](#pattern-matching)
- [Control Flow](#control-flow)
- [I/O & Modules](#io--modules)
- [Standard Library](#standard-library)

---

## Quick Start

### Installation
Ensure you have Rust and Cargo installed.
```bash
cargo build --release
```

### Running a Script
```bash
./target/release/slur path/to/script.slur
```

### REPL
Run without arguments to enter the interactive REPL:
```bash
./target/release/slur
```

---

## Fundamentals

SLUR uses **postfix notation**. Operations follow their operands.
- Comments start with `;;` and end with `;;`.
- Most literals (except Booleans) must be explicitly pushed using the `push` command.
- Variables and functions are stored in "elements" memory.

---

## Data Types

| Type | Syntax | Description |
| :--- | :--- | :--- |
| **Integer** | `42`, `-10` | 64-bit signed integers. |
| **Boolean** | `true`, `false` | Boolean literals (auto-pushed). |
| **String** | `"Hello"` | UTF-8 strings. |
| **Char** | N/A | Character type (result of list/string operations). |
| **List** | `[ 1 2 3 ]` | Heterogeneous collections. |
| **Block** | `{ add }` | A collection of delayed tokens. |
| **Function** | `(signature) { block }` | A block with pattern-matching capabilities. |
| **Type** | `@int`, `@any` | Type literals used for matching or conversion. |

---

## Stack Manipulation

The data stack is where most operations happen.

- `push <lit|var>...`: Pushes literals or variable values onto the stack. Variables are **cloned** into the stack.
- `drop`: Removes the top element.
- `dup`: Duplicates the top element.
- `swap`: Swaps the top two elements.
- `over`: Copies the second element to the top.
- `rot`: Rotates the third element to the top.
- `pick <index>`: Copies the `index`-th element from top to the top.
- `roll <index>`: Moves the `index`-th element from top to the top.
- `len`: Pushes the length of the top element (if it's a list).
- `stack-len`: Pushes the current depth of the data stack.
- `clear`: Clears the entire data stack.

---

## Arithmetic & Logic

### Arithmetic
- `add`, `sub`, `mul`, `div`: Standard operations on the top two integers.
- `neg`: Negates the top integer.

### Comparison
- `eq`, `lt`, `gt`: Comparison operators (`a b eq` checks if `a == b`).

### Logic
- `and`, `or`, `not`: Standard Boolean logic.

---

## Strings & Lists

SLUR treats strings and lists similarly for many operations.

- `concat`: Concatenates two lists or strings.
- `cons`: Prepends an element to a list or a char to a string.
- `uncon`: Splits a list/string into `head` and `tail`.
- `at`: Accesses element at index.
- `explode`: Pushes all elements of a list onto the data stack.
- `pack <n>`: Takes `n` elements from the stack and creates a list.
- `first <n>`, `last <n>`: Takes the first or last `n` elements.
- `splitb`: Splits a string once by a pattern, pushing `right`, `left`, and a success flag.

### Conversion
- `int?`, `string?`, `bool?`: Attempts to convert the top element to the respective type, pushing the result and a success flag.

---

## Variables & Functions

### Variables
- `into <name>`: Pops the top value and stores it in `<name>`.
- `take <name>`: Removes `<name>` from memory and pushes its value (Move semantics).
- `delete <name>`: Removes `<name>` from memory.
- `name` (or `#name` or `call name`): 
    - If `name` is a simple value, it is pushed to the stack.
    - If `name` is a function (or list of functions), it is **executed**.

### Functions
Functions are defined using a signature and a block:
```slur
(int int) { add } into my_add
```
- `(signature)`: A list of patterns to match against the stack.
- `when { guard }`: Optional guard block that must return `true`.
- `eval`: Pops a function from the stack and executes it.
- `ret`: Returns early from the current function.

---

## Pattern Matching

Pattern matching is a core feature of SLUR, used in function signatures.

| Pattern | Example | Description |
| :--- | :--- | :--- |
| **Type** | `@int` | Matches a value of the specified type. |
| **Literal** | `0`, `"ok"` | Matches an exact literal value. |
| **Range** | `0..<10` | Matches an integer within the range. |
| **List** | `[ @int @int ]` | Matches a list of specific structure. |
| **Destructuring** | `[ head | tail ]` | Splits list/string during match. |
| **Fallback** | `..` | Matches anything. |
| **Variadic** | `..@int` | Matches zero or more elements of a type. |

**Multiple Dispatch**: Assigning a list of functions to a name allows for overloading:
```slur
[
    (0) { push "Zero" }
    (@int) { push "Not zero" }
] into describe_int
```

---

## Control Flow

- `if { ... }`: Executes block if top is `true`.
- `if { ... } else { ... }`: Conditional branching.
- `quit`: Exits the program.

---

## I/O & Modules

- `include <module>`: Imports a module from the `std/` directory.
- `sys-open`: Opens a file, pushes a file descriptor.
- `sys-close`: Closes a file descriptor.
- `sys-read`: Reads `n` bytes from a descriptor.
- `sys-write`: Writes a value to a descriptor.

Standard descriptors: `0` (stdin), `1` (stdout), `2` (stderr).

---

## Standard Library

Include with `include std`. Key words:
- `print`, `println`: Output to stdout.
- `printf`: Formatted output (e.g., `push 10 "Value: %" call printf`).
- `map`: Apply function to list or stack elements.
- `as-int`, `as-string`: Helper conversion wrappers.
- `empty?`, `null?`: Checks for emptiness or zero-values.
- `dip`, `keep`, `bi`, `tri`: Combinators for stack manipulation.

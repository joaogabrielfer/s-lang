# SLUR
**S**tack based **L**ang-**U**age in **R**ust.

SLUR is a minimalistic, stack-oriented programming language implemented in Rust. It features a simple postfix notation, move-semantics for variables, and a greedy variadic type system.

## Quick Start

### Installation
Ensure you have Rust and Cargo installed.
```bash
cargo build --release
```

### Running a Script
```bash
./target/release/slur examples/test_variadics.slur
```

### REPL
Run without arguments to enter the interactive REPL:
```bash
./target/release/slur
```

---

## Data Types
- **Integer**: 64-bit signed integers (e.g., `42`, `-10`).
- **Boolean**: `true` and `false`.
- **String**: Quoted literals (e.g., `"Hello SLUR"`).
- **Block**: A collection of tokens enclosed in `{ ... }`.
- **Function**: A block combined with a type signature `( ... ) -> ... { ... }`.

---

## Keywords & Commands

### Stack Manipulation
- `push <val1> <val2> ...`: Pushes literals or variables onto the stack. Pushing a variable **moves** it out of memory.
- `pop`: Pops the top element and prints it to stdout. String `\n` is printed as a newline.
- `drop`: Removes the top element from the stack.
- `dup`: Duplicates the top element.
- `swap`: Swaps the top two elements.
- `over`: Copies the second element from the top to the top.
- `rot`: Reverses the entire data stack.
- `pick <index>`: Copies the `index`-th element (from top) to the top.
- `roll <index>`: Moves the `index`-th element (from top) to the top.
- `len`: Pushes the current stack depth.
- `clear`: Clears the data stack.

### Arithmetic
- `add`, `sub`, `mul`, `div`: Standard operations on the top two integers.
- `neg`: Negates the top integer.

### Logic & Comparison
- `eq`, `lt`, `gt`: Comparison operators.
- `and`, `or`, `not`: Boolean logic.

### String & Type Manipulation
- `splitb`: Pops a `pattern` and a `source` string. Splits once, pushing the `right` part then the `left` part, and a `true` flag. If not found, pushes `source` and `false`.
- `as_intb`: Converts string/bool to integer, pushing the result and a success flag.

### Variables & Functions
- `into <name>`: Pops the top value and assigns it to `<name>`.
- `(type1 type2 ..type3)`: Pushes a function signature onto the stack.
- `..type`: Variadic argument. Greedily matches continuous items of `type` from the stack. `..any` matches the whole stack (up to fixed arguments).
- `{ ... }`: If a signature is on top, creates a function. Otherwise, creates a block.
- `eval`: Pops a function or block and executes it.
- `call <name>` (or `#name`): Executes a named function from memory.
- `ret`: Returns early from the current function.

### I/O & Modules
- `include <module>`: Includes a module from the standard library (e.g., `include io`).
- `readline`: Pops `path` (string) and `line_number` (int), pushes the line.
- `readlineb`: Same as `readline` but pushes a boolean success flag.

### Control Flow
- `if { ... }`: Executes block if the top is `true`.
- `if { ... } else { ... }`: Conditional branching.
- `quit`: Exits the program.

---

## Examples

### Variadic Printf (from `std/io.slur`)
```slur
include io
push 10 20 30 "% % %"
call printf
;; Output: 30 20 10 ;;
```

### Defining a Function
```slur
(int int) { 
    add 
    into result
    push "Result is: " pop
    push result pop
} into add_and_print

push 10 20 call add_and_print
```

### Move Semantics
```slur
push 10 into x
push x  ;; 'x' is now moved to the stack and gone from memory ;;
dup into x ;; Put it back into 'x' while keeping a copy on stack ;;
```

---

## Comments
Use double semicolons for comments:
`;; This is a comment ;;`

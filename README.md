# SLUR
**S**tack based **L**ang-**U**age in **R**ust.

SLUR is a minimalistic, stack-oriented programming language implemented in Rust. It features a simple postfix notation, function support, and basic file I/O.

## Quick Start

### Installation
Ensure you have Rust and Cargo installed.
```bash
cargo build --release
```

### Running a Script
```bash
./target/release/slur examples/file.slur
```

### REPL
Run without arguments to enter the interactive REPL:
```bash
./target/release/slur
```

---

## Data Types
- **Integer**: 32-bit signed integers (e.g., `42`, `-10`).
- **Boolean**: `true` and `false`.
- **String**: Quoted literals (e.g., `"Hello SLUR"`).

---

## Keywords & Commands

### Stack Manipulation
- `push <val>`: Pushes a literal or variable onto the stack. (Note: pushing a variable moves it out of memory).
- `pop`: Pops the top element and prints it to stdout.
- `drop`: Removes the top element from the stack.
- `dup`: Duplicates the top element.
- `dup <var>`: Pushes a copy of a variable onto the stack without removing it from memory.
- `swap`: Swaps the top two elements.
- `over`: Copies the second element to the top.
- `rot`: Reverses the entire stack.
- `len`: Pushes the current stack depth.

### Arithmetic
- `add`, `sub`, `mul`, `div`: Standard operations on the top two integers.
- `neg`: Negates the top integer.

### Logic & Comparison
- `eq`, `lt`, `gt`: Comparison operators (works with literals or stack values).
- `and`, `or`, `not`: Boolean logic.

### String Manipulation
- `split`: Pops a `pattern` and a `source` string. Splits once, pushing the `right` part then the `left` part (leaving the start of the string on top).
- `splitb`: Same as `split` but pushes a boolean success flag on top. If the pattern is not found, pushes the original `source` and `false`.

### Variables
- `var <name>`: Declares a variable initialized to `0`.
- `into <name>`: Pops the top value into an existing variable.
- `into var <name>`: Declares and assigns in one step.
- `push <name>`: Moves the value from the variable onto the stack.

### Functions
- `fun <name> { ... }`: Defines a function.
- `call <name>`: Executes a function.
- `ret`: Returns early from a function.

### Control Flow
- `if { ... }`: Executes block if the top is `true`.
- `if { ... } else { ... }`: Conditional branching.
- `quit`: Exits the program.

### I/O & Modules
- `include <module>`: Includes a `.slur` file from the standard library path.
- `pushline`: Pushes a specific line from a file onto the stack (requires `path` and `line_number` on stack).
- `pushlineb`: Similar to `pushline` but pushes a boolean success flag.

---

## Examples

### Factorial
```slur
fun factorial {
    dup gt 1 if {
        dup push 1 swap sub
        call factorial
        mul
    } else {
        drop push 1
    }
}

push 5 call factorial pop
;; Output: 120 ;;
```

### Simple Loop (Recursion)
```slur
fun count_down {
    dup pop push " " pop
    push 1 sub
    dup gt 0 if {
        call count_down
    } else {
        drop
    }
}

push 5 call count_down
;; Output: 5 4 3 2 1 ;;
```

---

## Comments
Use double semicolons for comments:
`;; This is a comment ;;`

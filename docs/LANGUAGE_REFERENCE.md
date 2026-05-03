# 📘 Snask Language Reference: The Comprehensive Manual (v0.4.0)
### Mastery through Flexibility and Native Execution

Snask is a high-performance language that bridges the gap between the productivity of Python/JS and the efficiency of C/Rust. Version 0.4.0 introduces significant Developer Experience (DX) improvements, making the language more flexible and modern.

Status note:
- This reference describes the intended language surface of Snask v0.4.0.
- Some items in this document are still `parcial` or `experimental` in the current compiler.
- Before treating a feature as stable, check `docs/FEATURE_STATUS.md`.

---

## 📖 Table of Contents
1. [Lexical Grammar](#1-lexical-grammar)
2. [Variables & Scope](#2-variables--scope)
3. [Operators & Expressions](#3-operators--expressions)
4. [Control Flow Architecture](#4-control-flow-architecture)
5. [Functions & Lambdas](#5-functions--lambdas)
6. [Object-Oriented Programming (OOP)](#6-object-oriented-programming-oop)
7. [Dynamic Dispatch (`__s_call_by_name`)](#7-dynamic-dispatch)
8. [Collections: Lists & Dicts](#8-collections-lists--dicts)
9. [Modules & The Project System (SPS)](#9-modules--the-project-system)
10. [Error Handling & Nil Safety](#10-error-handling--nil-safety)

---

## 1. Lexical Grammar

Snask supports two styles of block delimitation:
1.  **Significant Indentation:** Default unit is **4 spaces**.
2.  **Braces `{ }`:** Standard C-style block delimitation.

You can freely mix both styles, though consistency is recommended.

### Semicolons
Semicolons (`;`) are **optional**. A statement can be terminated by a newline or the end of a block.

### Comments
*   **Single-line:** Use `//` for comments. They can be placed on their own line or at the end of a code line (inline).
*   **Multi-line:** Not supported (use consecutive `//`).

### Keywords
The following words are reserved:
`if`, `else`, `while`, `for`, `fun`, `class`, `extends`, `return`, `let`, `mut`, `const`, `nil`, `true`, `false`, `is_nil`, `zone`, `promote`, `import`, `export`, `from`.

---

## 2. Variables & Scope

Snask enforces three levels of mutability.

### `let` (Immutable)
Once assigned, it cannot be changed.
```snask
let x = 10
x = 20 // 🛑 SEMANTIC ERROR: Cannot reassign immutable variable 'x'
```

### `mut` (Mutable)
Standard reassignable variable.
```snask
mut counter = 0
counter = counter + 1 // ✅ OK
```

### `const` (Global/Immutable)
Declared at the top level of a module.
```snask
const API_URL = "https://api.snask.org"
```

---

## 3. Operators & Expressions

### Arithmetic
*   `+`: Addition and **Auto-String Concatenation**.
*   `-`, `*`, `/`, `//` (Integer Division).

### String Interpolation (v0.4.0)
You can embed expressions directly inside strings using curly braces.
```snask
let name = "Davi"
print("Olá, {name}! 1 + 1 = {1 + 1}")
```

### Trailing Commas (v0.4.0)
Allowed in lists, dicts, and function arguments/parameters.
```snask
let list = [
    1,
    2,
    3, // ✅ OK
]
```

---

## 4. Control Flow Architecture

Blocks can use indentation or braces.

### `if` / `else`
```snask
if score > 90 {
    print("A Grade")
} else if score > 80
    print("B Grade") // Indentation still works!
else {
    print("C Grade")
}
```

### `while` (The Core Loop)
```snask
mut i = 0
while i < 3 {
    print("Loop: {i}\n")
    i = i + 1
}
```

---

## 5. Functions & Lambdas

### Standard Function
```snask
fun calculate_total(price, tax) {
    return price * (1.0 + tax)
}
```

### Flexible Entry Point (v0.4.0)
Every program requires a `class main`. However, the entry function no longer needs to be named `start()`. Snask will execute the **first method** found in `class main` if `start` is not present.

---

## 6. Object-Oriented Programming (OOP)

### Class Definition
```snask
class Point {
    mut x = 0
    mut y = 0

    fun init(x, y) {
        self.x = x
        self.y = y
    }
}
```

---

## 8. Collections: Lists & Dicts

### Lists (Ordered)
```snask
let fruits: list<str> = ["Apple", "Banana", "Cherry",] // Trailing comma!
print(fruits[0])
```

### Dictionaries (Key/Value)
```snask
let user: dict<str, int> = {
    "id": 1,
    "level": 10,
}
print(user["id"])
```

Collection annotations can be nested:

```snask
let table: dict<str, list<int>> = {
    "scores": [10, 20, 30],
}
```

---

🚀 **Snask 0.4.0: Modern, Fast, and expressive.**

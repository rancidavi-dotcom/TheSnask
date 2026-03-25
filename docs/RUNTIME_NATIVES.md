# ⚙️ Snask Runtime Native API: Deep-Dive Technical Manual (v0.3.6)
### The Low-Level Interface and Internal C-Ecosystem

The Snask Runtime is a high-performance C layer that provides the bridge between the LLVM-compiled code and the Operating System. This document provides a complete technical specification of the runtime's internal structure and its native function exports.

---

## 📖 Table of Contents
1. [Memory Layout: The Snask Structs](#1-memory-layout)
2. [C Calling Convention](#2-c-calling-convention)
3. [Module: RT_OBJ (Object & List Management)](#3-module-rt_obj)
4. [Module: RT_JSON (Serialization Architecture)](#4-module-rt_json)
5. [Module: RT_IO (Input/Output & Time)](#5-module-rt_io)
6. [Module: RT_SQLITE (Persistence Layer)](#6-module-rt_sqlite)
7. [Module: RT_GUI (The GTK-Snask Bridge)](#7-module-rt_gui)
8. [Module: RT_HTTP (Networking Stack)](#8-module-rt_http)
9. [Module: RT_GC (Garbage Collection Tracking)](#9-module-rt_gc)
10. [Advanced: Dynamic Dispatch Mechanics](#10-advanced-manual-pointer-control)

---

## 1. Memory Layout: The Snask Structs

Snask uses a uniform representation for all values to ensure maximum speed and compatibility.

### `SnaskValue` (24 Bytes)
The basic unit of data. Everything from a `number` to a `User` class instance is a `SnaskValue`.

```c
typedef struct {
    double tag;   // 0=NIL, 1=NUM, 2=BOOL, 3=STR, 4=OBJ
    double num;   // Stores Number value or Boolean (1.0/0.0)
    void* ptr;    // Stores Pointer to String or SnaskObject
} SnaskValue;
```

### `SnaskObject` (Variable Size)
Represents a Dict, List, or Class Instance.

```c
typedef struct {
    char** names;      // Array of field names (null for Lists)
    SnaskValue* values;// Array of property values
    int count;         // Number of elements
} SnaskObject;
```

---

## 2. C Calling Convention

Snask LLVM backend calls C functions using a pointer-based signature to avoid stack-copying large structs.

### Function Signature Pattern:
For a Snask function `fun sum(a, b)`:
`void s_sum(SnaskValue* out, SnaskValue* a, SnaskValue* b)`

*   **`out`:** The destination where the return value must be stored.
*   **`args...`:** Pointers to the input values on the caller's stack frame.

---

## 3. Module: RT_OBJ (Object & List Management)

This module handles the creation, access, and modification of objects.

### Core Native Exports:
*   `s_alloc_obj(out, size, names)`: Allocates a new heap-managed object.
*   `s_get_member(out, obj, index)`: Retrieves a field value.
*   `s_set_member(obj, index, value)`: Updates a field value.
*   `s_len(out, val)`: Returns the count of elements (or string length).
*   `s_is_nil(out, val)`: Returns boolean check for nil.

### OM Extensions:
*   `s_arena_alloc_obj(out, size, names)`: Used for `new arena` expressions.
*   `s_arena_reset()`: Used for `zone` cleanup.

---

## 4. Module: RT_JSON (Serialization Architecture)

The JSON module provides high-speed serialization and deserialization.

### Key Implementation:
*   `json_stringify(out, obj)`: Recursively converts a SnaskObject to a JSON string. Uses an internal `string_buffer` for performance.
*   `json_parse(out, json_str)`: Uses the **JP (JSON Parser)** library to convert strings back into SnaskObjects.

---

## 5. Module: RT_IO (Input/Output & Time)

The primary interface for system interaction.

### Native Exports:
*   `s_print(val)`: Generic printer for all Snask types.
*   `s_println()`: Prints a newline and flushes stdout.
*   `s_time(out)`: Monotonic high-resolution clock provider.
*   `s_concat(out, s1, s2)`: Native string concatenation.

---

## 6. Module: RT_SQLITE (Persistence Layer)

Currently implemented as a mock/proxy layer for testing.

### Native Exports:
*   `sqlite_query(out, handle, sql)`: Executes SQL. Currently prints the SQL and returns an empty list `[]` for testing.
*   `sqlite_open(out, path)`: Opens a database handle.

---

## 7. Module: RT_GUI (The GTK-Snask Bridge)

Enables desktop application development using GTK3/4.

### Native Exports:
*   `gui_window_new(out, title)`: Creates a new window.
*   `gui_button_new(out, label)`: Creates a button component.
*   `gui_main_loop()`: Hands over execution to the GTK event loop.

---

## 8. Module: RT_HTTP (Networking Stack)

Uses `libcurl` or standard sockets internally for HTTP requests.

### Native Exports:
*   `http_get(out, url)`: Blocking GET request.
*   `http_post(out, url, body)`: Blocking POST request.

---

## 9. Module: RT_GC (Garbage Collection Tracking)

The runtime uses a simplified **Deferred Free** model.

### Process:
1.  All heap pointers are tracked in a global registry.
2.  Allocations are kept alive for the duration of the program/context.
3.  `snask_gc_cleanup()` is called at exit to free all remaining references.

---

## 10. Advanced: Dynamic Dispatch Mechanics

The function `__s_call_by_name` (internal `s_call_by_name`) is implemented as follows:

```c
void s_call_by_name(SnaskValue* out, SnaskValue* method_name_val, ...) {
    char* name = (char*)method_name_val->ptr;
    // Replace '::' with '_NS_'
    char* sym_name = sanitize_symbol(name);
    void* fp = dlsym(RTLD_DEFAULT, sym_name);
    
    if (fp) {
        typedef void (*Fn)(SnaskValue*, ...);
        ((Fn)fp)(out, arg1, arg2, arg3);
    }
}
```

This mechanism allows for complete type-safe reflection in a natively compiled language.

---
🚀 **Master the Runtime, and you master the very heart of Snask Performance.**

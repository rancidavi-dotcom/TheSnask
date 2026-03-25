# 📖 Orchestrated Memory (OM) Overview

Orchestrated Memory (OM) is Snask's native memory management system.

Unlike traditional approaches like Garbage Collection or manual management, OM is based on **Intent Orchestration**, where memory behavior is determined by context, scope, and explicit developer choices.

The goal is simple:

⚡ **Performance close to or exceeding C**
🧠 **Total predictability**
🛡️ **Safety without unnecessary overhead**
🎯 **Progressive control (from simple to advanced)**

---

## 🧱 1. Intelligent Default
```snask
let x = new Object();
```

By default, Snask does NOT use:
- Traditional GC
- Mandatory heap
- Fixed allocation

The compiler automatically decides based on:
- Lifetime
- Scope
- Usage

Possible destinations:
- **Stack**
- **Arena**
- **Heap** (if necessary)

The developer writes simply. The compiler does the heavy lifting.

---

## 🧬 2. Implicit Lifetimes
Snask automatically understands the lifetime of variables.

```snask
zone "request"
    let user = new User();
// Upon exiting the scope, user is automatically freed.
```

**Benefits:**
- Zero leaks by default.
- No need for `free()`.
- Predictable behavior.

---

## 🔗 3. Memory Promotion
Objects can change regions during execution.

```snask
let user = new User();
promote user to heap;
```

**Common usage:**
- Object starts local.
- Later needs to survive longer.
- Promotion is explicit, avoiding unnecessary copies while maintaining total control.

---

## ⚔️ 4. Explicit Control
The developer can directly define where to allocate:

```snask
let a = new stack Object();
let b = new arena Enemy();
let c = new heap Data();
```

**Available Types:**
- **Stack** → Fast, short scope.
- **Arena** → Grouped, bulk release.
- **Heap** → Long-lived, manual control.

---

## 🧨 5. @unsafe System
Dangerous code must be explicitly marked.

**Function definition:**
```snask
@unsafe fn dangerous() -> unsafe_ptr
    return malloc(64);
```

**Safe call:**
```snask
@unsafe -> dangerous();

// OR

@unsafe
    dangerous();
```

---

## 🧠 6. Memory-Aware Types
Types carry information about allocation and safety.

```snask
let a: stack Object;
let b: heap Object;
let c: unsafe_ptr;
```

---

## ⚡ 7. Invisible Optimization
The compiler can automatically optimize:
- Allocation in registers.
- Inline stack.
- Complete elimination (if possible).

---

## 🎯 8. Execution Profiles
OM behavior can be adjusted globally:

```snask
@profile "game"
// or
@profile "server"
```

**Adjustments include:**
- Aggressive arena usage.
- Conservative heap.
- GC disabled or incremental.

---

## 🧠 OM Philosophy
Orchestrated Memory doesn't ask "where to allocate?". It asks **"what is the role of this data in the program's flow?"**.

---

## 🏁 Conclusion
OM combines:
- Intelligent automation.
- Explicit control.
- Traceable safety.
- Extreme performance.

...without sacrificing the developer experience.
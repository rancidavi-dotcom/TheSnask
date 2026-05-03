# 🧠 Orchestrated Memory (OM): The Definitive Guide (v0.4.1)
### Deterministic Performance without Garbage Collection

**Orchestrated Memory (OM)** is the core innovation of the Snask programming language. It is a memory management paradigm designed to eliminate the trade-off between **Safety** and **Performance**.

In the world of C/C++, memory is a minefield. In Java/Go/Python, memory is managed by a background process (Garbage Collector) that causes unpredictable pauses (stuttering). **OM is the Third Way.**

For the C interop layer built on top of OM, see `docs/OM_SNASK_SYSTEM.md`. That document covers `import_c_om`, Auto-OM contract inference, `.om.snif` patches, native LLVM calls to C libraries, and OM-managed C resources.

---

## 🏗️ The OM Philosophy: "Cleaning Contexts, Not Objects"

Traditional managed languages track every object and its references. When memory is full, the GC scans the heap to see what can be deleted. 

**OM works differently.** In Snask, you define **Zones** (lexical scopes). Every allocation within a zone is tracked collectively. When the zone ends, the entire context is reclaimed **instantly**.

---

## 📊 The Memory Hierarchy

Snask manages memory through four distinct tiers, allowing developers to choose the exact trade-off for every task:

| Tier | Backed By | Lifecycle | Speed |
| :--- | :--- | :--- | :--- |
| **Stack** | CPU Stack | Current Frame | Instant |
| **Arena** | Zone Buffer | Current Zone | ~3 Assembly Instructions |
| **Managed Heap** | Global Heap | Reference Counted | High-Performance |
| **Static** | Binary Image | Full Program | Zero-Cost |

---

## ⚡ 1. Arenas: The Performance Beast

Arenas are the star of Snask's performance. When you allocate with `new arena`, the runtime simply increments a pointer in a contiguous block of memory.

- **Zero Fragmentation:** No searching for free space.
- **Cache Local:** Data allocated together stays together in the CPU cache.
- **O(1) Allocation:** Constant time, regardless of object size.

```snask
zone "heavy_computation"
    mut list = []
    while i < 1000000
        list.push(new arena LargeObject())
// One instruction resets the entire 128MB+ buffer here.
```

---

## 📦 2. Zones: Lexical Lifetimes

Zones are Snask's primary way of organizing memory. They follow the **lexical structure** of your code.

### Named Zones for Services
In the **Zenith Framework**, every incoming HTTP request is wrapped in a named zone. 

```snask
class ApiServer {
    fun handle_request(req)
        zone "http_req_{req.id}"
            let data = db.fetch_all()
            return json::stringify(data)
        // Everything used to process the request is purged here.
}
```

---

## 🏗️ 3. Stack Allocation: Avoiding the Malloc

Snask allows allocating classes directly on the stack. This is the ultimate optimization for short-lived helper objects.

```snask
fun update_physics(obj)
    // Point lives in the function frame, zero heap interaction
    let delta = new stack Point(0.5, 9.8)
    obj.move(delta)
```

---

## 🚀 4. Promotion: The Elastic Lifecycle

What if data created in a temporary zone needs to outlive it? Snask provides the `promote` mechanism.

```snask
fun build_cache()
    zone "temp_buffer"
        let data = expensive_parse()
        promote data to heap // Moves data from Arena to Global Heap
        return data
    // temp_buffer is cleared, but 'data' survives.
```

---

## 🔬 Deep Dive: The Block System (v0.4.1)

Unlike the fragmented heaps of C or Java, Snask's Arena (OM) uses a **Linear Block Memory** structure. In our benchmarks, Snask achieves up to **40x faster allocation** than Python 3.12 by avoiding `malloc` and `free` syscalls during the critical path of execution.

### Safety Guarantees
- **Boundary Checks:** All arenas have guard pages.
- **Debug Tracking:** The runtime detects if an Arena pointer escapes its Zone.
- **Type Safety:** Memory is typed from the moment of allocation to reclamation.

---

🚀 **Snask OM is about predictable power. It's memory management for the modern era.**

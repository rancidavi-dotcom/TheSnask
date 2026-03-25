# 🧠 Orchestrated Memory (OM): The Definitive Guide (v0.3.6)
### Architecting Zero-GC High-Performance Applications in Snask

Welcome to the heart of the Snask revolution. **Orchestrated Memory (OM)** is not just a feature; it is a paradigm shift in how we think about computer memory in high-level programming languages. 

This guide provides an exhaustive deep dive into the mechanics, strategies, and patterns of OM.

---

## 📖 Table of Contents
1. [Theoretical Foundation](#1-theoretical-foundation)
2. [The Memory Hierarchy](#2-the-memory-hierarchy)
3. [Zones: Lexical Memory Lifecycles](#3-zones-lexical-memory-lifecycles)
4. [Arenas: The Speed of Pointer Increment](#4-arenas-the-speed-of-pointer-increment)
5. [Stack Allocation: Zero Malloc Architecture](#5-stack-allocation-zero-malloc-architecture)
6. [Promotion: Moving Between Strategies](#6-promotion-moving-between-strategies)
7. [The Zenith Request & Service Model](#7-the-zenith-request--service-model)
8. [Performance Benchmarks & Analysis](#8-performance-benchmarks--analysis)
9. [Anti-Patterns & Common Pitfalls](#9-anti-patterns--common-pitfalls)
10. [Advanced: Manual Pointer Control (`@unsafe`)](#10-advanced-manual-pointer-control-unsafe)
11. [Deep-Dive: Arena Memory Layout](#11-deep-dive-arena-memory-layout)
12. [Zone Hierarchies: The Memory Stack](#12-zone-hierarchies-the-memory-stack)
13. [Case Study: Game Engine Tick (60 FPS)](#13-case-study-game-engine-tick-60-fps)
14. [Arena Safety & Boundaries](#14-arena-safety--boundaries)
15. [The "Promote" Mechanism: Behind the Scenes](#15-the-promote-mechanism-behind-the-scenes)

---

## 1. Theoretical Foundation

In traditional languages, we have two extremes:
*   **Manual Management (C/C++):** High performance, but extreme risk of memory leaks and dangling pointers.
*   **Garbage Collection (Java/Python/Go):** Safety and ease of use, but at the cost of "Stop the World" pauses and CPU overhead.

**OM (Orchestrated Memory)** is the "Third Way". It combines the speed of manual management with the safety of scope-based cleanup.

### The Philosophy
In Snask, memory is treated as a **Stream of Execution Tokens**. Instead of cleaning up individual objects, we clean up **Contexts**.

---

## 2. The Memory Hierarchy

Snask v0.3.6 manages four distinct memory tiers:

| Tier | Type | Lifetime | Performance |
| :--- | :--- | :--- | :--- |
| **Static** | Read-only | Program Execution | Instant |
| **Stack** | Frame-based | Function Scope | Extremely Fast |
| **Arena** | Zone-based | Zone Scope | Very Fast |
| **Heap** | Managed | Global / Reference Tracked | Reliable |

---

## 3. Zones: Lexical Memory Lifecycles

A `zone` is a named or anonymous block that defines a temporal boundary for allocations.

### Anonymous Zone
```snask
zone ""
    let data = [1..1000];
    print(len(data));
// Memory for 'data' list is reclaimed here instantly.
```

### Named Zone (Runtime Traceability)
Named zones are used by the **Zenith Framework** to isolate requests.

```snask
zone "request_abc"
    let user = User().all();
    let response = json_stringify(user);
    send(response);
// 128MB of arena space is reset to offset 0 here.
```

### Nested Zones
Snask supports nested zones. Each zone manages its own buffer offset.

---

## 4. Arenas: The Speed of Pointer Increment

The **Arena** is a pre-allocated contiguous buffer (default 128MB). When you use `new arena`, the allocator does only one thing:
1.  Check if `current_offset + size < buffer_capacity`.
2.  Return `current_offset`.
3.  Increment `current_offset` by `size`.

**There is no complex searching, no fragmentation, and no immediate free.**

---

## 5. Stack Allocation: Zero Malloc Architecture

Snask v0.3.6 allows allocating full objects on the CPU stack. This is the ultimate optimization for short-lived data structures.

```snask
fun update_position(delta)
    // 'point' lives in the current function stack frame
    let point = new stack Point(self.x, self.y);
    point.move(delta);
    self.x = point.x;
    self.y = point.y;
    // No memory was ever allocated or freed in the heap.
```

---

## 6. Promotion: Moving Between Strategies

Sometimes a piece of data starts as temporary but needs to persist.

```snask
fun load_config()
    zone "parser"
        let raw = read_file("config.json");
        let obj = json_parse(raw); // Allocated in Arena
        
        promote obj to heap;
        return obj;
    // 'raw' is freed, 'obj' is moved to managed heap.
```

---

## 7. The Zenith Request & Service Model

Zenith uses OM to achieve legendary performance.

### System Services Architecture (Zenith for Snask OS)
When building system services (long-running background processes), the Zenith architecture uses "Ephemeral Zones" for each service tick or event.

```snask
class MonitorService extends Service
    fun on_tick()
        zone "service_cycle"
            let cpu = os::cpu_usage(); // Temporary allocation
            let ram = os::ram_usage(); // Temporary allocation
            
            if ram > threshold
                self.dispatcher.dispatch("system.alert", { "msg": "High RAM" });
        // All temporary data for this tick is flushed here.
```

---

## 11. Deep-Dive: Arena Memory Layout (The Block System)

To understand Snask's performance, look at how the Arena organizes bytes. Unlike traditional `malloc`, which maintains a linked list of free blocks, Snask OM uses a **Linear Block Allocation** system.

### Buffer Structure
Each Arena v0.3.6 consists of a contiguous buffer divided into:
1.  **Header (64 bytes):** Metadata (ID, total size, current offset).
2.  **Payload (128MB+):** Where objects reside.
3.  **Guard Page (4KB):** Protected memory to detect Arena overflows.

### The Cost of Allocation
In Snask, `new arena` compiles to only 3-5 assembly instructions:
1.  `ADD` (increment the Arena offset).
2.  `CMP` (check if it exceeded the limit).
3.  `JMP` (trigger an error if it did).

**There is no search for free blocks.** This makes object allocation in Snask orders of magnitude faster than in Java or Python.

---

## 12. Zone Hierarchies: The Memory Stack

Snask allows **nested zones**. This creates a **Context Stack** structure.

### Behavior of Nested Zones:
When you enter a zone inside another, Snask saves the `current_offset` of the parent zone.

```snask
zone "parent"
    let p1 = new arena Point(0, 0); // Offset 0
    zone "child"
        let c1 = new arena Point(1, 1); // Offset 24
        let c2 = new arena Point(2, 2); // Offset 48
    // End of "child", offset returns to 24.
    // p1 remains valid, but c1 and c2 are cleared.
```

---

## 13. Case Study: Game Engine Tick (60 FPS)

Imagine a game running at 60 FPS. Each frame (tick), thousands of particles are created.

*   **In Python/Unity (C#):** The GC would sweep these objects every few seconds, causing stutters.
*   **In Snask:**
    ```snask
    while running
        zone "frame_tick"
            update_entities(); // Thousands of new arena Sprite();
            render_frame();
        // End of frame: Memory reset in 0.1ms.
    ```
Snask guarantees **Zero Stuttering**, making it ideal for simulations and game engines.

---

## 14. Arena Safety & Boundaries

Although fast, the Arena introduces **Temporal Ownership**. 

### Temporal Ownership Rules:
1.  **Lifetime Bond:** An Arena object dies with its zone.
2.  **No References Out:** Never let an Arena object's reference escape to a higher zone.
3.  **Cross-Zone Check:** The v0.3.6 runtime checks for dangerous assignments in debug mode.

---

## 15. The "Promote" Mechanism: Behind the Scenes

When you run `promote obj to heap`, the runtime:
1.  Calculates the recursive size of the object graph.
2.  Allocates space in the **Global Managed Heap**.
3.  Performs a bitwise **Deep Copy**.
4.  Updates the original reference.

This allows using the Arena as a **Work Buffer**, moving only the final result to the permanent Heap.

---
🚀 **Snask OM is freedom from the Garbage Collector. Build something incredible.**

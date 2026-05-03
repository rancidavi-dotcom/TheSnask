# 🚀 Learn Snask (Complete Guide v0.4.0)

This is the definitive guide to mastering Snask, from "Hello World" to complex service architectures.

---

## 1) Hello World
Every Snask app needs a `class main`. You can name the entry method whatever you like!

```snask
class main {
    fun run_app() {
        // Hello, Snask!
        print("Hello, Snask!\n")
    }
}
```

---

## 2) Variables and Types
- `let`: Immutable
- `mut`: Mutable

```snask
class main {
    fun start() {
        let name = "Davi"
        mut age = 25
        age = 26

        print("Name: {name}\n")
        print("Age: {age}\n")
    }
}
```

---

## 3) Conditionals and Loops
You can use indentation or braces `{ }`.

```snask
class main {
    fun start() {
        let score = 8.5

        if score >= 7.0 {
            print("Approved\n")
        } else {
            print("Failed\n")
        }

        mut i = 1
        while i <= 3 {
            print("Step: {i}\n")
            i = i + 1
        }
    }
}
```

---

## 4) Global Functions and Recursion

```snask
fun add(a, b) {
    return a + b
}

fun fact(n) {
    if n <= 1 {
        return 1
    }
    return n * fact(n - 1)
}

class main {
    fun main() {
        print("Sum: {add(10, 20)}\n")
        print("Factorial(5): {fact(5)}\n")
    }
}
```

---

## 5) Classes, Attributes, and Inheritance

```snask
class Animal {
    mut name = ""

    fun say_hello() {
        print("Hi, my name is {self.name}\n")
    }
}

class Dog extends Animal {
    fun bark() {
        print("{self.name} says: Woof Woof!\n")
    }
}

class main {
    fun start() {
        let d = Dog()
        d.name = "Rex"
        d.say_hello()
        d.bark()
    }
}
```

---

## 6) Modules and Namespaces

```snask
import "os"
import "json"

class main {
    fun start() {
        print("Platform: {os::platform()}\n")
        
        let data = { "id": 1, "status": "ok", } // Trailing comma!
        print("JSON: {json::stringify(data)}\n")
    }
}
```

---

## 7) OM-Snask-System

```snask
class User {
    mut name = "Davi"
}

class main {
    fun start() {
        zone "request" {
            let user = User()
            print("{user.name}\n")
        }
        // Memory cleaned instantly.
    }
}
```

`zone`, `new stack`, `new arena`, `promote` and C resources imported through `import_c_om` are all part of the same OM-Snask-System. See `docs/OM_SNASK_SYSTEM.md`.

---

## 8) SPS Commands
- `snask init`: Initializes a new project.
- `snask build`: Compiles the current project.
- `snask run`: Compiles and executes.
- `snask install <package>`: Adds a dependency.

---
🚀 **Snask is the future of native computing. Develop responsibly.**

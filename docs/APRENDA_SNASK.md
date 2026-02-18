# Learn Snask (Step-by-step)

This guide is a practical path for learning Snask with small, tested examples.

## 1) Hello world
```snask
class main
    fun start()
        print("Hello, Snask!");
```

## 2) Variables
```snask
class main
    fun start()
        let name = "Davi";   // immutable
        mut age = 25;        // mutable
        age = 26;
        print(name, "age", age);
```

## 3) Conditions and loops
```snask
class main
    fun start()
        let score = 8.5;
        if score >= 7.0
            print("Approved");
        else
            print("Failed");

        mut i = 1;
        while i <= 3
            print("Step:", i);
            i = i + 1;
```

## 4) Functions (recursion works)
```snask
fun add(a, b)
    return a + b;

fun fact(n)
    if n <= 1
        return 1;
    return n * fact(n - 1);

class main
    fun start()
        print("sum:", add(10, 20));
        print("fact(5):", fact(5));
```

## 5) SPS projects
```bash
snask init
snask build
```


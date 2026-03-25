# 📁 Project Configuration (snask.snif)
### Snask Infrastructure File (v0.3.6)

The `.snif` file is the heart of every Snask project. It defines metadata, source entries, and build options.

---

## 1. Minimal Configuration

Created automatically by `snask init`.

```snif
[project]
name = "hello_snask"
version = "0.3.6"
main = "src/main.snask"
```

---

## 2. Dependencies

Manage external libraries and official packages.

```snif
[dependencies]
zenith = "0.2.0"
logger = "1.0.0"
```

---

## 3. Build Strategies

Configure how your project is compiled via LLVM.

```snif
[build]
optimize = "O3"
target = "native"
strip = true
```

---

## 4. Custom Scripts

Define shortcuts for common tasks.

```snif
[scripts]
test = "snask build tests/test_all.snask && ./tests/test_all"
bench = "snask build bench.snask --release && ./bench"
```

---

## 5. Metadata for Zenith Framework

If your project is a Zenith application, you can define controllers and routes discovery.

```snif
[zenith]
controllers = "app/controllers"
models = "app/models"
```

---
🚀 *Organize your code, optimize your world.*

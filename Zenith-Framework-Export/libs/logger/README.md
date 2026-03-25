# logger — Zenith Library

Logging library with levels and timestamps for Snask.

## Installation

```bash
# The logger.snask file is already in ~/.snask/packages/
```

## Usage

```snask
import "logger";

logger::info("Server started on port 8080");
logger::warn("Slow database connection");
logger::error("Critical failure in module X");
logger::debug("Return value: 42");
```

## Output

```
[INFO]  1742560461 | Server started on port 8080
[WARN]  1742560461 | Slow database connection
[ERROR] 1742560461 | Critical failure in module X
[DEBUG] 1742560461 | Return value: 42
```

## Functions

| Function | Description |
|---|---|
| `logger::info(msg)` | Info log (green in supported terminals) |
| `logger::warn(msg)` | Warning log |
| `logger::error(msg)` | Error log |
| `logger::debug(msg)` | Debug log |
| `logger::raw(level, msg)` | Log with custom level |

## Observations (Snask pre-alpha)
- Timestamp uses `time()` in Unix seconds.
- Human-readable formatting is approximate.
- Compatible with Zenith Framework and standalone apps.

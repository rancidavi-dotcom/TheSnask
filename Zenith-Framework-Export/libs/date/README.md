# date — Zenith Library

Date and time library for Snask, based on the Unix `time()` timestamp.

## Installation

```bash
# The date.snask file is already in ~/.snask/packages/
```

## Usage

```snask
import "date";

let ts = date::now();
// 1742560461 (unix timestamp)

let stamp = date::stamp();
// "[ts:1742560461]"

let elapsed = date::since(ts_prev);
// "3 seconds ago"
```

## Functions

| Function | Description |
|---|---|
| `date::now()` | Returns current Unix timestamp (float). |
| `date::stamp()` | Formatted string `[ts:123456789]`. |
| `date::label()` | Human-readable label (Unix prefix currently). |
| `date::since(ts)` | Elapsed time as string ("3s ago"). |
| `date::diff(ts1, ts2)` | Difference in seconds between two timestamps. |

## Observations (Snask pre-alpha)
- `date::stamp()` and `date::since()` use integer division of the Unix timestamp.
- No timezone support.
- Based on system time via native `time()`.

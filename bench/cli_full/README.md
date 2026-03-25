# cli_full — “real CLI app” size benchmark

This benchmark is meant to represent a more realistic CLI tool:

- a small command table (subcommands)
- a config string in **SNIF** that gets parsed (manifest-like)
- a couple of utility helpers (hash, sum, etc.)

It is designed to:
- run with no args (so `bench/run.sh` can execute it),
- still include “real app” code paths so the executable size is meaningful.


# Snask Vault (demo app)

This is a **demo password vault** built with Snask 0.3.1 to exercise the “batteries-included desktop + tooling” direction.

## Security note (important)
This demo implements **basic obfuscation** (character shifting) to avoid storing passwords as plain text, but it is **not cryptographically secure**.

Do not use this to store real secrets until Snask has a real cryptography library/runtime primitives.

## Build / run
From this directory:
```bash
snask build
./snask_vault
```

## Data files
The vault is stored in:
- `~/.snask_vault/vault.snif`


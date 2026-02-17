#!/usr/bin/env python3
"""
Migra um registry.json no formato legado para um índice por pacote:

  index/<primeira-letra>/<nome>.json

Uso:
  python3 tools/registry/migrate_registry_to_index.py /caminho/para/registry.json /caminho/para/repo
"""

import json
import os
import sys


def main() -> int:
    if len(sys.argv) != 3:
        print("Uso: migrate_registry_to_index.py <registry.json> <repo_dir>", file=sys.stderr)
        return 2

    registry_path = sys.argv[1]
    repo_dir = sys.argv[2]

    with open(registry_path, "r", encoding="utf-8") as f:
        obj = json.load(f)

    pkgs = obj.get("packages", {})
    if not isinstance(pkgs, dict):
        print("registry.json inválido: campo 'packages' não é objeto", file=sys.stderr)
        return 2

    index_dir = os.path.join(repo_dir, "index")
    os.makedirs(index_dir, exist_ok=True)

    count = 0
    for name, meta in pkgs.items():
        if not isinstance(name, str) or not name:
            continue
        if not isinstance(meta, dict):
            continue

        first = name[0].lower()
        out_dir = os.path.join(index_dir, first)
        os.makedirs(out_dir, exist_ok=True)

        out_path = os.path.join(out_dir, f"{name}.json")
        out_obj = {
            "version": str(meta.get("version", "")),
            "url": str(meta.get("url", "")),
            "description": str(meta.get("description", "")),
        }
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(out_obj, f, ensure_ascii=False, indent=2)
            f.write("\n")
        count += 1

    print(f"OK: gerado index/ para {count} pacotes em {index_dir}")
    print("Dica: mantenha registry.json por compatibilidade por enquanto.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())


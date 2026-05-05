# Benchmarks de RAM: Snask vs Electron

Objetivo: medir uso de memoria de uma janela nativa Snask/GTK contra uma janela minima Electron/Chromium.

## Metrica principal

PSS, porque Electron cria uma arvore de processos e paginas compartilhadas precisam ser contabilizadas com justica.

## Fonte de verdade

```text
bench/ram/out/report.md
```

## Observacao

O numero exato varia com tema GTK, versao do Chromium, distro e desktop. A conclusao esperada e estrutural: GUI nativa tende a carregar muito menos runtime que Electron.

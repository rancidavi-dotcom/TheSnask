# Instalacao do Snask no Linux

O caminho recomendado e usar o instalador da raiz. Ele baixa ou atualiza o fonte em `~/.snask/src/TheSnask`, compila o binario, roda `snask setup` e instala `snask` em `~/.snask/bin/snask`.

```bash
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

Depois, garanta que o binario do usuario esteja no `PATH`:

```bash
export PATH="$HOME/.snask/bin:$PATH"
```

Para deixar permanente, coloque essa linha no `~/.bashrc`, `~/.zshrc` ou arquivo equivalente do seu shell.

## Por que LLVM 18?

Snask usa `inkwell` com `llvm-sys` configurado para LLVM 18. Isso significa que o build do compilador precisa encontrar um `llvm-config` da versao 18. O erro classico e:

```text
No suitable version of LLVM was found system-wide or pointed to by LLVM_SYS_180_PREFIX.
```

O instalador tenta resolver isso sozinho procurando:

- `llvm-config-18`
- `/usr/lib/llvm18/bin/llvm-config`
- `/usr/lib/llvm-18/bin/llvm-config`
- `llvm-config`, somente se ele for LLVM 18

Quando encontra, ele exporta automaticamente `LLVM_CONFIG_PATH` e `LLVM_SYS_180_PREFIX` durante a compilacao.

## Distros suportadas pelo instalador

O script tenta instalar dependencias automaticamente em:

- Arch Linux e derivados com `pacman`
- Debian, Ubuntu e derivados com `apt`
- Fedora com `dnf`
- openSUSE com `zypper`
- Alpine com `apk`

Em distros onde o repositorio padrao nao oferece LLVM 18, instale LLVM 18 manualmente e rode o instalador com as variaveis apontadas.

## Arch Linux

```bash
sudo pacman -Syu --needed base-devel git rust cargo llvm18 llvm18-libs clang18 lld18 pkgconf gtk3 zlib sqlite
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

Se quiser buildar manualmente:

```bash
export LLVM_CONFIG_PATH=/usr/lib/llvm18/bin/llvm-config
export LLVM_SYS_180_PREFIX=/usr/lib/llvm18
cargo build --release
./target/release/snask setup
```

## Ubuntu/Debian

```bash
sudo apt update
sudo apt install build-essential git curl ca-certificates pkg-config rustc cargo \
  clang-18 llvm-18 llvm-18-dev lld-18 libclang-18-dev \
  libgtk-3-dev zlib1g-dev libsqlite3-dev

curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

## Variaveis de escape

Use essas variaveis quando a distro instala ferramentas em nomes/caminhos diferentes:

```bash
export LLVM_CONFIG_PATH=/caminho/para/llvm-config-18
export LLVM_SYS_180_PREFIX=/prefixo/do/llvm18
export SNASK_CLANG=/caminho/para/clang-18
export SNASK_LLC=/caminho/para/llc-18
export SNASK_LLVM_STRIP=/caminho/para/llvm-strip-18
export SNASK_LD_LLD=/caminho/para/ld.lld
```

Essas variaveis tambem sao respeitadas pelo proprio compilador Snask ao chamar `clang`, `llc`, `llvm-strip` e `ld.lld`.

## Instalar sem mexer nas dependencias do sistema

Se voce ja instalou tudo manualmente:

```bash
SNASK_INSTALL_DEPS=0 ./install.sh
```

## Verificacao

```bash
snask doctor
snask --help
```

Para testar um programa:

```bash
cat > hello.snask <<'SNASK'
class main {
    fun start() {
        print("Ola, Snask!\n")
    }
}
SNASK

snask build hello.snask --output hello
./hello
```

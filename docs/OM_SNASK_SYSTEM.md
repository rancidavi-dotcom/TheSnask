# OM-Snask-System

## Snask puro sobre bibliotecas C, com memoria orquestrada

Este documento descreve o **OM-Snask-System**, a camada de interop nativa que permite usar bibliotecas C a partir de codigo Snask puro, sem escrever sintaxe C no programa e sem gerenciar manualmente a vida de ponteiros C.

A frase guia do sistema e:

> Em vez de voce escrever o contrato, o Snask deve deduzir o contrato.

O objetivo nao e transformar Snask em um dialeto de C. O objetivo e o contrario: permitir que Snask use o ecossistema C sem herdar a experiencia manual de C. A biblioteca C continua existindo como ABI externa; o codigo do usuario continua sendo Snask; e o OM fica responsavel por transformar recursos nativos em valores com ciclo de vida orquestrado.

Status atual:

- `experimental`: a arquitetura ja existe e passa em testes reais com SDL2 e stdio.h.
- Ainda nao e uma promessa de suporte universal para toda API C existente.
- As regras de seguranca sao conservadoras: o scanner expoe apenas o que consegue mapear com seguranca razoavel.
- APIs com callbacks, structs complexas por valor, ownership ambiguo e varargs avancado ainda precisam de regras novas ou patches `.om.snif`.

## O que o sistema resolve

Em C, usar uma biblioteca normalmente exige lidar com:

- headers
- tipos opacos
- ponteiros
- alocacao indireta
- funcoes `create` / `destroy`
- caminhos de erro que precisam limpar recursos parcialmente criados
- constantes de macro e enum
- linkagem com `pkg-config`
- conversoes de strings e numeros
- disciplina manual de ownership

O OM-Snask-System centraliza essas preocupacoes no compilador, no scanner e no runtime OM.

O usuario escreve:

```snask
import_c_om "SDL2/SDL.h" as sdl2

class main
    fun start()
        zone "app":
            sdl2.init(sdl2.INIT_VIDEO)

            let window = sdl2.create_window(
                "Snask SDL2",
                100, 100,
                800, 600,
                sdl2.WINDOW_SHOWN
            )

            let renderer = sdl2.create_renderer(window, -1, 0)

            sdl2.set_render_draw_color(renderer, 20, 120, 220, 255)
            sdl2.render_clear(renderer)
            sdl2.render_present(renderer)

            sdl2.delay(2000)
            sdl2.quit()
```

O compilador entende que:

- `SDL2/SDL.h` deve ser lido como header C.
- `sdl2` e o namespace Snask da biblioteca.
- `SDL_Init` pode ser exposto como `sdl2.init`.
- `SDL_INIT_VIDEO` pode ser exposto como `sdl2.INIT_VIDEO`.
- `SDL_CreateWindow` cria um recurso opaco.
- `SDL_DestroyWindow` e o destrutor desse recurso.
- `SDL_CreateRenderer` cria outro recurso opaco.
- `SDL_DestroyRenderer` e o destrutor correspondente.
- O `zone "app"` delimita a vida dos recursos.

Quando a zona termina, o OM executa os destrutores associados aos recursos registrados nela.

## Snask e compilado, nao transpilado

O OM-Snask-System nao muda a natureza do compilador.

Snask nao gera C como saida principal. O pipeline continua sendo:

```text
codigo .snask
    -> lexer/parser
    -> AST
    -> analise semantica
    -> contratos OM deduzidos
    -> LLVM IR
    -> linker
    -> binario nativo
```

A biblioteca C entra no sistema como ABI nativa. O Snask emite chamadas LLVM para simbolos externos como `SDL_CreateWindow`, `SDL_RenderClear` ou `puts`. Isso e interop nativa, nao transpilacao.

Exemplo conceitual:

```text
sdl2.render_clear(renderer)
```

vira uma chamada nativa no LLVM para:

```text
SDL_RenderClear(SDL_Renderer*)
```

Mas isso acontece dentro do compilador. O usuario nao escreve C.

## Separacao de responsabilidades

O sistema e dividido em camadas:

```text
Snask source
    |
    v
Parser / AST
    |
    v
Import C OM Resolver
    |
    v
C Header Scanner
    |
    v
OM Contract Inference
    |
    v
Optional .om.snif Patch
    |
    v
LLVM Native Call Emitter
    |
    v
OM Runtime Resource Registry
    |
    v
Native Binary
```

Cada camada tem um trabalho claro.

## `import_c_om`

`import_c_om` e a porta de entrada explicita para bibliotecas C via OM.

```snask
import_c_om "stdio.h" as stdio
import_c_om "SDL2/SDL.h" as sdl2
```

A sintaxe significa:

- leia este header C;
- deduza uma superficie Snask para ele;
- use o alias como namespace publico;
- exponha somente chamadas que o OM considera seguras;
- registre recursos C em zonas quando houver ownership reconhecido.

O alias e importante porque define a API Snask:

```snask
stdio.puts("ola")
sdl2.init(sdl2.INIT_VIDEO)
```

## Contrato deduzido

O scanner le o header com Clang e extrai:

- funcoes
- assinaturas C
- parametros
- retornos
- enums
- macros numericas
- tipos opacos
- candidatos a construtor
- candidatos a destrutor

A partir disso, ele cria um contrato OM em memoria.

Um contrato gerado pode conter:

```snif
library sdl2

constant INIT_VIDEO: 32
constant WINDOW_HIDDEN: 8
constant QUIT: 256

resource SDLWindow:
    c_type: SDL_Window*
    constructor: SDL_CreateWindow
    destructor: SDL_DestroyWindow
    surface_type: sdl2.SDLWindow
    safety: SAFE
    reason: constructor `SDL_CreateWindow` returns `SDL_Window*` and paired destructor `SDL_DestroyWindow` accepts `SDL_Window*`

function create_window:
    c_function: SDL_CreateWindow
    surface: sdl2.create_window
    input: value
    output: resource
    c_return_type: SDL_Window*
    c_param_types: char*, int, int, int, int, Uint32
    safety: SAFE
    reason: constructor `SDL_CreateWindow` is registered as OM resource `sdl2.SDLWindow`
```

Esse contrato pode existir apenas em memoria durante a compilacao. O usuario nao precisa escrever esse arquivo.

## `.om.snif` agora e patch, nao binding gigante

No desenho antigo, uma biblioteca grande poderia exigir um `.om.snif` enorme. Isso nao escala.

No OM-Snask-System, o `.om.snif` tem outro papel:

- corrigir nomes ruins;
- bloquear uma funcao perigosa;
- ensinar uma regra que o scanner nao conseguiu deduzir;
- ajustar uma funcao especifica;
- declarar uma politica de ownership especial.

O `.om.snif` nao deve ser a fonte principal de bindings.

Fluxo correto:

```text
Header C
    -> contrato deduzido automaticamente
    -> patch .om.snif opcional
    -> contrato final usado no LLVM
```

Se existir `contracts/sdl2.om.snif`, ele entra como patch sobre o contrato gerado. O que nao estiver no patch continua vindo do scanner.

Isso permite que uma biblioteca enorme tenha apenas um arquivo pequeno de excecoes.

## Como o scanner decide o que e seguro

As regras atuais sao conservadoras. O scanner prefere bloquear uma funcao util a expor uma funcao insegura.

### Constantes

Macros numericas e enums sao expostos como propriedades do namespace:

```snask
sdl2.INIT_VIDEO
sdl2.WINDOW_HIDDEN
sdl2.QUIT
```

Para SDL2, o prefixo `SDL_` e removido:

```text
SDL_INIT_VIDEO -> sdl2.INIT_VIDEO
SDL_WINDOW_HIDDEN -> sdl2.WINDOW_HIDDEN
```

### Funcoes simples

Funcoes com parametros numericos, strings e retornos simples podem ser expostas diretamente:

```snask
stdio.puts("texto")
sdl2.delay(50)
sdl2.init(sdl2.INIT_VIDEO)
```

Elas viram chamadas nativas para C.

### `const char*`

Retornos `const char*` podem ser tratados como leitura/copia quando a funcao nao exige ownership manual.

Exemplo conceitual:

```snask
let platform = sdl2.get_platform()
print(platform)
```

O scanner marca esse tipo de API como `COPY_ONLY` quando a politica correta e copiar para um valor Snask, nao entregar o ponteiro cru para o usuario.

### Recursos opacos

O scanner procura padroes como:

```c
SDL_Window* SDL_CreateWindow(...);
void SDL_DestroyWindow(SDL_Window*);
```

e transforma isso em:

```snask
let window = sdl2.create_window(...)
```

O valor `window` e um recurso OM. O ponteiro real fica encapsulado.

### Metodos de recurso

Se uma funcao recebe o recurso como primeiro argumento, e os outros parametros sao simples, ela pode ser exposta como chamada segura:

```c
int SDL_RenderClear(SDL_Renderer *renderer);
```

vira:

```snask
sdl2.render_clear(renderer)
```

O compilador desempacota o ponteiro do recurso para fazer a chamada C, mas o programa Snask nao recebe acesso manual ao ponteiro.

### Destrutores

Funcoes como:

```c
SDL_DestroyWindow(SDL_Window*);
SDL_DestroyRenderer(SDL_Renderer*);
free(void*);
sqlite3_close(sqlite3*);
```

nao devem aparecer como chamadas normais para o usuario.

Elas sao escondidas atras do OM.

Em vez de:

```snask
sdl2.destroy_window(window)
```

o usuario escreve:

```snask
zone "screen":
    let window = sdl2.create_window(...)
```

e a zona limpa o recurso.

## O registro de recursos no runtime

Quando uma funcao C retorna um ponteiro opaco reconhecido como recurso, o gerador LLVM emite a chamada C e registra o resultado no runtime OM.

Fluxo:

```text
Snask:
    let window = sdl2.create_window(...)

LLVM:
    %ptr = call SDL_CreateWindow(...)
    %handle = call s_zone_register(%ptr, SDL_DestroyWindow, "sdl2.SDLWindow")
    retorna SnaskValue(TYPE_RESOURCE, %handle)
```

O valor Snask nao e o ponteiro C cru. Ele e um handle de recurso.

Quando uma funcao precisa do recurso:

```snask
sdl2.render_clear(renderer)
```

o gerador LLVM faz:

```text
%ptr = call s_om_resource_ptr(renderer_handle)
call SDL_RenderClear(%ptr)
```

Isso preserva a ergonomia do Snask e permite que o OM controle o ciclo de vida.

## Zonas e ordem de destruicao

Recursos criados dentro de uma zona pertencem a essa zona.

```snask
zone "sdl-renderer":
    let window = sdl2.create_window(...)
    let renderer = sdl2.create_renderer(window, -1, 0)
```

Quando a zona termina, o OM limpa em ordem apropriada para recursos registrados.

Em testes com SDL2, o trace mostrou:

```text
om cleanup sdl2.SDLRenderer in zone sdl-renderer
om cleanup sdl2.SDLWindow in zone sdl-renderer
```

Isso e exatamente o comportamento esperado: renderer antes da window.

## Exemplo: stdio.h

Em C:

```c
#include <stdio.h>

int main(void) {
    puts("stdio.h via C");
    return 0;
}
```

Em Snask:

```snask
import_c_om "stdio.h" as stdio

class main
    fun start()
        stdio.puts("stdio.h via Snask Auto-OM")
```

Aqui nao ha recurso para registrar, mas o exemplo prova a parte de chamada nativa simples:

- o header foi escaneado;
- `puts` foi deduzido;
- `char*` foi mapeado;
- o LLVM emitiu chamada para `puts`;
- o binario nativo chamou libc.

## Exemplo: SDL2 window

Snask:

```snask
import_c_om "SDL2/SDL.h" as sdl2

class main
    fun start()
        zone "sdl-window":
            let ok = sdl2.init(sdl2.INIT_VIDEO)
            let window = sdl2.create_window(
                "Snask SDL2 Auto-OM",
                0, 0,
                320, 240,
                sdl2.WINDOW_HIDDEN
            )
            sdl2.delay(50)
            print("SDL2 window allocated under OM")
            sdl2.quit()
```

Saida com trace OM:

```text
SDL2 window allocated under OM
om cleanup sdl2.SDLWindow in zone sdl-window
```

Em C equivalente, o programador teria que lembrar:

```c
SDL_Window *window = SDL_CreateWindow(...);
if (!window) {
    SDL_Quit();
    return 1;
}

SDL_Delay(50);

SDL_DestroyWindow(window);
SDL_Quit();
```

E se houver renderer:

```c
SDL_Renderer *renderer = SDL_CreateRenderer(window, -1, 0);
if (!renderer) {
    SDL_DestroyWindow(window);
    SDL_Quit();
    return 1;
}

SDL_DestroyRenderer(renderer);
SDL_DestroyWindow(window);
SDL_Quit();
```

No Snask:

```snask
zone "sdl-renderer":
    let window = sdl2.create_window(...)
    let renderer = sdl2.create_renderer(window, -1, 0)
```

O OM registra os dois e limpa ao sair.

## Exemplo: SDL2 renderer

```snask
import_c_om "SDL2/SDL.h" as sdl2

class main
    fun start()
        zone "sdl-renderer":
            let ok = sdl2.init(sdl2.INIT_VIDEO)

            let window = sdl2.create_window(
                "Snask SDL2 Renderer",
                0, 0,
                320, 240,
                sdl2.WINDOW_HIDDEN
            )

            let renderer = sdl2.create_renderer(window, -1, 0)

            sdl2.set_render_draw_color(renderer, 20, 120, 220, 255)
            sdl2.render_clear(renderer)
            sdl2.render_present(renderer)
            sdl2.delay(50)

            print("SDL2 renderer allocated under OM")
            sdl2.quit()
```

O codigo expressa a intencao visual. A parte de ownership fica fora da superficie do usuario.

## Exemplo: eventos SDL2

Algumas APIs C exigem structs por ponteiro:

```c
SDL_Event event;
SDL_PollEvent(&event);
```

Isso nao deve vazar para o usuario como ponteiro manual.

No estado atual, `SDL_PollEvent` recebe tratamento especial seguro:

```snask
let event_type = sdl2.poll_event()
print(event_type)
```

O compilador aloca armazenamento temporario na stack para o evento, chama `SDL_PollEvent`, le o campo de tipo e retorna um inteiro Snask.

Essa e a direcao correta para outras APIs com structs temporarias:

- o Snask deve expor uma superficie limpa;
- o compilador/runtime lida com a struct C;
- o usuario recebe valor Snask seguro.

## O que e considerado seguro hoje

O sistema ja cobre bem:

- funcoes C com parametros numericos;
- funcoes C com strings simples;
- retornos numericos;
- retornos `void`;
- retornos `const char*` copiaveis;
- constantes numericas de macros/enums;
- recursos opacos retornados por construtores;
- destrutores pareados por nome e tipo;
- metodos cujo primeiro parametro e um recurso conhecido;
- linkagem via `pkg-config` para bibliotecas conhecidas;
- overrides `.om.snif` como patch.

## O que ainda e limitado

Ainda precisa evoluir:

- callbacks C;
- function pointers;
- variadic functions complexas;
- structs C por valor;
- structs C com layout exposto;
- arrays e buffers mutaveis;
- ponteiros de saida com ownership ambiguo;
- ponteiros globais;
- recursos com destrutores dependentes de contexto;
- APIs que exigem ordem global complexa;
- APIs que retornam ponteiro emprestado com lifetime escondido;
- APIs que exigem thread affinity;
- unions;
- bitfields;
- macros que expandem para expressoes complexas;
- headers que dependem de defines de plataforma para revelar a API correta.

Essas limitacoes nao invalidam o sistema. Elas definem onde o scanner deve ser conservador.

## `SAFE`, `COPY_ONLY` e `BLOCKED`

O contrato OM usa niveis de exposicao.

### `SAFE`

A funcao pode ser chamada diretamente pela superficie Snask gerada.

Exemplos:

```snask
sdl2.delay(50)
sdl2.render_clear(renderer)
stdio.puts("ok")
```

### `COPY_ONLY`

A funcao pode ser usada, mas o OM deve copiar o resultado para memoria Snask ou esconder ponteiros crus.

Exemplo:

```snask
let platform = sdl2.get_platform()
```

Se `SDL_GetPlatform` retorna `const char*`, o Snask nao deve entregar esse ponteiro como ownership do usuario. Ele deve tratar como string segura.

### `BLOCKED`

A funcao existe no header, mas nao deve ser exposta.

Motivos comuns:

- destrutor manual;
- ponteiro cru ambiguo;
- retorno de ponteiro sem ownership claro;
- parametros `void*`;
- callback;
- out-param sem regra;
- tipo C ainda nao mapeado.

O usuario pode adicionar uma regra futura em `.om.snif`, mas a decisao padrao deve ser bloquear.

## Mapeamento de nomes

O scanner transforma nomes C em nomes Snask.

Exemplos SDL2:

```text
SDL_Init              -> sdl2.init
SDL_CreateWindow      -> sdl2.create_window
SDL_CreateRenderer    -> sdl2.create_renderer
SDL_RenderClear       -> sdl2.render_clear
SDL_RenderPresent     -> sdl2.render_present
SDL_SetRenderDrawColor -> sdl2.set_render_draw_color
SDL_WINDOW_HIDDEN     -> sdl2.WINDOW_HIDDEN
```

Regras gerais:

- prefixos de biblioteca podem ser removidos;
- CamelCase vira snake_case;
- constantes podem manter estilo maiusculo;
- o alias do import define o namespace.

## Linkagem

Para bibliotecas instaladas via `pkg-config`, o compilador pode coletar flags:

```text
pkg-config --cflags sdl2
pkg-config --libs sdl2
```

Isso permite:

- encontrar headers;
- passar include paths ao Clang;
- passar libs ao linker.

Para `import_c_om "SDL2/SDL.h" as sdl2`, o alias `sdl2` tambem e usado para tentar resolver `pkg-config`.

## Filosofia de API

O OM-Snask-System nao deve gerar uma copia literal da API C.

Uma API C geralmente expõe detalhes que existem por necessidade historica:

- ponteiros;
- buffers;
- tamanhos separados;
- structs de saida;
- erro por inteiro;
- destruicao manual;
- ordem de cleanup.

Uma API Snask deve expor intencao:

```snask
let window = sdl2.create_window(...)
let renderer = sdl2.create_renderer(window, -1, 0)
sdl2.render_present(renderer)
```

O sistema deve esconder:

```c
SDL_Window*
SDL_Renderer*
SDL_DestroyWindow
SDL_DestroyRenderer
```

quando isso puder ser provado com seguranca.

## Comparacao com bindings tradicionais

Bindings tradicionais geralmente sao escritos manualmente.

Problemas:

- custam muito tempo;
- ficam desatualizados;
- precisam repetir assinaturas;
- misturam ownership com API publica;
- exigem manutencao por biblioteca.

O OM-Snask-System tenta inverter a logica:

```text
binding manual gigante
    -> excecao

header C como fonte de verdade
    -> regra padrao

.om.snif pequeno
    -> patch quando a heuristica falha
```

Isso torna possivel mirar muitas bibliotecas C sem escrever binding completo para cada uma.

## Comparacao com FFI tradicional

FFI tradicional costuma dizer:

```text
voce pode chamar C, mas agora o problema de memoria e seu
```

O OM-Snask-System deve dizer:

```text
voce pode chamar C, e o Snask so expoe o que consegue manter sob OM
```

Essa diferenca e central.

## Como adicionar suporte a uma biblioteca nova

### Caminho ideal

Tente importar o header:

```snask
import_c_om "minhalib.h" as minhalib
```

Use funcoes simples:

```snask
minhalib.init()
minhalib.do_work(10)
```

Se a lib segue convencoes comuns (`create/free`, `open/close`, `new/destroy`), o scanner pode deduzir recursos automaticamente.

### Quando precisar de patch

Crie:

```text
contracts/minhalib.om.snif
```

Use apenas para excecoes.

Exemplo conceitual:

```snif
library minhalib

function open:
    c_function: ml_open_context
    surface: minhalib.open
    input: str
    output: resource
    safety: SAFE
    reason: library uses non-standard constructor name
```

O restante ainda vem do scanner.

## Boas praticas para `.om.snif`

Use `.om.snif` para:

- renomear uma funcao mal mapeada;
- adicionar `safety: BLOCKED` a uma funcao perigosa;
- declarar uma regra especial de recurso;
- corrigir um construtor/destrutor nao convencional;
- documentar uma decisao de ownership.

Evite `.om.snif` para:

- copiar centenas de assinaturas do header;
- expor ponteiros crus por conveniencia;
- recriar a API C inteira;
- fazer bypass da seguranca.

## Design de erro

O sistema deve falhar de forma clara.

Exemplos de erros bons:

```text
OM cannot map C parameter type `SDL_Rect*` yet.
```

```text
OM import blocked `sdl2.destroy_window`: cleanup/destructor functions are hidden behind OM zone cleanup.
```

```text
OM function `sdl2.render_clear` expects 1 C arguments, got 0.
```

O usuario deve entender se:

- a funcao nao foi encontrada;
- a funcao foi bloqueada;
- o tipo C ainda nao e suportado;
- a assinatura Snask nao bate com a assinatura C;
- falta pacote/linkagem.

## Relacao com o runtime OM

O runtime precisa oferecer pelo menos:

- registro de recurso na zona atual;
- ponteiro para destrutor;
- nome/tipo do recurso;
- extracao controlada do ponteiro para chamadas internas;
- cleanup ao sair da zona;
- trace de debug;
- ordem de destruicao consistente.

Funcoes internas relevantes:

```text
s_zone_register(ptr, destructor, type_name)
s_om_resource_ptr(resource_handle, expected_type)
```

Essas funcoes nao sao a API publica do usuario. Elas sao o contrato interno entre codegen e runtime.

## Relacao com tipos nativos

O OM-Snask-System depende do fato de Snask conseguir trabalhar com tipos nativos no LLVM:

- `i64`
- `i32`
- `double`
- ponteiros opacos
- strings como ponteiros/controladas pelo runtime

Isso e importante porque C espera ABI real, nao objetos dinamicos Snask para tudo.

Quando o Snask chama:

```snask
sdl2.delay(50)
```

o valor `50` precisa chegar como `Uint32`/inteiro C compativel, nao como uma struct dinamica.

Quando necessario, o compilador ainda pode fazer boxing/unboxing para integrar com partes dinamicas da linguagem.

## Garantias pretendidas

O sistema deve caminhar para estas garantias:

- Codigo Snask seguro nao recebe ponteiro C cru com ownership manual.
- Destrutores reconhecidos nao aparecem como funcoes publicas comuns.
- Recursos opacos sao limpos pela zona.
- Funcoes nao provadas como seguras sao bloqueadas.
- `.om.snif` e patch auditavel, nao binding manual gigante.
- Chamadas C sao emitidas como chamadas nativas no LLVM.
- O usuario escreve Snask, nao C.

## Nao garantias atuais

Hoje o sistema ainda nao garante:

- soundness total para toda API C;
- inferencia perfeita de ownership;
- suporte automatico a callbacks;
- suporte universal a structs;
- verificacao estatica completa de dependencia entre recursos;
- tratamento completo de todos os caminhos de erro de toda lib;
- API Snask ergonomica de alto nivel para cada dominio.

Isso precisa estar claro para evitar prometer mais do que existe.

## Roadmap

### Fase 1: Chamada nativa simples

Status: implementada em parte.

- scan de header;
- constantes;
- funcoes numericas/string;
- chamada LLVM direta;
- linkagem basica.

### Fase 2: Recursos opacos

Status: implementada em parte.

- construtor por retorno `Type*`;
- destrutor por `void destroy(Type*)`;
- registro em zona;
- cleanup automatico;
- metodo com recurso como primeiro parametro.

### Fase 3: Patches pequenos

Status: implementada em parte.

- `.om.snif` como patch;
- manter contrato deduzido como base;
- permitir override pontual.

### Fase 4: Structs seguras

Proximo grande salto.

Objetivo:

```snask
let rect = sdl2.Rect(10, 10, 100, 50)
sdl2.render_fill_rect(renderer, rect)
```

Por baixo:

```c
SDL_Rect rect = {10, 10, 100, 50};
SDL_RenderFillRect(renderer, &rect);
```

Sem ponteiro manual para o usuario.

### Fase 5: Eventos e callbacks

Objetivo:

```snask
while app.running:
    for event in sdl2.events():
        if event.type == sdl2.QUIT:
            app.running = false
```

Por baixo:

- stack structs;
- polling;
- copia para valores Snask;
- callbacks encapsulados quando seguro.

### Fase 6: APIs Snask idiomaticas

Objetivo final nao e apenas:

```snask
sdl2.create_window(...)
```

Mas permitir bibliotecas Snask de alto nivel:

```snask
app "Editor":
    window "Snask Editor", 900, 600:
        button "Abrir":
            print("abrir")
```

SDL2, raylib, GTK ou outra lib C podem ser motores por baixo, mas a experiencia publica deve ser Snask.

## Testes atuais importantes

Arquivos de teste relacionados:

```text
Testes/om_stdio_puts.snask
Testes/om_sdl2_platform.snask
Testes/om_sdl2_surface_zone.snask
Testes/om_sdl2_window_zone.snask
Testes/om_sdl2_renderer_zone.snask
Testes/om_sdl2_poll_event.snask
```

Comandos uteis:

```bash
cargo check
```

```bash
cargo run --quiet --bin snask -- build Testes/om_stdio_puts.snask --output test_stdio_puts
./test_stdio_puts
```

```bash
SDL_VIDEODRIVER=dummy cargo run --quiet --bin snask -- build Testes/om_sdl2_window_zone.snask --output test_sdl2_window
SNASK_OM_TRACE=1 SDL_VIDEODRIVER=dummy ./test_sdl2_window
```

```bash
cargo run --quiet --bin snask -- om scan SDL2/SDL.h --lib sdl2 --output /tmp/sdl2.generated.om.snif
```

## Criterios para promover status

Para sair de `experimental` para `parcial` ou `estavel`, o OM-Snask-System precisa:

- testes automatizados cobrindo stdio, zlib, sqlite e SDL2;
- documentacao de contrato `.om.snif`;
- regras formais para safety;
- mensagens de erro consistentes;
- suporte melhor a structs;
- story clara para callbacks;
- validacao de linkagem multiplataforma;
- fuzzing de headers ou corpus de libs C;
- testes de cleanup em caminhos de erro;
- auditoria do runtime OM.

## Principio final

O OM-Snask-System existe para permitir esta experiencia:

```snask
import_c_om "alguma_lib_c.h" as lib

class main
    fun start()
        zone "work":
            let resource = lib.create_resource()
            lib.use_resource(resource)
```

e nao esta:

```c
Resource *resource = create_resource();
if (!resource) return 1;
use_resource(resource);
destroy_resource(resource);
```

O poder vem do C.

A forma vem do Snask.

A memoria fica com o OM.

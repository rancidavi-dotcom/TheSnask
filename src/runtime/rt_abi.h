#ifndef RT_ABI_H
#define RT_ABI_H

// =========================================================================
// Snask Runtime ABI — Versão Estável
// =========================================================================
//
// Este header define o contrato de ABI entre o código LLVM gerado pelo
// compilador Snask e a runtime C. Qualquer alteração nestas definições
// REQUER um bump do SNASK_ABI_VERSION.
//
// # Convenção de versionamento
//
//   SNASK_ABI_VERSION  |  Mudança
//   -------------------+----------------------------------------------
//   1                  |  Versão inicial estável
//
// # SnaskValue (frozen)
//
//   Campo  | Índice  | Tipo    | Finalidade
//   -------+---------+---------+--------------------------------------
//   tag    | 0       | f64     | Discriminante de tipo (SnaskType)
//   num    | 1       | f64     | Payload numérico (int, bool, count)
//   ptr    | 2       | void*   | Payload de ponteiro (string, obj, resource)
//
//   Total: 24 bytes (3 × 8). ALIGNMENT: 8.
//   O código LLVM acessa via build_insert_value / build_extract_value
//   com índices 0, 1, 2 — NUNCA mude a ordem ou os tipos.
//
// # SnaskType enum (frozen)
//
//   Nome           | Valor | Rust constant
//   ---------------+-------+--------------
//   SNASK_NIL      | 0     | TYPE_NIL
//   SNASK_NUM      | 1     | TYPE_NUM
//   SNASK_BOOL     | 2     | TYPE_BOOL
//   SNASK_STR      | 3     | TYPE_STR
//   SNASK_OBJ      | 4     | TYPE_OBJ
//   SNASK_RESOURCE | 5     | TYPE_RESOURCE
//   SNASK_BYTES    | 6     | TYPE_BYTES
//
//   NUNCA reordene ou remova valores existentes. Novos tipos DEVEM
//   ser adicionados ao final com novo valor.
//
// # Funções runtime estáveis
//
//   As funções exportadas pela runtime têm ABI estável quando documentadas
//   em rt_abi.h. A assinatura (tipo de retorno, número e tipo dos
//   parâmetros) NÃO pode mudar sem bump de ABI major.
//
// # Verificação em tempo de execução
//
//   O compilador gera uma chamada para s_check_abi() no início de main().
//   Se a versão do runtime não corresponder à versão do compilador,
//   o programa aborta com mensagem de erro antes de qualquer operação.
// =========================================================================

#define SNASK_ABI_VERSION 1

// Verifica se a versão da runtime corresponde à esperada pelo compilador.
// Chamada automaticamente no início de main() — NÃO chame manualmente.
void s_check_abi(int expected_version);

#endif // RT_ABI_H

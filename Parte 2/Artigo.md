# Projeto e Implementação da Fase de Análise Sintática de um Compilador para a Linguagem TPP em Rust

**Autor:** Éric Seles Lourenço  
**Instituição:** Universidade Tecnológica Federal do Paraná (UTFPR)  
**Disciplina:** OPT011 — Implementação de Linguagens de Programação  

---

## Resumo

Este artigo apresenta o projeto e a implementação da fase de análise sintática de um compilador para a linguagem TPP, desenvolvido em Rust. A análise sintática é uma etapa crucial no processo de compilação, responsável por validar a estrutura do código-fonte com base na gramática da linguagem e gerar uma representação no formato de uma árvore sintática abstrata (AST), que será utilizada nas fases subsequentes do compilador. O trabalho aborda a construção de um analisador sintático utilizando técnicas de parsing, detalhando a gramática da linguagem TPP, a implementação do parser em Rust e os desafios enfrentados durante o desenvolvimento.

**Palavras-chave:** Compiladores, Análise Sintática, Parsing, Linguagem TPP, Rust, Logos.

## Introdução

Louden (2012) define análise sintática como a etapa do compilador que determina a sintaxe, ou estrutura, de um programa de acordo com a gramática da linguagem de programação. A análise sintática é responsável por verificar se a sequência de tokens gerada na fase de análise léxica está de acordo com as regras gramaticais da linguagem. Caso o código-fonte esteja correto, o analisador sintático constrói uma árvore sintática abstrata (AST), que representa a estrutura hierárquica do programa e serve como base para as fases subsequentes do compilador, como análise semântica e geração de código, além de ter um tratamento de erros sintáticos, que permite identificar e relatar problemas na estrutura do código-fonte, fornecendo mensagens de erro significativas para auxiliar o programador na correção dos erros.

Para a implementação da análise sintática do compilador para a linguagem TPP, será feito de forma manual, utilizando técnicas de parsing recursivo descendente. A estratégia de parsing recursivo descendente é uma abordagem top-down, onde cada regra gramatical é representada por uma função recursiva que processa os tokens de entrada e constrói a AST correspondente. Essa técnica é adequada para gramáticas livres de contexto, pois permite uma implementação direta e intuitiva do analisador sintático, facilitando a compreensão e manutenção do código.

## Metodologia

Para a implementação da análise sintática do compilador para a linguagem TPP, foi adotada uma abordagem de parsing recursivo descendente. A gramática da linguagem TPP foi cuidadosamente definida, levando em consideração as regras sintáticas e a estrutura dos programas escritos nessa linguagem. A implementação do parser em Rust envolveu a criação de funções recursivas para cada regra gramatical, permitindo a análise dos tokens gerados na fase de análise léxica e a construção da árvore sintática abstrata (AST).

### Gramática da Linguagem TPP
Segue a gramática da linguagem TPP, que define a estrutura sintática dos programas escritos nessa linguagem:

```
programa ::= lista_declaracoes
lista_declaracoes ::= lista_declaracoes declaracao NEWLINE
                      | declaracao NEWLINE
declaracao ::= declaracao_variaveis
              | inicializacao_variaveis
              | declaracao_funcao
declaracao_variaveis ::= tipo DOIS_PONTOS lista_variaveis
                         | tipo DOIS_PONTOS lista_variaveis ATRIBUICAO lista_expressoes
                         | tipo DOIS_PONTOS lista_variaveis ATRIBUICAO atribuicao_encadeada
atribuicao_encadeada ::= var ATRIBUICAO lista_expressoes
                         | var ATRIBUICAO atribuicao_encadeada
inicializacao_variaveis ::= lista_alvos ATRIBUICAO lista_expressoes
                            | ABRE_PARENTESE lista_alvos FECHA_PARENTESE ATRIBUICAO lista_expressoes
lista_variaveis ::= lista_variaveis VIRGULA var
                    | var
var ::= ID 
        | ID indice
indice ::= indice ABRE_COLCHETE expressao FECHA_COLCHETE
          | ABRE_COLCHETE expressao FECHA_COLCHETE
tipo ::= INTEIRO 
         | FLUTUANTE
declaracao_funcao ::= tipo cabecalho
                      | tipos_retorno cabecalho
                      | cabecalho
tipos_retorno ::= ABRE_PARENTESE lista_tipos FECHA_PARENTESE
lista_tipos ::= lista_tipos VIRGULA tipo
                | tipo
cabecalho ::= ID ABRE_PARENTESE lista_parametros FECHA_PARENTESE NEWLINE bloco FIM
lista_parametros ::= lista_parametros VIRGULA parametro 
                     | parametro 
                     | vazio
parametro ::= tipo DOIS_PONTOS ID 
              | parametro ABRE_COLCHETE FECHA_COLCHETE
bloco ::= corpo
          | corpo acao
corpo ::= corpo acao NEWLINE
          | vazio
acao ::= expressao
         | atribuicao
         | declaracao_variaveis
         | se 
         | repita
         | leia 
         | escreva
         | retorna
         | erro
se ::= SE expressao ENTAO bloco FIM
       | SE expressao ENTAO bloco SENAO bloco FIM
repita ::= REPITA bloco ATE expressao
atribuicao ::= lista_expressoes ATRIBUICAO lista_expressoes
               | lista_expressoes ATRIBUICAO atribuicao
lista_alvos ::= lista_alvos VIRGULA var
                | var
lista_expressoes ::= lista_expressoes VIRGULA expressao
                     | expressao
leia ::= LEIA ABRE_PARENTESE var FECHA_PARENTESE
escreva ::= ESCREVA ABRE_PARENTESE expressao FECHA_PARENTESE
retorna ::= RETORNA ABRE_PARENTESE lista_expressoes FECHA_PARENTESE
expressao ::= expressao_logica
expressao_logica ::= expressao_simples
                     | expressao_logica operador_logico expressao_simples
expressao_simples ::= expressao_aditiva
                      | expressao_simples operador_relacional expressao_aditiva
expressao_aditiva ::= expressao_multiplicativa
                      | expressao_aditiva operador_soma expressao_multiplicativa
expressao_multiplicativa ::= expressao_unaria
                             | expressao_multiplicativa operador_multiplicacao expressao_unaria
expressao_unaria ::= fator
                     | operador_soma fator
                     | operador_negacao fator
operador_relacional ::= MENOR
                        | MAIOR 
                        | IGUAL 
                        | DIFERENTE 
                        | MENOR_IGUAL 
                        | MAIOR_IGUAL
operador_soma ::= MAIS
                  | MENOS
operador_logico ::= E
                  | OU
operador_negacao ::= NAO
operador_multiplicacao ::= VEZES 
                           | DIVIDE
fator ::= ABRE_PARENTESE expressao FECHA_PARENTESE
          | var
          | chamada_funcao
          | numero
          | inicializacao_arranjo
          | tupla
tupla ::= ABRE_PARENTESE lista_expressoes VIRGULA expressao FECHA_PARENTESE
inicializacao_arranjo ::= ABRE_CHAVE lista_expressoes FECHA_CHAVE
numero ::= NUM_INTEIRO
           | NUM_PONTO_FLUTUANTE
           | NUM_NOTACAO_CIENTIFICA
chamada_funcao ::= ID ABRE_PARENTESE lista_argumentos FECHA_PARENTESE
lista_argumentos  ::= lista_argumentos VIRGULA expressao
                      | expressao
                      | vazio
```

Além dessa gramática, foi pedido para que fosse implementado uma nova funcionalidade na linguagem TPP, que é a capacidade de fazer atribuições multiplas, permitindo que uma lista de variáveis seja atribuída a uma lista de expressões. A seguinte gramática foi adicionada para suportar essa funcionalidade:

__Extensões para a Gramática:__
1. __Atribuição Múltipla__

Queríamos que fosse possível a _atribuição múltipla_, como existem em linguagens como o `Python`. A atribuição de números, variáveis, expressões ou de retornos de função.

```c++
a := 1
a := b
a,b := c,1
a,b := 1,2
(a,b) := 1,2
a,b := (1,2)
(a,b) := (1,2)
a,b := c+d,3+4
(a,b) := c+d,3+4
a,b := (c+d,3+4)
(a,b) := (c+d,3+4)
a,b := c,d
a,b := (c,d)
(a,b) := c,d
(a,b) := (c,d)
a,b := funct(x, y)
(a,b) := funct(x, y)
```

 2. __Retorno Múltiplo__

A mesma ideia da atribuição múltipla, mas para funções. A função pode retornar uma lista de valores, que podem ser atribuídos a uma lista de variáveis.

```c++
(inteiro, flutuante) funct(inteiro: x, inteiro:y)
    inteiro: res
    res := x * 2;
    retorna(res, x/y)
fim
       
inteiro principal()
    inteiro: a
    flutuante: b

    (a,b) := funct(1,2)
       
    retorna(0)
fim
```

### Implementação do Parser em Rust

A implementação da fase sintática foi organizada no módulo `src/tppparser/` (composto por `mod.rs`, `ast.rs`, `parser.rs` e `ParserErrorMessages.properties`), integrada ao binário principal em `src/main.rs` e ao gerenciador de mensagens `src/myerror.rs`.

#### 1. Representação da Árvore Sintática Abstrata (`src/tppparser/ast.rs`)

Para representar a árvore gerada pelo parser de maneira uniforme e independente das próximas fases, foi definida a estrutura `AstNode`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    pub node_type: String,
    pub children: Vec<AstNode>,
}
```

Cada nó armazena seu rótulo sintático (`node_type`, como `"Programa"`, `"DeclaracaoFuncao"`, `"ExpressionAdditive"` ou `"Literal: ..."`) e um vetor recursivo de filhos (`children`). A estrutura disponibiliza dois construtores: `AstNode::new(label)` para nós folha e `AstNode::with_position(label, line, column, children)` para nós com filhos e posição de origem.

Além do armazenamento em memória, `ast.rs` implementa duas rotinas de exportação utilizadas pela flag `-t`:

- **`to_ascii_tree(&self)` (via `fmt_ascii`):** percorre a árvore em profundidade utilizando conectores Unicode (`├── `, `└── ` e `│   `) para produzir uma visualização hierárquica legível diretamente no terminal ou em arquivo texto (`.ascii_tree.txt`).
- **`to_dot(&self)` (via `fmt_dot`):** atribui um identificador sequencial único (`n0`, `n1`, ...) a cada nó, realiza o escape de aspas nos rótulos e gera um grafo direcionado válido na linguagem DOT do Graphviz (`.ast.dot`).

#### 2. Arquitetura do Analisador Sintático (`src/tppparser/parser.rs`)

O analisador sintático é encapsulado pela `struct Parser`, que mantém o fluxo de tokens e o estado da análise:

- `tokens: Vec<Token>`: sequência linear de tokens produzida pelo `LogosLexer`.
- `pos: usize`: cursor compartilhado que avança monotonicamente sobre `tokens`.
- `check_key: bool`: controla se os erros e avisos devem ser exibidos apenas como chaves (`-k`) ou com mensagens formatadas.
- `error_handler: MyError`: instância configurada com a seção `"ParserErrors"` para formatação padronizada.

Para navegar e validar os tokens sem retrocesso excessivo (*backtracking*), o `Parser` utiliza um conjunto de métodos auxiliares:

- `current_token()` e `peek(offset)`: permitem inspecionar o token atual ou olhar $k$ posições à frente sem consumir a entrada.
- `check(kind)` e `check_at(offset, kind)`: comparam o `token_type` do token com a representação textual de `TokenKind` (`kind.as_str()`).
- `match_token(expected)`: consome o token e avança `pos` caso corresponda ao esperado.
- `expect(expected, error_key)`: exige a presença de um token; caso o fluxo tenha chegado ao fim, retorna `ERR-SYN-EOF-INESPERADO`, e caso haja outro token inesperado, formata `error_key` com linha e coluna via `format_error`.

##### Estratégias para Gramática Livre de Contexto

Como a gramática original da linguagem TPP foi especificada para parsers ascendentes (LALR/Yacc), duas adaptações foram aplicadas na tradução para *Recursive Descent*:

1. **Eliminação de recursão à esquerda:** regras como `expressao_logica`, `expressao_simples`, `expression_additive`, `expressao_multiplicativa` e `indice` foram convertidas em laços `while`. A função lê o primeiro operando de maior precedência e, enquanto o cursor apontar para um operador daquele nível, consome o operador, lê o próximo operando e agrupa os nós à esquerda. A cadeia de chamadas (`parse_expressao_logica` → `parse_expressao_simples` → `parse_expression_additive` → `parse_expressao_multiplicativa` → `parse_expressao_unaria` → `parse_expressao_primaria`) garante naturalmente a precedência correta entre operadores lógicos, relacionais, aritméticos, unários (`+`, `-`, `!`) e fatores.
2. **Desambiguação por *lookahead* (`peek` / `check_at`):**
   - Em `parse_declaracao`: quando o token atual é um tipo (`INTEIRO` ou `FLUTUANTE`), `check_at(1, TokenKind::DoisPontos)` distingue `declaracao_variaveis` (`inteiro: a`) de `declaracao_funcao` (`inteiro principal()`). Quando começa com `ABRE_PARENTESE`, `is_tipo_at(1)` separa funções com múltiplo retorno `(inteiro, flutuante) f()` de inicializações `(a, b) := 1, 2`. Quando começa com `ID`, `check_at(1, TokenKind::AbreParentese)` separa funções sem tipo explícito `principal()` de atribuições globais.
   - Em `parse_expressao_primaria`: ao encontrar `ID`, `check_at(1, TokenKind::AbreParentese)` diferencia `chamada_funcao` de `var`; ao encontrar `(`, verifica a presença de `VIRGULA` após a primeira expressão para distinguir uma expressão agrupada de uma `tupla`.
   - Em `parse_acao`: analisa `lista_expressoes` e verifica se há `ATRIBUICAO` (`:=`) em seguida, unificando expressões isoladas, atribuições simples, múltiplas e encadeadas (`parse_rhs_atribuicao`).

O ponto de entrada principal `parse()` chama `parse_programa()` e garante que não restaram tokens não consumidos após o fim do programa; adicionalmente, `parse_expressao_completa()` foi exposto para testes unitários isolados de expressões.

#### 3. Integração no `src/main.rs` e Tratamento de Erros (`src/myerror.rs`)

Para integrar a Análise Sintática sem quebrar a execução e os testes da Fase 1 (Análise Léxica), o `src/main.rs` foi atualizado com novas opções de linha de comando:

- `--parser on|yacc|off`: habilita ou desabilita a execução do parser (`run_parser`).
- `--lexer on|off`: controla a execução da impressão léxica quando o parser está desativado (`run_lexer`, padrão `true`).
- `-t`: ativa tanto `run_parser` quanto `gen_tree`, solicitando a geração da árvore sintática.
- `-k`: mantém o modo de impressão apenas de chaves de erro/aviso.

Quando `run_parser` é `false`, o compilador mantém o comportamento original da Fase 1 e imprime apenas os tokens reconhecidos pelo `LogosLexer`. Quando `run_parser` é `true`, o programa obtém todos os tokens via `lexer.tokenize(&source_file)` e executa `Parser::new(tokens, check_key).parse()`:

- **Em caso de sucesso (`Ok(ast)`):** se `-t` estiver ativo, imprime `WAR-SYN-GEN-SYNTAX-TREE`, grava os arquivos `<fonte>.ast.dot` e `<fonte>.ascii_tree.txt` ao lado do arquivo-fonte, imprime `WAR-SYN-OUTPUT-FILE` (e exibe a árvore ASCII no console quando `-k` não é usado) e finaliza com `WAR-SYN-ANA-SUCCESS`.
- **Em caso de erro (`Err(err)`):** imprime a mensagem ou chave de erro retornada pelo parser e, se `-t` estiver ativo, imprime `WAR-SYN-NOT-GEN-SYNTAX-TREE`.

No módulo `src/myerror.rs`, foi adicionado o carregamento embutido de `DEFAULT_PARSER_ERRORS` (`include_str!("tppparser/ParserErrorMessages.properties")`) para garantir que todas as chaves sintáticas funcionem independentemente do diretório de execução do binário.

## Resultados e Testes

A validação do compilador foi automatizada em `tests/tppparser_test.rs` e executada com `cargo test`, em conjunto com as suítes já existentes da Fase 1:

- **Suíte Léxica (`tests/tpplex_test.rs`):** 42 testes passando sem regressões.
- **Suíte de Performance (`tests/test_perf_large_tokens.rs`):** 1 teste passando.
- **Suíte Sintática (`tests/tppparser_test.rs`):** 14 testes passando, cobrindo precedência aritmética, operadores relacionais e lógicos, ponto flutuante e notação científica, programas completos com múltiplo retorno, arrays e `repita ... até`, detecção de erros sintáticos (`ERR-SYN-EOF-INESPERADO`, `ERR-SYN-INDICE`, `ERR-SYN-LEIA`, `ERR-SYN-IRRECUPERAVEL`), erros globais de CLI e os 10 casos válidos da bateria oficial `tests/syn-tests/` executados via binário (`--parser on -k -t`).

Nos casos inválidos da bateria `tests/syn-tests/`, o parser identifica corretamente os erros sintáticos e impede a geração da árvore (`WAR-SYN-NOT-GEN-SYNTAX-TREE`), havendo diferença apenas em testes que esperam recuperação múltipla de erros específica do gerador PLY/Yacc utilizado como referência.

## Conclusão

A implementação manual do analisador sintático descendente recursivo em Rust permitiu cobrir integralmente as regras da gramática da linguagem TPP e suas extensões de atribuição e retorno múltiplos. A separação clara entre a representação da árvore (`ast.rs`), a lógica de parsing (`parser.rs`) e a orquestração via CLI (`main.rs`) manteve o compilador modular, tipado e preparado para a próxima etapa do projeto: a Análise Semântica.


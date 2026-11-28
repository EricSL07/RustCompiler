# RustCompiler — Compilador TPP em Rust

Compilador para a linguagem **TPP** implementado em **Rust**, contemplando atualmente as três primeiras fases do pipeline de compilação:

1. **Análise Léxica (`tpplexer`):** construída com o crate **`logos`** e validada com os 42 casos de teste léxicos.
2. **Análise Sintática (`tppparser`):** parser recursivo descendente manual com geração da Árvore Sintática Abstrata (AST) e tratamento de erros sintáticos.
3. **Análise Semântica (`tppsemantic`):** tabela de símbolos (`SymbolTable`), validação semântica (escopos, tipos, coerções implícitas, arranjos, funções e retornos múltiplos) e geração da AST Podada e Anotada (ASTO).

## Pré-requisitos (Instalando Rust e Cargo)

Caso ainda não tenha o **Rust** e o **Cargo** instalados na máquina:

- **Linux / macOS:**
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
  ```
- **Windows:** baixe e execute o instalador oficial em [rustup.rs](https://rustup.rs/).

Para confirmar que a instalação funcionou:
```bash
cargo --version
```

## Estrutura do Projeto

```text
RustCompiler/
├── Parte 1/                                # Documentação e artigo da Análise Léxica
├── Parte 2/                                # Documentação, gramática e artigo da Análise Sintática
├── Parte 3/                                # Especificação (SEM-SPECS.md) e artigo da Análise Semântica
├── Cargo.toml                              # Manifesto do projeto Cargo
├── README.md                               # Documentação do projeto
├── src/
│   ├── main.rs                             # Ponto de entrada da CLI (tpp_compiler)
│   ├── lib.rs                              # Biblioteca principal exportando os módulos
│   ├── myerror.rs                          # Gerenciador de erros e leitura dos .properties
│   ├── GlobalErrorMessages.properties      # Mensagens de erro globais
│   ├── tpplexer/                           # Fase 1: Análise Léxica (baseada em Logos)
│   │   ├── mod.rs
│   │   ├── base.rs
│   │   ├── tokens.rs
│   │   ├── logos_lexer.rs
│   │   ├── log_error_lexer.rs
│   │   └── LexerErrorMessages.properties
│   ├── tppparser/                          # Fase 2: Análise Sintática (Parser + AST)
│   │   ├── mod.rs
│   │   ├── ast.rs
│   │   ├── parser.rs
│   │   └── ParserErrorMessages.properties
│   └── tppsemantic/                        # Fase 3: Análise Semântica (Tabela de Símbolos + ASTO)
│       ├── mod.rs
│       ├── symbol_table.rs
│       ├── sema.rs
│       ├── rules/
│       └── SemaErrorMessages.properties
└── tests/
    ├── tpplex_test.rs                      # Testes da Análise Léxica
    ├── tppparser_test.rs                   # Testes da Análise Sintática
    ├── tppsema_test.rs                     # Testes da Análise Semântica
    ├── test_perf_large_tokens.rs           # Teste de performance (>36.000 tokens)
    ├── lex-tests/                          # Casos de teste léxicos (.tpp e .lex.out)
    ├── syn-tests/                          # Casos de teste sintáticos (.tpp e .syn.out)
    └── sem-tests/                          # Casos de teste semânticos (.tpp e .sem.out)
```

## Como Executar o Compilador

### Exibir ajuda da CLI
```bash
cargo run -- --help
```

### Análise Léxica (padrão)
```bash
# Mensagens de erro detalhadas com linha e coluna:
cargo run -- tests/lex-tests/lex-test-002.tpp

# Flag -k (imprime apenas as chaves de erro, ex.: ERR-LEX-INV-CHAR):
cargo run -- -k tests/lex-tests/lex-test-002.tpp
```

### Análise Sintática e AST
```bash
# Executar o parser:
cargo run -- --parser on tests/syn-tests/syn-test-001.tpp

# Imprimir a Árvore Sintática Abstrata (AST) com -t:
cargo run -- -t tests/syn-tests/syn-test-001.tpp
```

### Análise Semântica
```bash
# Mensagens semânticas legíveis:
cargo run -- --sema on tests/sem-tests/sem-test-001.tpp

# Apenas chaves de erro/aviso (-k):
cargo run -- -k --sema on tests/sem-tests/sem-test-001.tpp
```

## Como Executar os Testes

Para rodar toda a suíte de testes:
```bash
cargo test
```

Para rodar cada fase separadamente:
```bash
cargo test --test tpplex_test
cargo test --test tppparser_test
cargo test --test tppsema_test
```

Para executar o teste de performance mostrando a vazão em tokens/s no terminal:
```bash
cargo test --test test_perf_large_tokens -- --nocapture
```

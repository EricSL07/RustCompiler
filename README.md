# RustCompiler — Compilador TPP em Rust

Esqueleto completo do compilador para a linguagem **TPP** implementado em **Rust**, estruturado com base na arquitetura modular de referência (`tppcompiler`) e contendo a **1ª Fase (Análise Léxica)** completa e integrada com os **42 casos de teste do professor**.

## Estrutura do Projeto

```text
RustCompiler/
├── Artigo.md                               # Artigo sobre a construção do compilador em Rust
├── Cargo.toml                              # Manifesto do projeto Cargo
├── README.md                               # Documentação do projeto
├── src/
│   ├── main.rs                             # Ponto de entrada da CLI (tpp_compiler)
│   ├── lib.rs                              # Biblioteca principal exportando os módulos
│   ├── myerror.rs                          # Gerenciador de erros e leitura dos .properties
│   ├── GlobalErrorMessages.properties      # Mensagens de erro globais
│   ├── tpplexer/                           # Fase 1: Análise Léxica
│   │   ├── mod.rs                          # Factory get_lexer e exportações
│   │   ├── base.rs                         # Trait BaseLexer, struct Token e utilitários
│   │   ├── tokens.rs                       # Definições de Tokens, palavras reservadas e operadores
│   │   ├── log_error_lexer.rs              # Tratamento e formatação de erros léxicos
│   │   ├── LexerErrorMessages.properties   # Mensagens de erro léxicas
│   │   ├── ply_lexer.rs                    # Estratégia padrão compatível com PLY (--lexer ply)
│   │   ├── mandfa_lexer.rs                 # Estratégia AFD manual procedural (--lexer mandfa)
│   │   ├── symtable_lexer.rs               # Estratégias com Tabela de Símbolos (--lexer symtable / symtableman)
│   │   └── automatalib_lexer.rs            # Estratégias com AFDs formais e Maximal Munch (--lexer automatalib / automatalibman)
│   ├── tppparser/                          # Fase 2: Análise Sintática (Esqueleto)
│   │   ├── mod.rs                          # Trait BaseParser e SyntaxTree
│   │   └── ParserMessages.properties       # Mensagens de erro sintáticas
│   ├── tppsema/                            # Fase 3: Análise Semântica (Esqueleto)
│   │   └── mod.rs                          # Esqueleto do SemanticAnalyzer
│   └── tppcodegen/                         # Fase 4: Geração de Código (Esqueleto)
│       └── mod.rs                          # Esqueleto do CodeGenerator
└── tests/
    ├── tpplex_test.rs                      # Suíte automatizada com os 42 testes do professor
    ├── test_perf_large_tokens.rs           # Testes de performance e paridade (>36.000 tokens)
    └── lex-tests/                          # Os 39 arquivos .tpp e 42 saídas esperadas (.lex.out)
```

## Como Executar o Compilador

### Exibir ajuda da CLI
```bash
cargo run -- --help
```

### Executar a Análise Léxica em um arquivo `.tpp`
```bash
# Execução normal (imprime mensagens de erro detalhadas com linha e coluna):
cargo run -- tests/lex-tests/lex-test-002.tpp

# Execução com flag -k (imprime apenas as chaves de erro, ex.: ERR-LEX-INV-CHAR):
cargo run -- --lexer ply -k tests/lex-tests/lex-test-002.tpp

# Escolhendo a estratégia léxica (ply, mandfa, symtable, symtableman, automatalib, automatalibman):
cargo run -- --lexer mandfa tests/lex-tests/lex-test-001.tpp
cargo run -- --lexer symtable tests/lex-tests/lex-test-001.tpp
cargo run -- --lexer automatalib tests/lex-tests/lex-test-001.tpp
```

## Como Executar os Testes do Professor

Para rodar toda a suíte de testes (os 42 testes do professor + validação de todas as estratégias + teste de stress/performance):

```bash
cargo test
```

Para executar apenas a suíte de testes léxicos do professor (`tpplex_test`):

```bash
cargo test --test tpplex_test
```

Para executar os testes de performance mostrando a vazão em tokens/s no terminal:

```bash
cargo test --test test_perf_large_tokens -- --nocapture
```

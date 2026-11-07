# RustCompiler — Compilador TPP em Rust

Compilador para a linguagem **TPP** implementado em **Rust**, contendo a **1ª Fase (Análise Léxica)** construída com o crate **`logos`** (padrão do ecossistema Rust para geração de analisadores léxicos em tempo de compilação) e integrada com os **42 casos de teste do professor**.

## Estrutura do Projeto

```text
RustCompiler/
├── Parte 1/
│   └── Artigo.md                           # Artigo sobre a construção do analisador léxico em Rust com Logos
├── Cargo.toml                              # Manifesto do projeto Cargo (dependência: logos)
├── README.md                               # Documentação do projeto
├── src/
│   ├── main.rs                             # Ponto de entrada da CLI (tpp_compiler)
│   ├── lib.rs                              # Biblioteca principal exportando os módulos
│   ├── myerror.rs                          # Gerenciador de erros e leitura dos .properties
│   ├── GlobalErrorMessages.properties      # Mensagens de erro globais
│   ├── tpplexer/                           # Fase 1: Análise Léxica (baseada em Logos)
│   │   ├── mod.rs                          # Organização e exportações do lexer
│   │   ├── base.rs                         # Trait BaseLexer, struct Token e utilitários
│   │   ├── tokens.rs                       # Enum TokenKind com #[derive(Logos)] e regras léxicas
│   │   ├── logos_lexer.rs                  # Implementação do LogosLexer (rastreamento de linha/coluna e erros)
│   │   ├── log_error_lexer.rs              # Tratamento e formatação de erros léxicos
│   │   └── LexerErrorMessages.properties   # Mensagens de erro léxicas
└── tests/
    ├── tpplex_test.rs                      # Suíte automatizada com os 42 testes do professor
    ├── test_perf_large_tokens.rs           # Teste de performance (>36.000 tokens)
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
cargo run -- -k tests/lex-tests/lex-test-002.tpp
```

## Como Executar os Testes do Professor

Para rodar toda a suíte de testes (os 42 testes do professor + teste de stress/performance):

```bash
cargo test
```

Para executar apenas a suíte de testes léxicos do professor (`tpplex_test`):

```bash
cargo test --test tpplex_test
```

Para executar o teste de performance mostrando a vazão em tokens/s no terminal:

```bash
cargo test --test test_perf_large_tokens -- --nocapture
```

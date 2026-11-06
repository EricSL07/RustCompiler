use logos::Logos;

/// Enumeração das classes léxicas (Tokens) da linguagem TPP utilizando o crate `logos`.
///
/// A macro procedural `#[derive(Logos)]` compila todas as regras `#[token]` e `#[regex]`
/// em tempo de compilação para um Autômato Finito Determinístico (AFD) otimizado em tabelas
/// de salto (jump tables), aplicando automaticamente a regra do Prefixo Mais Longo (Maximal Munch)
/// e desambiguação por prioridade.
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // 1. Quebra de linha e espaços em branco
    #[token("\n")]
    Newline,

    #[regex(r"[ \t\r]+", priority = 1)]
    Whitespace,

    // 2. Abertura de comentário de bloco `{ ... }`
    #[token("{")]
    CommentStart,

    // 3. Palavras Reservadas da linguagem TPP
    #[token("se")]
    Se,
    #[token("então")]
    Entao,
    #[token("senão")]
    Senao,
    #[token("fim")]
    Fim,
    #[token("repita")]
    Repita,
    #[token("flutuante")]
    Flutuante,
    #[token("retorna")]
    Retorna,
    #[token("até")]
    Ate,
    #[token("leia")]
    Leia,
    #[token("escreva")]
    Escreva,
    #[token("inteiro")]
    Inteiro,

    // 4. Identificadores (`ID`): letra (ASCII ou acentuada da TPP) seguida de letras, dígitos ou '_'
    #[regex(r"[a-zA-ZáÁãÃàÀéÉíÍóÓõÕ][a-zA-ZáÁãÃàÀéÉíÍóÓõÕ0-9_]*", priority = 2)]
    Id,

    // 5. Literais Numéricos (Notação Científica, Ponto Flutuante e Inteiro)
    #[regex(r"[1-9]\.[0-9]+[eE][-+]?[0-9]+", priority = 30)]
    NumNotacaoCientifica,

    #[regex(
        r"([0-9]+\.[0-9]*|\.[0-9]+)([eE][-+]?[0-9]+)?|[0-9]+[eE][-+]?[0-9]+",
        priority = 20
    )]
    NumPontoFlutuante,

    #[regex(r"[0-9]+", priority = 10)]
    NumInteiro,

    // 6. Operadores Relacionais, Lógicos, Aritméticos e Delimitadores
    #[token(":=")]
    Atribuicao,
    #[token("<=")]
    MenorIgual,
    #[token(">=")]
    MaiorIgual,
    #[token("<>")]
    Diferente,
    #[token("&&")]
    E,
    #[token("||")]
    Ou,
    #[token("!")]
    Nao,
    #[token("+")]
    Mais,
    #[token("-")]
    Menos,
    #[token("*")]
    Vezes,
    #[token("/")]
    Divide,
    #[token("<")]
    Menor,
    #[token(">")]
    Maior,
    #[token("=")]
    Igual,
    #[token("(")]
    AbreParentese,
    #[token(")")]
    FechaParentese,
    #[token("[")]
    AbreColchete,
    #[token("]")]
    FechaColchete,
    #[token(",")]
    Virgula,
    #[token(":")]
    DoisPontos,

    // 7. Fallback para qualquer caractere UTF-8 isolado inválido (ex.: 'ç', '$', '&', '|')
    #[regex(r".", priority = 0)]
    InvalidChar,
}

impl TokenKind {
    /// Retorna a representação textual padronizada do Token na linguagem TPP.
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            Self::Se => Some("SE"),
            Self::Entao => Some("ENTAO"),
            Self::Senao => Some("SENAO"),
            Self::Fim => Some("FIM"),
            Self::Repita => Some("REPITA"),
            Self::Flutuante => Some("FLUTUANTE"),
            Self::Retorna => Some("RETORNA"),
            Self::Ate => Some("ATE"),
            Self::Leia => Some("LEIA"),
            Self::Escreva => Some("ESCREVA"),
            Self::Inteiro => Some("INTEIRO"),
            Self::Id => Some("ID"),
            Self::NumNotacaoCientifica => Some("NUM_NOTACAO_CIENTIFICA"),
            Self::NumPontoFlutuante => Some("NUM_PONTO_FLUTUANTE"),
            Self::NumInteiro => Some("NUM_INTEIRO"),
            Self::Atribuicao => Some("ATRIBUICAO"),
            Self::MenorIgual => Some("MENOR_IGUAL"),
            Self::MaiorIgual => Some("MAIOR_IGUAL"),
            Self::Diferente => Some("DIFERENTE"),
            Self::E => Some("E"),
            Self::Ou => Some("OU"),
            Self::Nao => Some("NAO"),
            Self::Mais => Some("MAIS"),
            Self::Menos => Some("MENOS"),
            Self::Vezes => Some("VEZES"),
            Self::Divide => Some("DIVIDE"),
            Self::Menor => Some("MENOR"),
            Self::Maior => Some("MAIOR"),
            Self::Igual => Some("IGUAL"),
            Self::AbreParentese => Some("ABRE_PARENTESE"),
            Self::FechaParentese => Some("FECHA_PARENTESE"),
            Self::AbreColchete => Some("ABRE_COLCHETE"),
            Self::FechaColchete => Some("FECHA_COLCHETE"),
            Self::Virgula => Some("VIRGULA"),
            Self::DoisPontos => Some("DOIS_PONTOS"),
            Self::Newline | Self::Whitespace | Self::CommentStart | Self::InvalidChar => None,
        }
    }
}

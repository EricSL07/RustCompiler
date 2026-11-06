use std::fmt;

/// Representação padronizada de um Token no compilador TPP.
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: String,
    pub value: String,
    pub lineno: usize,
    pub lexpos: usize,
}

impl Token {
    pub fn new(
        token_type: impl Into<String>,
        value: impl Into<String>,
        lineno: usize,
        lexpos: usize,
    ) -> Self {
        Self {
            token_type: token_type.into(),
            value: value.into(),
            lineno,
            lexpos,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
            && self.value == other.value
            && self.lineno == other.lineno
    }
}

impl Eq for Token {}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token({}, {:?}, line={}, pos={})",
            self.token_type, self.value, self.lineno, self.lexpos
        )
    }
}

/// Trait que define a interface do Analisador Léxico (Lexer) do compilador TPP.
pub trait BaseLexer {
    /// Carrega a string do código-fonte TPP no lexer.
    fn input(&mut self, data: &str);

    /// Retorna o próximo `Token` da entrada ou `None` quando atingir o Fim de Arquivo (EOF).
    fn token(&mut self) -> Option<Token>;

    /// Consome toda a entrada e retorna a lista de `Token`s reconhecidos.
    fn tokenize(&mut self, data: &str) -> Vec<Token> {
        self.input(data);
        let mut tokens = Vec::new();
        while let Some(tok) = self.token() {
            tokens.push(tok);
        }
        tokens
    }
}

/// Calcula a coluna (1-indexed, em caracteres Unicode) a partir do início da linha atual (`line_start_byte`)
/// até a posição atual (`token_start_byte`) na fatia UTF-8 `source`.
pub fn get_column_str(source: &str, line_start_byte: usize, token_start_byte: usize) -> usize {
    let start = line_start_byte.min(source.len());
    let end = token_start_byte.min(source.len()).max(start);
    source[start..end].chars().count() + 1
}

/// Normaliza quebras de linha `\r\n` para `\n`.
pub fn normalize_source_str(data: &str) -> String {
    data.replace("\r\n", "\n")
}

/// Verifica se o início de `slice` corresponde a um literal em Notação Científica:
/// Padrão: `^[\-\+]?[1-9]\.[0-9]+[eE][\-\+]?[0-9]+`
/// Retorna o comprimento em bytes do literal reconhecido.
pub fn match_scientific_notation_str(slice: &str) -> Option<usize> {
    let bytes = slice.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    if i >= n {
        return None;
    }

    if bytes[i] == b'+' || bytes[i] == b'-' {
        i += 1;
        if i >= n {
            return None;
        }
    }

    if !matches!(bytes[i], b'1'..=b'9') {
        return None;
    }
    i += 1;

    if i >= n || bytes[i] != b'.' {
        return None;
    }
    i += 1;

    let frac_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == frac_start {
        return None;
    }

    if i >= n || (bytes[i] != b'e' && bytes[i] != b'E') {
        return None;
    }
    i += 1;

    if i < n && (bytes[i] == b'+' || bytes[i] == b'-') {
        i += 1;
    }

    let exp_start = i;
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == exp_start {
        return None;
    }

    Some(i)
}

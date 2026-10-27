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
    pub fn new(token_type: impl Into<String>, value: impl Into<String>, lineno: usize, lexpos: usize) -> Self {
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

/// Trait que define a interface obrigatória de todos os Analisadores Léxicos (Lexers).
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

/// Calcula a coluna (1-indexed) a partir da posição `lexpos` (em caracteres) no vetor de caracteres.
pub fn get_column(chars: &[char], lexpos: usize) -> usize {
    let limit = lexpos.min(chars.len());
    let begin_line = chars[..limit]
        .iter()
        .rposition(|&c| c == '\n')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    (limit - begin_line) + 1
}

/// Normaliza quebras de linha `\r\n` para `\n` (equivalente ao universal newlines do Python).
pub fn normalize_source(data: &str) -> Vec<char> {
    data.replace("\r\n", "\n").chars().collect()
}

/// Verifica se um caractere pertence ao conjunto de letras válidas na linguagem TPP.
#[inline]
pub fn is_tpp_letter(c: char) -> bool {
    matches!(
        c,
        'a'..='z'
            | 'A'..='Z'
            | 'á'
            | 'Á'
            | 'ã'
            | 'Ã'
            | 'à'
            | 'À'
            | 'é'
            | 'É'
            | 'í'
            | 'Í'
            | 'ó'
            | 'Ó'
            | 'õ'
            | 'Õ'
    )
}

/// Verifica se um caractere é válido no corpo de um identificador TPP (`letra | digito | _`).
#[inline]
pub fn is_tpp_alphanum_under(c: char) -> bool {
    is_tpp_letter(c) || c.is_ascii_digit() || c == '_'
}

/// Tenta casar um literal em Notação Científica no início de `slice`:
/// Padrão: `^[\-\+]?[1-9]\.[0-9]+[eE][\-\+]?[0-9]+`
pub fn match_scientific_notation(slice: &[char]) -> Option<usize> {
    let mut i = 0;
    let n = slice.len();
    if i >= n {
        return None;
    }

    // Sinal opcional (+ ou -)
    if slice[i] == '+' || slice[i] == '-' {
        i += 1;
        if i >= n {
            return None;
        }
    }

    // Primeiro dígito deve ser [1-9]
    if !matches!(slice[i], '1'..='9') {
        return None;
    }
    i += 1;

    // Ponto decimal obrigatório '.'
    if i >= n || slice[i] != '.' {
        return None;
    }
    i += 1;

    // Pelo menos um dígito após o ponto: [0-9]+
    let frac_start = i;
    while i < n && slice[i].is_ascii_digit() {
        i += 1;
    }
    if i == frac_start {
        return None;
    }

    // Letra de expoente [eE]
    if i >= n || (slice[i] != 'e' && slice[i] != 'E') {
        return None;
    }
    i += 1;

    // Sinal opcional do expoente (+ ou -)
    if i < n && (slice[i] == '+' || slice[i] == '-') {
        i += 1;
    }

    // Pelo menos um dígito no expoente: [0-9]+
    let exp_start = i;
    while i < n && slice[i].is_ascii_digit() {
        i += 1;
    }
    if i == exp_start {
        return None;
    }

    Some(i)
}

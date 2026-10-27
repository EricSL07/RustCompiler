use crate::tpplexer::base::{
    get_column, is_tpp_alphanum_under, is_tpp_letter, match_scientific_notation, normalize_source,
    BaseLexer, Token,
};
use crate::tpplexer::log_error_lexer::LogErrorLexer;
use crate::tpplexer::tokens::resolve_keyword_or_id;

/// Analisador Léxico manual baseado em Autômato Finito Determinístico (AFD / DFA).
/// Percorre a cadeia de entrada caractere a caractere utilizando lookahead para transições
/// de estado, resolvendo palavras reservadas, identificadores, numerais, operadores compostos,
/// comentários de bloco e erros léxicos padronizados.
#[derive(Debug, Clone)]
pub struct ManualDFALexer {
    pub check_key: bool,
    pub log_err: LogErrorLexer,
    pub chars: Vec<char>,
    pub pos: usize,
    pub lineno: usize,
}

impl ManualDFALexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            check_key,
            log_err: LogErrorLexer::new(),
            chars: Vec::new(),
            pos: 0,
            lineno: 1,
        }
    }

    #[inline]
    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn report_error(&self, key: &str, lineno: usize, lexpos: usize, valor: Option<&str>) {
        let col = get_column(&self.chars, lexpos);
        let msg = self
            .log_err
            .format_error(self.check_key, key, lineno, col, valor);
        println!("{}", msg);
    }

    fn slice_to_string(&self, start: usize, end: usize) -> String {
        self.chars[start..end].iter().collect()
    }
}

impl BaseLexer for ManualDFALexer {
    fn input(&mut self, data: &str) {
        self.chars = normalize_source(data);
        self.pos = 0;
        self.lineno = 1;
    }

    fn token(&mut self) -> Option<Token> {
        let len = self.chars.len();

        while self.pos < len {
            let c = self.chars[self.pos];

            // 1. Espaços em branco e quebras de linha
            if c == '\n' {
                self.lineno += 1;
                self.pos += 1;
                continue;
            } else if c == ' ' || c == '\t' || c == '\r' {
                self.pos += 1;
                continue;
            }

            // 2. Comentários: { ... }
            if c == '{' {
                let start_pos = self.pos;
                let start_lineno = self.lineno;
                self.pos += 1;
                let mut fechado = false;

                while self.pos < len {
                    let ch = self.chars[self.pos];
                    if ch == '\n' {
                        self.lineno += 1;
                    } else if ch == '}' {
                        self.pos += 1;
                        fechado = true;
                        break;
                    }
                    self.pos += 1;
                }

                if !fechado {
                    self.report_error("ERR-LEX-UNC-COMMENT", start_lineno, start_pos, None);
                    return None;
                }
                continue;
            }

            let start_pos = self.pos;
            let start_lineno = self.lineno;

            // 3. Notação Científica iniciada por sinal (+ ou -)
            if (c == '+' || c == '-')
                && matches!(self.peek(1), Some('1'..='9'))
            {
                if let Some(match_len) = match_scientific_notation(&self.chars[self.pos..]) {
                    let lexeme = self.slice_to_string(start_pos, start_pos + match_len);
                    self.pos += match_len;
                    return Some(Token::new(
                        "NUM_NOTACAO_CIENTIFICA",
                        lexeme,
                        start_lineno,
                        start_pos,
                    ));
                }
            }

            // 4. Identificadores e Palavras Reservadas: [letra][letra|digito|_]*
            if is_tpp_letter(c) {
                self.pos += 1;
                while self.pos < len && is_tpp_alphanum_under(self.chars[self.pos]) {
                    self.pos += 1;
                }
                let lexeme = self.slice_to_string(start_pos, self.pos);
                let tok_type = resolve_keyword_or_id(&lexeme);
                return Some(Token::new(tok_type, lexeme, start_lineno, start_pos));
            }

            // 5. Números (Inteiro, Ponto Flutuante, Notação Científica)
            if c.is_ascii_digit()
                || (c == '.' && matches!(self.peek(1), Some(d) if d.is_ascii_digit()))
            {
                if let Some(match_len) = match_scientific_notation(&self.chars[self.pos..]) {
                    let lexeme = self.slice_to_string(start_pos, start_pos + match_len);
                    self.pos += match_len;
                    return Some(Token::new(
                        "NUM_NOTACAO_CIENTIFICA",
                        lexeme,
                        start_lineno,
                        start_pos,
                    ));
                }

                let mut has_dot = false;
                let mut has_exp = false;

                if c == '.' {
                    has_dot = true;
                    self.pos += 1;
                }

                while self.pos < len {
                    let curr = self.chars[self.pos];
                    if curr.is_ascii_digit() {
                        self.pos += 1;
                    } else if curr == '.' && !has_dot && !has_exp {
                        has_dot = true;
                        self.pos += 1;
                    } else if (curr == 'e' || curr == 'E') && !has_exp {
                        let next1 = self.peek(1);
                        let next2 = self.peek(2);
                        if matches!(next1, Some(d) if d.is_ascii_digit()) {
                            has_exp = true;
                            self.pos += 2;
                        } else if matches!(next1, Some('+' | '-'))
                            && matches!(next2, Some(d) if d.is_ascii_digit())
                        {
                            has_exp = true;
                            self.pos += 3;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let lexeme = self.slice_to_string(start_pos, self.pos);
                let tok_type = if has_dot || has_exp {
                    "NUM_PONTO_FLUTUANTE"
                } else {
                    "NUM_INTEIRO"
                };
                return Some(Token::new(tok_type, lexeme, start_lineno, start_pos));
            }

            // 6. Operadores Relacionais e Atribuição
            if c == ':' {
                self.pos += 1;
                if self.peek(0) == Some('=') {
                    self.pos += 1;
                    return Some(Token::new("ATRIBUICAO", ":=", start_lineno, start_pos));
                }
                return Some(Token::new("DOIS_PONTOS", ":", start_lineno, start_pos));
            }

            if c == '<' {
                self.pos += 1;
                match self.peek(0) {
                    Some('=') => {
                        self.pos += 1;
                        return Some(Token::new("MENOR_IGUAL", "<=", start_lineno, start_pos));
                    }
                    Some('>') => {
                        self.pos += 1;
                        return Some(Token::new("DIFERENTE", "<>", start_lineno, start_pos));
                    }
                    _ => return Some(Token::new("MENOR", "<", start_lineno, start_pos)),
                }
            }

            if c == '>' {
                self.pos += 1;
                if self.peek(0) == Some('=') {
                    self.pos += 1;
                    return Some(Token::new("MAIOR_IGUAL", ">=", start_lineno, start_pos));
                }
                return Some(Token::new("MAIOR", ">", start_lineno, start_pos));
            }

            if c == '=' {
                self.pos += 1;
                return Some(Token::new("IGUAL", "=", start_lineno, start_pos));
            }

            // 7. Operadores Lógicos
            if c == '&' {
                self.pos += 1;
                if self.peek(0) == Some('&') {
                    self.pos += 1;
                    return Some(Token::new("E", "&&", start_lineno, start_pos));
                }
                self.report_error("ERR-LEX-INV-CHAR", start_lineno, start_pos, Some("&"));
                continue;
            }

            if c == '|' {
                self.pos += 1;
                if self.peek(0) == Some('|') {
                    self.pos += 1;
                    return Some(Token::new("OU", "||", start_lineno, start_pos));
                }
                self.report_error("ERR-LEX-INV-CHAR", start_lineno, start_pos, Some("|"));
                continue;
            }

            if c == '!' {
                self.pos += 1;
                return Some(Token::new("NAO", "!", start_lineno, start_pos));
            }

            // 8. Operadores Aritméticos
            if c == '+' {
                self.pos += 1;
                return Some(Token::new("MAIS", "+", start_lineno, start_pos));
            }

            if c == '-' {
                self.pos += 1;
                return Some(Token::new("MENOS", "-", start_lineno, start_pos));
            }

            if c == '*' {
                self.pos += 1;
                return Some(Token::new("VEZES", "*", start_lineno, start_pos));
            }

            if c == '/' {
                self.pos += 1;
                return Some(Token::new("DIVIDE", "/", start_lineno, start_pos));
            }

            // 9. Delimitadores
            if c == '(' {
                self.pos += 1;
                return Some(Token::new("ABRE_PARENTESE", "(", start_lineno, start_pos));
            }

            if c == ')' {
                self.pos += 1;
                return Some(Token::new("FECHA_PARENTESE", ")", start_lineno, start_pos));
            }

            if c == '[' {
                self.pos += 1;
                return Some(Token::new("ABRE_COLCHETE", "[", start_lineno, start_pos));
            }

            if c == ']' {
                self.pos += 1;
                return Some(Token::new("FECHA_COLCHETE", "]", start_lineno, start_pos));
            }

            if c == ',' {
                self.pos += 1;
                return Some(Token::new("VIRGULA", ",", start_lineno, start_pos));
            }

            // 10. Caractere Inválido
            let s = c.to_string();
            self.report_error("ERR-LEX-INV-CHAR", start_lineno, start_pos, Some(&s));
            self.pos += 1;
        }

        None
    }
}

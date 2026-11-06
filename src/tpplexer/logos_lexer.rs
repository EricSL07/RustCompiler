use logos::Logos;

use crate::tpplexer::base::{
    get_column_str, match_scientific_notation_str, normalize_source_str, BaseLexer, Token,
};
use crate::tpplexer::log_error_lexer::LogErrorLexer;
use crate::tpplexer::tokens::TokenKind;

/// Analisador Léxico da linguagem TPP baseado no crate `logos` (padrão de mercado em Rust).
///
/// Utiliza o AFD gerado em tempo de compilação por `#[derive(Logos)]` sobre `TokenKind`,
/// mantendo o rastreamento de linha (`lineno`), coluna (`line_start_byte`) e posição (`char_pos`),
/// além do tratamento de comentários multilinha `{ ... }` e recuperação de erros léxicos.
#[derive(Debug, Clone)]
pub struct LogosLexer {
    pub check_key: bool,
    pub log_err: LogErrorLexer,
    pub source: String,
    pub byte_pos: usize,
    pub char_pos: usize,
    pub lineno: usize,
    pub line_start_byte: usize,
}

impl LogosLexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            check_key,
            log_err: LogErrorLexer::new(),
            source: String::new(),
            byte_pos: 0,
            char_pos: 0,
            lineno: 1,
            line_start_byte: 0,
        }
    }

    fn report_error(&self, key: &str, lineno: usize, token_start_byte: usize, valor: Option<&str>) {
        let col = get_column_str(&self.source, self.line_start_byte, token_start_byte);
        let msg = self
            .log_err
            .format_error(self.check_key, key, lineno, col, valor);
        println!("{}", msg);
    }
}

impl BaseLexer for LogosLexer {
    fn input(&mut self, data: &str) {
        self.source = normalize_source_str(data);
        self.byte_pos = 0;
        self.char_pos = 0;
        self.lineno = 1;
        self.line_start_byte = 0;
    }

    fn token(&mut self) -> Option<Token> {
        let total_len = self.source.len();

        while self.byte_pos < total_len {
            let remaining = &self.source[self.byte_pos..];
            let mut lex = TokenKind::lexer(remaining);

            let Some(result) = lex.next() else {
                break;
            };

            let span = lex.span();
            let matched = lex.slice();
            let token_start_byte = self.byte_pos + span.start;
            let token_lineno = self.lineno;
            let token_lexpos = self.char_pos;

            match result {
                // 1. Quebra de linha
                Ok(TokenKind::Newline) => {
                    self.byte_pos += span.end;
                    self.char_pos += 1;
                    self.lineno += 1;
                    self.line_start_byte = self.byte_pos;
                    continue;
                }

                // 2. Espaços em branco (espaço, tabulação, \r)
                Ok(TokenKind::Whitespace) => {
                    self.byte_pos += span.end;
                    self.char_pos += matched.len();
                    continue;
                }

                // 3. Comentários de bloco: { ... }
                Ok(TokenKind::CommentStart) => {
                    let after_open = &remaining[span.end..];
                    if let Some(close_offset) = after_open.find('}') {
                        let comment_body = &after_open[..close_offset];
                        for (idx, b) in comment_body.bytes().enumerate() {
                            if b == b'\n' {
                                self.lineno += 1;
                                self.line_start_byte = self.byte_pos + span.end + idx + 1;
                            }
                        }
                        self.byte_pos += span.end + close_offset + 1;
                        self.char_pos += 1 + comment_body.chars().count() + 1;
                        continue;
                    } else {
                        self.report_error(
                            "ERR-LEX-UNC-COMMENT",
                            token_lineno,
                            token_start_byte,
                            None,
                        );
                        self.byte_pos = total_len;
                        return None;
                    }
                }

                // 4. Sinais '+' e '-' (podem iniciar Notação Científica com sinal ou ser operador)
                Ok(kind @ (TokenKind::Mais | TokenKind::Menos)) => {
                    if let Some(sci_len) = match_scientific_notation_str(remaining) {
                        let lexeme = &remaining[..sci_len];
                        self.byte_pos += sci_len;
                        self.char_pos += sci_len;
                        return Some(Token::new(
                            "NUM_NOTACAO_CIENTIFICA",
                            lexeme,
                            token_lineno,
                            token_lexpos,
                        ));
                    }
                    let tok_type = kind.as_str().unwrap();
                    self.byte_pos += span.end;
                    self.char_pos += 1;
                    return Some(Token::new(tok_type, matched, token_lineno, token_lexpos));
                }

                // 5. Caractere inválido detectado pelo fallback ou erro do Logos
                Ok(TokenKind::InvalidChar) | Err(()) => {
                    let inv_str = if !matched.is_empty() {
                        matched
                    } else {
                        let ch = remaining.chars().next().unwrap();
                        &remaining[..ch.len_utf8()]
                    };
                    self.report_error(
                        "ERR-LEX-INV-CHAR",
                        token_lineno,
                        token_start_byte,
                        Some(inv_str),
                    );
                    self.byte_pos += inv_str.len();
                    self.char_pos += inv_str.chars().count();
                    continue;
                }

                // 6. Tokens válidos da linguagem TPP
                Ok(kind) => {
                    if let Some(tok_type) = kind.as_str() {
                        self.byte_pos += span.end;
                        self.char_pos += matched.chars().count();
                        return Some(Token::new(tok_type, matched, token_lineno, token_lexpos));
                    }
                }
            }
        }

        None
    }
}

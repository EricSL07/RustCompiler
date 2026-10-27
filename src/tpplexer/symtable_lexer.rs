use std::collections::HashMap;

use crate::tpplexer::base::{
    get_column, is_tpp_alphanum_under, is_tpp_letter, match_scientific_notation, normalize_source,
    BaseLexer, Token,
};
use crate::tpplexer::log_error_lexer::LogErrorLexer;
use crate::tpplexer::mandfa_lexer::ManualDFALexer;
use crate::tpplexer::tokens::{reserved_words, OPERATORS_AND_DELIMITERS};

/// Entrada da Tabela de Símbolos contendo metadados do símbolo nativo ou identificador do usuário.
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub lexeme: String,
    pub token_type: String,
    pub first_line: usize,
    pub first_column: usize,
    pub is_builtin: bool,
    pub occurrences: Vec<(usize, usize)>,
}

impl SymbolEntry {
    pub fn new(
        lexeme: impl Into<String>,
        token_type: impl Into<String>,
        lineno: usize,
        column: usize,
        is_builtin: bool,
    ) -> Self {
        let occurrences = if is_builtin {
            Vec::new()
        } else {
            vec![(lineno, column)]
        };
        Self {
            lexeme: lexeme.into(),
            token_type: token_type.into(),
            first_line: lineno,
            first_column: column,
            is_builtin,
            occurrences,
        }
    }

    pub fn add_occurrence(&mut self, lineno: usize, column: usize) {
        self.occurrences.push((lineno, column));
    }
}

/// Tabela de Símbolos pré-populada com palavras reservadas, operadores e delimitadores da linguagem TPP.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    table: HashMap<String, SymbolEntry>,
}

impl SymbolTable {
    pub fn new_with_builtins() -> Self {
        let mut table = HashMap::new();
        for (lexeme, tok_type) in reserved_words() {
            table.insert(
                lexeme.to_string(),
                SymbolEntry::new(lexeme, tok_type, 0, 0, true),
            );
        }
        for &(lexeme, tok_type) in OPERATORS_AND_DELIMITERS {
            table.insert(
                lexeme.to_string(),
                SymbolEntry::new(lexeme, tok_type, 0, 0, true),
            );
        }
        Self { table }
    }

    pub fn new_empty() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn lookup(&self, lexeme: &str) -> Option<&SymbolEntry> {
        self.table.get(lexeme)
    }

    pub fn lookup_mut(&mut self, lexeme: &str) -> Option<&mut SymbolEntry> {
        self.table.get_mut(lexeme)
    }

    pub fn install_identifier(&mut self, lexeme: &str, lineno: usize, column: usize) {
        if let Some(entry) = self.table.get_mut(lexeme) {
            if !entry.is_builtin {
                entry.add_occurrence(lineno, column);
            }
            return;
        }
        self.table.insert(
            lexeme.to_string(),
            SymbolEntry::new(lexeme, "ID", lineno, column, false),
        );
    }

    pub fn reset_user_symbols(&mut self) {
        self.table.retain(|_, v| v.is_builtin);
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }

    pub fn all_symbols(&self) -> &HashMap<String, SymbolEntry> {
        &self.table
    }
}

/// Analisador Léxico Puro Orientado a Tabela de Símbolos (`symtable`).
/// Consulta a Tabela de Símbolos tanto para palavras reservadas/identificadores quanto para operadores.
#[derive(Debug, Clone)]
pub struct SymbolTableLexer {
    pub check_key: bool,
    pub log_err: LogErrorLexer,
    pub chars: Vec<char>,
    pub pos: usize,
    pub lineno: usize,
    pub symbol_table: SymbolTable,
}

impl SymbolTableLexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            check_key,
            log_err: LogErrorLexer::new(),
            chars: Vec::new(),
            pos: 0,
            lineno: 1,
            symbol_table: SymbolTable::new_with_builtins(),
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

impl BaseLexer for SymbolTableLexer {
    fn input(&mut self, data: &str) {
        self.chars = normalize_source(data);
        self.pos = 0;
        self.lineno = 1;
        self.symbol_table.reset_user_symbols();
    }

    fn token(&mut self) -> Option<Token> {
        let len = self.chars.len();

        while self.pos < len {
            let c = self.chars[self.pos];

            if c == '\n' {
                self.lineno += 1;
                self.pos += 1;
                continue;
            }
            if c == ' ' || c == '\t' || c == '\r' {
                self.pos += 1;
                continue;
            }

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

            // Notação científica iniciada por sinal
            if (c == '+' || c == '-') && matches!(self.peek(1), Some('1'..='9')) {
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

            // Termos alfanuméricos resolvidos via Tabela de Símbolos
            if is_tpp_letter(c) {
                self.pos += 1;
                while self.pos < len && is_tpp_alphanum_under(self.chars[self.pos]) {
                    self.pos += 1;
                }
                let lexeme = self.slice_to_string(start_pos, self.pos);
                let col = get_column(&self.chars, start_pos);

                if let Some(entry) = self.symbol_table.lookup_mut(&lexeme) {
                    let tok_type = entry.token_type.clone();
                    if !entry.is_builtin {
                        entry.add_occurrence(start_lineno, col);
                    }
                    return Some(Token::new(tok_type, lexeme, start_lineno, start_pos));
                } else {
                    self.symbol_table
                        .install_identifier(&lexeme, start_lineno, col);
                    return Some(Token::new("ID", lexeme, start_lineno, start_pos));
                }
            }

            // Numerais
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

            // Operadores compostos de 2 caracteres consultados na Tabela de Símbolos
            if self.pos + 2 <= len {
                let two_chars = self.slice_to_string(self.pos, self.pos + 2);
                if let Some(entry) = self.symbol_table.lookup(&two_chars) {
                    if entry.is_builtin {
                        let tok_type = entry.token_type.clone();
                        self.pos += 2;
                        return Some(Token::new(tok_type, two_chars, start_lineno, start_pos));
                    }
                }
            }

            // Operadores e delimitadores de 1 caractere consultados na Tabela de Símbolos
            let one_char = c.to_string();
            if let Some(entry) = self.symbol_table.lookup(&one_char) {
                if entry.is_builtin {
                    let tok_type = entry.token_type.clone();
                    self.pos += 1;
                    return Some(Token::new(tok_type, one_char, start_lineno, start_pos));
                }
            }

            // Caractere inválido
            self.report_error("ERR-LEX-INV-CHAR", start_lineno, start_pos, Some(&one_char));
            self.pos += 1;
        }

        None
    }
}

/// Analisador Léxico Híbrido que compõe o `ManualDFALexer` com Tabela de Símbolos (`symtableman`).
#[derive(Debug, Clone)]
pub struct SymbolTableManLexer {
    pub inner: ManualDFALexer,
    pub symbol_table: SymbolTable,
}

impl SymbolTableManLexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            inner: ManualDFALexer::new(check_key),
            symbol_table: SymbolTable::new_empty(),
        }
    }
}

impl BaseLexer for SymbolTableManLexer {
    fn input(&mut self, data: &str) {
        self.inner.input(data);
        self.symbol_table.clear();
    }

    fn token(&mut self) -> Option<Token> {
        let tok = self.inner.token()?;
        if tok.token_type == "ID" {
            let col = get_column(&self.inner.chars, tok.lexpos);
            self.symbol_table
                .install_identifier(&tok.value, tok.lineno, col);
        }
        Some(tok)
    }
}

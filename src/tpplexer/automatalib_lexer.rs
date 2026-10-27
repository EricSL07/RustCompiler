use std::collections::{HashMap, HashSet};

use crate::tpplexer::base::{get_column, normalize_source, BaseLexer, Token};
use crate::tpplexer::log_error_lexer::LogErrorLexer;
use crate::tpplexer::mandfa_lexer::ManualDFALexer;
use crate::tpplexer::tokens::resolve_keyword_or_id;

/// Representação formal da 5-tupla de um Autômato Finito Determinístico:
/// `M = (Q, Sigma, delta, q0, F)`
#[derive(Debug, Clone)]
pub struct FormalDFA {
    pub states: HashSet<String>,
    pub input_symbols: HashSet<char>,
    pub transitions: HashMap<String, HashMap<char, String>>,
    pub initial_state: String,
    pub final_states: HashSet<String>,
}

impl FormalDFA {
    /// Simula o AFD determinístico a partir de `slice` e retorna o comprimento
    /// do maior prefixo aceito (Maximal Munch / Longest Match Rule), ou `0` se rejeitado.
    pub fn match_longest(&self, slice: &[char]) -> usize {
        let mut curr_state = self.initial_state.as_str();
        let mut longest_match_len = 0;

        for (idx, &ch) in slice.iter().enumerate() {
            let Some(state_trans) = self.transitions.get(curr_state) else {
                break;
            };
            let Some(next_state) = state_trans.get(&ch) else {
                break;
            };
            curr_state = next_state.as_str();
            if self.final_states.contains(curr_state) {
                longest_match_len = idx + 1;
            }
        }

        longest_match_len
    }

    /// Verifica se uma cadeia completa é aceita pelo AFD formal.
    pub fn accepts_input(&self, input: &str) -> bool {
        let chars: Vec<char> = input.chars().collect();
        let mut curr_state = self.initial_state.as_str();
        for ch in chars {
            let Some(state_trans) = self.transitions.get(curr_state) else {
                return false;
            };
            let Some(next_state) = state_trans.get(&ch) else {
                return false;
            };
            curr_state = next_state.as_str();
        }
        self.final_states.contains(curr_state)
    }
}

fn tpp_letters() -> Vec<char> {
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZáÁãÃàÀéÉíÍóÓõÕ"
        .chars()
        .collect()
}

fn tpp_digits() -> Vec<char> {
    "0123456789".chars().collect()
}

pub fn build_dfa_id() -> FormalDFA {
    let letters = tpp_letters();
    let digits = tpp_digits();
    let mut alphanum = letters.clone();
    alphanum.extend(digits.iter().copied());
    alphanum.push('_');

    let mut q0_trans = HashMap::new();
    for &c in &letters {
        q0_trans.insert(c, "q1".to_string());
    }

    let mut q1_trans = HashMap::new();
    for &c in &alphanum {
        q1_trans.insert(c, "q1".to_string());
    }

    let mut transitions = HashMap::new();
    transitions.insert("q0".to_string(), q0_trans);
    transitions.insert("q1".to_string(), q1_trans);

    FormalDFA {
        states: HashSet::from(["q0".to_string(), "q1".to_string()]),
        input_symbols: alphanum.into_iter().collect(),
        transitions,
        initial_state: "q0".to_string(),
        final_states: HashSet::from(["q1".to_string()]),
    }
}

pub fn build_dfa_inteiro() -> FormalDFA {
    let digits = tpp_digits();
    let mut trans = HashMap::new();
    for &c in &digits {
        trans.insert(c, "q1".to_string());
    }

    let mut transitions = HashMap::new();
    transitions.insert("q0".to_string(), trans.clone());
    transitions.insert("q1".to_string(), trans);

    FormalDFA {
        states: HashSet::from(["q0".to_string(), "q1".to_string()]),
        input_symbols: digits.into_iter().collect(),
        transitions,
        initial_state: "q0".to_string(),
        final_states: HashSet::from(["q1".to_string()]),
    }
}

pub fn build_dfa_notacao_cientifica() -> FormalDFA {
    let digits = tpp_digits();
    let nonzero: Vec<char> = "123456789".chars().collect();
    let mut symbols: HashSet<char> = digits.iter().copied().collect();
    symbols.extend(['+', '-', 'e', 'E', '.']);

    let state_names = [
        "q0",
        "q_sign",
        "q_first",
        "q_dot",
        "q_frac",
        "q_exp",
        "q_expsign",
        "q_expdigits",
    ];
    let mut transitions: HashMap<String, HashMap<char, String>> = state_names
        .iter()
        .map(|s| (s.to_string(), HashMap::new()))
        .collect();

    for c in ['+', '-'] {
        transitions
            .get_mut("q0")
            .unwrap()
            .insert(c, "q_sign".to_string());
    }
    for &c in &nonzero {
        transitions
            .get_mut("q0")
            .unwrap()
            .insert(c, "q_first".to_string());
        transitions
            .get_mut("q_sign")
            .unwrap()
            .insert(c, "q_first".to_string());
    }
    transitions
        .get_mut("q_first")
        .unwrap()
        .insert('.', "q_dot".to_string());

    for &c in &digits {
        transitions
            .get_mut("q_dot")
            .unwrap()
            .insert(c, "q_frac".to_string());
        transitions
            .get_mut("q_frac")
            .unwrap()
            .insert(c, "q_frac".to_string());
        transitions
            .get_mut("q_exp")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
        transitions
            .get_mut("q_expsign")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
        transitions
            .get_mut("q_expdigits")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
    }
    for c in ['e', 'E'] {
        transitions
            .get_mut("q_frac")
            .unwrap()
            .insert(c, "q_exp".to_string());
    }
    for c in ['+', '-'] {
        transitions
            .get_mut("q_exp")
            .unwrap()
            .insert(c, "q_expsign".to_string());
    }

    FormalDFA {
        states: state_names.iter().map(|s| s.to_string()).collect(),
        input_symbols: symbols,
        transitions,
        initial_state: "q0".to_string(),
        final_states: HashSet::from(["q_expdigits".to_string()]),
    }
}

pub fn build_dfa_flutuante() -> FormalDFA {
    let digits = tpp_digits();
    let mut symbols: HashSet<char> = digits.iter().copied().collect();
    symbols.extend(['.', 'e', 'E', '+', '-']);

    let state_names = [
        "q0",
        "q_dot_start",
        "q_int",
        "q_dot",
        "q_frac",
        "q_exp",
        "q_expsign",
        "q_expdigits",
    ];
    let mut transitions: HashMap<String, HashMap<char, String>> = state_names
        .iter()
        .map(|s| (s.to_string(), HashMap::new()))
        .collect();

    for &c in &digits {
        transitions
            .get_mut("q0")
            .unwrap()
            .insert(c, "q_int".to_string());
        transitions
            .get_mut("q_dot_start")
            .unwrap()
            .insert(c, "q_frac".to_string());
        transitions
            .get_mut("q_int")
            .unwrap()
            .insert(c, "q_int".to_string());
        transitions
            .get_mut("q_dot")
            .unwrap()
            .insert(c, "q_frac".to_string());
        transitions
            .get_mut("q_frac")
            .unwrap()
            .insert(c, "q_frac".to_string());
        transitions
            .get_mut("q_exp")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
        transitions
            .get_mut("q_expsign")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
        transitions
            .get_mut("q_expdigits")
            .unwrap()
            .insert(c, "q_expdigits".to_string());
    }

    transitions
        .get_mut("q0")
        .unwrap()
        .insert('.', "q_dot_start".to_string());
    transitions
        .get_mut("q_int")
        .unwrap()
        .insert('.', "q_dot".to_string());

    for c in ['e', 'E'] {
        transitions
            .get_mut("q_int")
            .unwrap()
            .insert(c, "q_exp".to_string());
        transitions
            .get_mut("q_dot")
            .unwrap()
            .insert(c, "q_exp".to_string());
        transitions
            .get_mut("q_frac")
            .unwrap()
            .insert(c, "q_exp".to_string());
    }
    for c in ['+', '-'] {
        transitions
            .get_mut("q_exp")
            .unwrap()
            .insert(c, "q_expsign".to_string());
    }

    FormalDFA {
        states: state_names.iter().map(|s| s.to_string()).collect(),
        input_symbols: symbols,
        transitions,
        initial_state: "q0".to_string(),
        final_states: HashSet::from([
            "q_dot".to_string(),
            "q_frac".to_string(),
            "q_expdigits".to_string(),
        ]),
    }
}

pub fn build_dfa_literal(literal: &str) -> FormalDFA {
    let chars: Vec<char> = literal.chars().collect();
    let n = chars.len();
    let mut states = HashSet::new();
    let mut symbols = HashSet::new();
    let mut transitions = HashMap::new();

    for i in 0..=n {
        let s = format!("q{}", i);
        states.insert(s.clone());
        transitions.insert(s, HashMap::new());
    }

    for (i, &ch) in chars.iter().enumerate() {
        symbols.insert(ch);
        let curr = format!("q{}", i);
        let next = format!("q{}", i + 1);
        transitions.get_mut(&curr).unwrap().insert(ch, next);
    }

    FormalDFA {
        states,
        input_symbols: symbols,
        transitions,
        initial_state: "q0".to_string(),
        final_states: HashSet::from([format!("q{}", n)]),
    }
}

/// Analisador Léxico Puro baseado em AFDs formais (`M = (Q, Sigma, delta, q0, F)`)
/// utilizando o algoritmo Maximal Munch (Prefixo Mais Longo) (`automatalib`).
#[derive(Debug, Clone)]
pub struct AutomataLibLexer {
    pub check_key: bool,
    pub log_err: LogErrorLexer,
    pub chars: Vec<char>,
    pub pos: usize,
    pub lineno: usize,
    pub rules: Vec<(&'static str, FormalDFA)>,
}

impl AutomataLibLexer {
    pub fn new(check_key: bool) -> Self {
        let rules = vec![
            ("NUM_NOTACAO_CIENTIFICA", build_dfa_notacao_cientifica()),
            ("NUM_PONTO_FLUTUANTE", build_dfa_flutuante()),
            ("NUM_INTEIRO", build_dfa_inteiro()),
            ("ID", build_dfa_id()),
            ("ATRIBUICAO", build_dfa_literal(":=")),
            ("MENOR_IGUAL", build_dfa_literal("<=")),
            ("MAIOR_IGUAL", build_dfa_literal(">=")),
            ("DIFERENTE", build_dfa_literal("<>")),
            ("E", build_dfa_literal("&&")),
            ("OU", build_dfa_literal("||")),
            ("MENOR", build_dfa_literal("<")),
            ("MAIOR", build_dfa_literal(">")),
            ("IGUAL", build_dfa_literal("=")),
            ("NAO", build_dfa_literal("!")),
            ("MAIS", build_dfa_literal("+")),
            ("MENOS", build_dfa_literal("-")),
            ("VEZES", build_dfa_literal("*")),
            ("DIVIDE", build_dfa_literal("/")),
            ("ABRE_PARENTESE", build_dfa_literal("(")),
            ("FECHA_PARENTESE", build_dfa_literal(")")),
            ("ABRE_COLCHETE", build_dfa_literal("[")),
            ("FECHA_COLCHETE", build_dfa_literal("]")),
            ("VIRGULA", build_dfa_literal(",")),
            ("DOIS_PONTOS", build_dfa_literal(":")),
        ];

        Self {
            check_key,
            log_err: LogErrorLexer::new(),
            chars: Vec::new(),
            pos: 0,
            lineno: 1,
            rules,
        }
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

impl BaseLexer for AutomataLibLexer {
    fn input(&mut self, data: &str) {
        self.chars = normalize_source(data);
        self.pos = 0;
        self.lineno = 1;
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
                let start_lineno = self.lineno;
                let start_pos = self.pos;
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

            let start_lineno = self.lineno;
            let start_pos = self.pos;

            let mut best_type: Option<&'static str> = None;
            let mut best_len: usize = 0;

            let remaining = &self.chars[self.pos..];
            for &(tok_type, ref dfa) in &self.rules {
                let mlen = dfa.match_longest(remaining);
                if mlen > best_len {
                    best_len = mlen;
                    best_type = Some(tok_type);
                }
            }

            if let Some(btype) = best_type {
                if best_len > 0 {
                    let lexeme = self.slice_to_string(start_pos, start_pos + best_len);
                    self.pos += best_len;
                    let final_type = if btype == "ID" {
                        resolve_keyword_or_id(&lexeme)
                    } else {
                        btype
                    };
                    return Some(Token::new(final_type, lexeme, start_lineno, start_pos));
                }
            }

            let s = c.to_string();
            self.report_error("ERR-LEX-INV-CHAR", start_lineno, start_pos, Some(&s));
            self.pos += 1;
        }

        None
    }
}

/// Analisador Léxico que emprega AFDs formais acoplado ao `ManualDFALexer` (`automatalibman`).
#[derive(Debug, Clone)]
pub struct AutomataLibManLexer {
    pub inner: ManualDFALexer,
    pub dfa_id: FormalDFA,
    pub dfa_notacao_cientifica: FormalDFA,
    pub dfa_inteiro: FormalDFA,
}

impl AutomataLibManLexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            inner: ManualDFALexer::new(check_key),
            dfa_id: build_dfa_id(),
            dfa_notacao_cientifica: build_dfa_notacao_cientifica(),
            dfa_inteiro: build_dfa_inteiro(),
        }
    }

    pub fn is_valid_identifier(&self, lexeme: &str) -> bool {
        self.dfa_id.accepts_input(lexeme)
    }

    pub fn is_scientific_notation(&self, lexeme: &str) -> bool {
        self.dfa_notacao_cientifica.accepts_input(lexeme)
    }

    pub fn is_integer(&self, lexeme: &str) -> bool {
        self.dfa_inteiro.accepts_input(lexeme)
    }
}

impl BaseLexer for AutomataLibManLexer {
    fn input(&mut self, data: &str) {
        self.inner.input(data);
    }

    fn token(&mut self) -> Option<Token> {
        self.inner.token()
    }
}

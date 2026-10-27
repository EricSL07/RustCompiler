use std::collections::HashMap;

/// Retorna o mapeamento das palavras reservadas da linguagem TPP para seus respectivos tipos de Token.
pub fn reserved_words() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("se", "SE"),
        ("então", "ENTAO"),
        ("senão", "SENAO"),
        ("fim", "FIM"),
        ("repita", "REPITA"),
        ("flutuante", "FLUTUANTE"),
        ("retorna", "RETORNA"),
        ("até", "ATE"),
        ("leia", "LEIA"),
        ("escreva", "ESCREVA"),
        ("inteiro", "INTEIRO"),
    ])
}

/// Resolve se um lexema é uma palavra reservada da linguagem TPP ou um identificador (`ID`).
pub fn resolve_keyword_or_id(lexeme: &str) -> &'static str {
    match lexeme {
        "se" => "SE",
        "então" => "ENTAO",
        "senão" => "SENAO",
        "fim" => "FIM",
        "repita" => "REPITA",
        "flutuante" => "FLUTUANTE",
        "retorna" => "RETORNA",
        "até" => "ATE",
        "leia" => "LEIA",
        "escreva" => "ESCREVA",
        "inteiro" => "INTEIRO",
        _ => "ID",
    }
}

/// Lista de operadores e delimitadores estáticos da linguagem TPP.
pub const OPERATORS_AND_DELIMITERS: &[(&str, &str)] = &[
    (":=", "ATRIBUICAO"),
    ("<=", "MENOR_IGUAL"),
    (">=", "MAIOR_IGUAL"),
    ("<>", "DIFERENTE"),
    ("&&", "E"),
    ("||", "OU"),
    ("!", "NAO"),
    ("+", "MAIS"),
    ("-", "MENOS"),
    ("*", "VEZES"),
    ("/", "DIVIDE"),
    ("<", "MENOR"),
    (">", "MAIOR"),
    ("=", "IGUAL"),
    ("(", "ABRE_PARENTESE"),
    (")", "FECHA_PARENTESE"),
    ("[", "ABRE_COLCHETE"),
    ("]", "FECHA_COLCHETE"),
    (",", "VIRGULA"),
    (":", "DOIS_PONTOS"),
];

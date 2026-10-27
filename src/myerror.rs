use std::collections::HashMap;
use std::fs;
use std::path::Path;

const DEFAULT_GLOBAL_ERRORS: &str = include_str!("GlobalErrorMessages.properties");
const DEFAULT_LEXER_ERRORS: &str = include_str!("tpplexer/LexerErrorMessages.properties");
const DEFAULT_PARSER_ERRORS: &str = include_str!("tppparser/ParserMessages.properties");

/// Gerenciador padronizado de erros para todas as fases do compilador TPP.
/// Carrega mensagens a partir de arquivos `.properties` (estilo INI) e formata
/// tanto mensagens completas (`Erro[linha][coluna]: ...`) quanto chaves puras (`-k`).
#[derive(Debug, Clone)]
pub struct MyError {
    error_type: String,
    sections: HashMap<String, HashMap<String, String>>,
}

impl MyError {
    pub fn new(error_type: &str) -> Self {
        let mut instance = Self {
            error_type: error_type.to_string(),
            sections: HashMap::new(),
        };

        // Carrega os defaults embutidos em tempo de compilação
        instance.parse_ini(DEFAULT_GLOBAL_ERRORS);
        instance.parse_ini(DEFAULT_LEXER_ERRORS);
        instance.parse_ini(DEFAULT_PARSER_ERRORS);

        // Tenta sobrepor com arquivos do sistema de arquivos, se presentes
        let error_files = [
            "GlobalErrorMessages.properties",
            "src/GlobalErrorMessages.properties",
            "tpplexer/LexerErrorMessages.properties",
            "src/tpplexer/LexerErrorMessages.properties",
            "tppparser/ParserErrorMessages.properties",
            "src/tppparser/ParserMessages.properties",
            "tppsema/SemaErrorMessages.properties",
            "src/tppsema/SemaErrorMessages.properties",
            "tppcodegen/CodeGenErrorMessages.properties",
            "src/tppcodegen/CodeGenErrorMessages.properties",
        ];

        for file_path in error_files {
            if Path::new(file_path).exists() {
                if let Ok(content) = fs::read_to_string(file_path) {
                    instance.parse_ini(&content);
                }
            }
        }

        instance
    }

    fn parse_ini(&mut self, content: &str) {
        let mut current_section = String::from("DEFAULT");
        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();
                self.sections.entry(current_section.clone()).or_default();
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim().to_uppercase();
                let val = v.trim().to_string();
                self.sections
                    .entry(current_section.clone())
                    .or_default()
                    .insert(key, val);
            }
        }
    }

    /// Gera a mensagem de erro ou retorna apenas a chave caso `koption` (`-k`) seja `true`.
    pub fn new_error(
        &self,
        koption: bool,
        key: &str,
        line: Option<usize>,
        column: Option<usize>,
        args: &[&str],
        data: &[(&str, &str)],
    ) -> String {
        if koption {
            return key.to_string();
        }

        let mut message = String::new();

        if let (Some(l), Some(c)) = (line, column) {
            if l > 0 && c > 0 {
                message.push_str(&format!("Erro[{}][{}]: ", l, c));
            }
        }

        if !key.is_empty() {
            let template = self
                .sections
                .get(&self.error_type)
                .and_then(|sec| sec.get(&key.to_uppercase()))
                .cloned()
                .unwrap_or_else(|| key.to_string());

            let mut msg = template.clone();
            for arg in args {
                if let Some(pos) = msg.find("{}") {
                    msg.replace_range(pos..pos + 2, arg);
                }
            }
            for (k, v) in data {
                let placeholder = format!("{{{}}}", k);
                if msg.contains(&placeholder) {
                    msg = msg.replace(&placeholder, v);
                }
            }

            message.push_str(&msg);

            if !data.is_empty() {
                let mut unused = Vec::new();
                for (k, v) in data {
                    let placeholder = format!("{{{}}}", k);
                    if !template.contains(&placeholder) {
                        unused.push(format!("{}: {}", k, v));
                    }
                }
                if !unused.is_empty() {
                    message.push(' ');
                    message.push_str(&unused.join(", "));
                }
            }
        }

        message
    }
}

use crate::myerror::MyError;

/// Helper para tratamento e emissão de erros léxicos padronizados.
#[derive(Debug, Clone)]
pub struct LogErrorLexer {
    pub le: MyError,
}

impl LogErrorLexer {
    pub fn new() -> Self {
        Self {
            le: MyError::new("LexerErrors"),
        }
    }

    pub fn format_error(
        &self,
        check_key: bool,
        key: &str,
        lineno: usize,
        col: usize,
        valor: Option<&str>,
    ) -> String {
        if let Some(val) = valor {
            self.le
                .new_error(check_key, key, Some(lineno), Some(col), &[], &[("valor", val)])
        } else {
            self.le
                .new_error(check_key, key, Some(lineno), Some(col), &[], &[])
        }
    }
}

impl Default for LogErrorLexer {
    fn default() -> Self {
        Self::new()
    }
}

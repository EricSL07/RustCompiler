use crate::tpplexer::base::{BaseLexer, Token};
use crate::tpplexer::mandfa_lexer::ManualDFALexer;

/// Implementação compatível com a estratégia padrão `ply` (regras de precedência de expressões regulares da TPP).
#[derive(Debug, Clone)]
pub struct PLYLexer {
    inner: ManualDFALexer,
}

impl PLYLexer {
    pub fn new(check_key: bool) -> Self {
        Self {
            inner: ManualDFALexer::new(check_key),
        }
    }
}

impl BaseLexer for PLYLexer {
    fn input(&mut self, data: &str) {
        self.inner.input(data);
    }

    fn token(&mut self) -> Option<Token> {
        self.inner.token()
    }
}

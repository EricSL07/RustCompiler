use crate::myerror::MyError;
use crate::tpplexer::base::Token;
use crate::tpplexer::tokens::TokenKind;
use crate::tppparser::ast::AstNode;

/// Estrutura que representa o parser
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    check_key: bool,
    error_handler: MyError,
}

// Implementação do parser
impl Parser {
    // Cria uma nova instância do parser com os tokens fornecidos
    pub fn new(tokens: Vec<Token>, check_key: bool) -> Self {
        Self {
            tokens,
            pos: 0,
            check_key,
            error_handler: MyError::new("ParserErrors"),
        }
    }

    /// Analisa a entrada e retorna a árvore de sintaxe abstrata (AST) resultante.
    pub fn parse(&mut self) -> Result<AstNode, String> {
        let ast = self.parse_expression_additive()?;
        if let Some(tok) = self.current_token() {
            Err(self.error_handler.new_error(
                self.check_key,
                "ERR-SYN-EOF-INESPERADO",
                Some(tok.lineno),
                Some(tok.lexpos),
                &[],
                &[],
            ))
        } else {
            Ok(ast)
        }
    }

    // Pega o próximo token sem avançar a posição
    #[allow(dead_code)]
    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    // Verifica se o token atual corresponde ao token esperado
    fn check(&self, kind: TokenKind) -> bool {
        matches!(
            (self.current_token(), kind.as_str()),
            (Some(tok), Some(expected)) if tok.token_type == expected
        )
    }

    // Pega o token atual
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn format_error(&self, error_key: &str) -> String {
        let (line, col) = match self.current_token() {
            Some(tok) => (Some(tok.lineno), Some(tok.lexpos)),
            None => (None, None),
        };
        self.error_handler
            .new_error(self.check_key, error_key, line, col, &[], &[])
    }

    // Verifica se o token atual corresponde ao token esperado e avança a posição
    fn expect(&mut self, expected: TokenKind, error_key: &str) -> Result<Token, String> {
        if let Some(tok) = self.match_token(expected) {
            Ok(tok)
        } else {
            Err(self.format_error(error_key))
        }
    }

    // Consome o token se for do tipo esperado, caso contrário retorna None
    fn match_token(&mut self, expected: TokenKind) -> Option<Token> {
        if self.check(expected) {
            let token = self.current_token().cloned();
            self.advance();
            token
        } else {
            None
        }
    }

    /// Analisa expressões primárias (números, parênteses)
    fn parse_expressao_primaria(&mut self) -> Result<AstNode, String> {
        if let Some(token) = self.current_token().cloned() {
            match token.token_type.as_str() {
                "NUM_INTEIRO" | "NUM_PONTO_FLUTUANTE" => {
                    let node = AstNode::new(format!("Literal: {:?}", token));
                    self.advance();
                    Ok(node)
                }
                "ABRE_PARENTESE" => {
                    self.advance(); // Consome '('
                    let expr = self.parse_expression_additive()?;
                    self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;
                    Ok(expr)
                }
                _ => Err(self.format_error("ERR-SYN-IRRECUPERAVEL")),
            }
        } else {
            Err(self.format_error("ERR-SYN-EOF-INESPERADO"))
        }
    }

    fn parse_expressao_unaria(&mut self) -> Result<AstNode, String> {
        if self.check(TokenKind::Mais) || self.check(TokenKind::Menos) {
            let operator = self.current_token().unwrap().clone();
            self.advance();
            let operand = self.parse_expressao_unaria()?;
            Ok(AstNode::with_children(
                "ExpressionUnary",
                vec![
                    AstNode::new(format!("Operator: {:?}", operator)),
                    operand,
                ],
            ))
        } else {
            self.parse_expressao_primaria()
        }
    }

    // Função para analisar expressões multiplicativas (multiplicação e divisão)
    fn parse_expressao_multiplicativa(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_expressao_unaria()?;

        while self.check(TokenKind::Vezes) || self.check(TokenKind::Divide) {
            let operator = self.current_token().unwrap().clone();
            self.advance();

            let right = self.parse_expressao_unaria()?;
            left = AstNode::with_children(
                "ExpressionMultiplicative",
                vec![
                    AstNode::new(format!("Operator: {:?}", operator)),
                    left,
                    right,
                ],
            )
        }
        Ok(left)
    }

    // Função para analisar expressões aditivas (soma e subtração)
    fn parse_expression_additive(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_expressao_multiplicativa()?;

        while self.check(TokenKind::Mais) || self.check(TokenKind::Menos) {
            let operator = self.current_token().unwrap().clone();
            self.advance();

            let right = self.parse_expressao_multiplicativa()?;
            left = AstNode::with_children(
                "ExpressionAdditive",
                vec![
                    AstNode::new(format!("Operator: {:?}", operator)),
                    left,
                    right,
                ],
            )
        }
        Ok(left)
    }

    // Avança para o próximo token
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}

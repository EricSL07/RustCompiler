use crate::myerror::MyError;
use crate::tpplexer::base::Token;
use crate::tpplexer::tokens::TokenKind;
use crate::tppparser::ast::AstNode;

/// Analisador Sintático (Parser Descendente Recursivo) para a linguagem TPP.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    check_key: bool,
    error_handler: MyError,
}

impl Parser {
    /// Cria uma nova instância do parser com os tokens fornecidos.
    pub fn new(tokens: Vec<Token>, check_key: bool) -> Self {
        Self {
            tokens,
            pos: 0,
            check_key,
            error_handler: MyError::new("ParserErrors"),
        }
    }

    /// Analisa um programa completo (`programa ::= lista_declaracoes`) e retorna a AST.
    pub fn parse(&mut self) -> Result<AstNode, String> {
        let ast = self.parse_programa()?;
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

    /// Analisa uma única expressão isolada (útil para testes unitários de expressões).
    pub fn parse_expressao_completa(&mut self) -> Result<AstNode, String> {
        let ast = self.parse_expressao()?;
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

    // -------------------------------------------------------------------------
    // Helpers de navegação e erro
    // -------------------------------------------------------------------------

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
    }

    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn check(&self, kind: TokenKind) -> bool {
        matches!(
            (self.current_token(), kind.as_str()),
            (Some(tok), Some(expected)) if tok.token_type == expected
        )
    }

    fn check_at(&self, offset: usize, kind: TokenKind) -> bool {
        matches!(
            (self.peek(offset), kind.as_str()),
            (Some(tok), Some(expected)) if tok.token_type == expected
        )
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn match_token(&mut self, expected: TokenKind) -> Option<Token> {
        if self.check(expected) {
            let token = self.current_token().cloned();
            self.advance();
            token
        } else {
            None
        }
    }

    fn format_error(&self, error_key: &str) -> String {
        let (line, col) = match self.current_token() {
            Some(tok) => (Some(tok.lineno), Some(tok.lexpos)),
            None => (None, None),
        };
        self.error_handler
            .new_error(self.check_key, error_key, line, col, &[], &[])
    }

    fn expect(&mut self, expected: TokenKind, error_key: &str) -> Result<Token, String> {
        if let Some(tok) = self.match_token(expected) {
            Ok(tok)
        } else if self.current_token().is_none() {
            Err(self.format_error("ERR-SYN-EOF-INESPERADO"))
        } else {
            Err(self.format_error(error_key))
        }
    }

    fn is_tipo_at(&self, offset: usize) -> bool {
        self.check_at(offset, TokenKind::Inteiro) || self.check_at(offset, TokenKind::Flutuante)
    }

    // -------------------------------------------------------------------------
    // Programa e Declarações
    // -------------------------------------------------------------------------

    /// `programa ::= lista_declaracoes`
    fn parse_programa(&mut self) -> Result<AstNode, String> {
        let mut decls = Vec::new();
        if self.current_token().is_none() {
            return Err(self.format_error("ERR-SYN-EOF-INESPERADO"));
        }
        while self.current_token().is_some() {
            decls.push(self.parse_declaracao()?);
        }
        Ok(AstNode::with_children("Programa", decls))
    }

    /// `declaracao ::= declaracao_variaveis | inicializacao_variaveis | declaracao_funcao`
    fn parse_declaracao(&mut self) -> Result<AstNode, String> {
        if self.is_tipo_at(0) {
            if self.check_at(1, TokenKind::DoisPontos) {
                self.parse_declaracao_variaveis()
            } else {
                self.parse_declaracao_funcao()
            }
        } else if self.check(TokenKind::AbreParentese) {
            if self.is_tipo_at(1) {
                self.parse_declaracao_funcao()
            } else {
                self.parse_inicializacao_variaveis()
            }
        } else if self.check(TokenKind::Id) {
            if self.check_at(1, TokenKind::AbreParentese) {
                self.parse_declaracao_funcao()
            } else {
                self.parse_inicializacao_variaveis()
            }
        } else {
            Err(self.format_error("ERR-SYN-IRRECUPERAVEL"))
        }
    }

    /// `tipo ::= INTEIRO | FLUTUANTE`
    fn parse_tipo(&mut self) -> Result<AstNode, String> {
        if let Some(tok) = self
            .match_token(TokenKind::Inteiro)
            .or_else(|| self.match_token(TokenKind::Flutuante))
        {
            Ok(AstNode::new(format!("Tipo: {}", tok.token_type)))
        } else {
            Err(self.format_error("ERR-SYN-IRRECUPERAVEL"))
        }
    }

    /// `declaracao_variaveis ::= tipo DOIS_PONTOS lista_variaveis [ATRIBUICAO (atribuicao_encadeada | lista_expressoes)]`
    fn parse_declaracao_variaveis(&mut self) -> Result<AstNode, String> {
        let tipo = self.parse_tipo()?;
        self.expect(TokenKind::DoisPontos, "ERR-SYN-IRRECUPERAVEL")?;
        let vars = self.parse_lista_variaveis()?;
        let mut children = vec![tipo, vars];

        if self.match_token(TokenKind::Atribuicao).is_some() {
            let rhs = self.parse_rhs_atribuicao()?;
            children.push(rhs);
        }
        Ok(AstNode::with_position("DeclaracaoVariaveis", line, col, children))
    }

    /// `inicializacao_variaveis ::= lista_alvos ATRIBUICAO lista_expressoes | ABRE_PARENTESE lista_alvos FECHA_PARENTESE ATRIBUICAO lista_expressoes`
    fn parse_inicializacao_variaveis(&mut self) -> Result<AstNode, String> {
        let alvos = if self.match_token(TokenKind::AbreParentese).is_some() {
            let alvos = self.parse_lista_alvos()?;
            self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;
            alvos
        } else {
            self.parse_lista_alvos()?
        };

        self.expect(TokenKind::Atribuicao, "ERR-SYN-IRRECUPERAVEL")?;
        let exprs = self.parse_lista_expressoes()?;
        Ok(AstNode::with_children(
            "InicializacaoVariaveis",
            vec![alvos, exprs],
        ))
    }

    fn parse_lista_variaveis(&mut self) -> Result<AstNode, String> {
        let mut vars = vec![self.parse_var()?];
        while self.match_token(TokenKind::Virgula).is_some() {
            vars.push(self.parse_var()?);
        }
        Ok(AstNode::with_children("ListaVariaveis", vars))
    }

    fn parse_lista_alvos(&mut self) -> Result<AstNode, String> {
        let mut alvos = vec![self.parse_var()?];
        while self.match_token(TokenKind::Virgula).is_some() {
            alvos.push(self.parse_var()?);
        }
        Ok(AstNode::with_children("ListaAlvos", alvos))
    }

    /// `var ::= ID | ID indice`
    fn parse_var(&mut self) -> Result<AstNode, String> {
        let id_tok = self.expect(TokenKind::Id, "ERR-SYN-IRRECUPERAVEL")?;
        let mut children = vec![AstNode::new(format!("ID: {}", id_tok.value))];

        while self.match_token(TokenKind::AbreColchete).is_some() {
            let expr = self.parse_expressao()?;
            self.expect(TokenKind::FechaColchete, "ERR-SYN-INDICE")?;
            children.push(AstNode::with_children("Indice", vec![expr]));
        }

        Ok(AstNode::with_position("Var", id_tok.lineno, id_tok.lexpos, children))
    }

    // -------------------------------------------------------------------------
    // Funções
    // -------------------------------------------------------------------------

    /// `declaracao_funcao ::= tipo cabecalho | tipos_retorno cabecalho | cabecalho`
    fn parse_declaracao_funcao(&mut self) -> Result<AstNode, String> {
        let mut children = Vec::new();
        if self.is_tipo_at(0) {
            children.push(self.parse_tipo()?);
        } else if self.check(TokenKind::AbreParentese) {
            children.push(self.parse_tipos_retorno()?);
        }
        children.push(self.parse_cabecalho()?);
        Ok(AstNode::with_children("DeclaracaoFuncao", children))
    }

    /// `tipos_retorno ::= ABRE_PARENTESE lista_tipos FECHA_PARENTESE`
    fn parse_tipos_retorno(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::AbreParentese, "ERR-SYN-CABECALHO")?;
        let mut tipos = vec![self.parse_tipo()?];
        while self.match_token(TokenKind::Virgula).is_some() {
            tipos.push(self.parse_tipo()?);
        }
        self.expect(TokenKind::FechaParentese, "ERR-SYN-CABECALHO")?;
        Ok(AstNode::with_children("TiposRetorno", tipos))
    }

    /// `cabecalho ::= ID ABRE_PARENTESE lista_parametros FECHA_PARENTESE bloco FIM`
    fn parse_cabecalho(&mut self) -> Result<AstNode, String> {
        let id_tok = self.expect(TokenKind::Id, "ERR-SYN-CABECALHO")?;
        self.expect(TokenKind::AbreParentese, "ERR-SYN-CABECALHO")?;
        let params = self.parse_lista_parametros()?;
        self.expect(TokenKind::FechaParentese, "ERR-SYN-CABECALHO")?;
        let bloco = self.parse_bloco()?;
        self.expect(TokenKind::Fim, "ERR-SYN-CABECALHO")?;

        Ok(AstNode::with_children(
            "Cabecalho",
            vec![AstNode::new(format!("ID: {}", id_tok.value)), params, bloco],
        ))
    }

    /// `lista_parametros ::= (parametro (VIRGULA parametro)*)?`
    fn parse_lista_parametros(&mut self) -> Result<AstNode, String> {
        let mut params = Vec::new();
        if !self.check(TokenKind::FechaParentese) {
            params.push(self.parse_parametro()?);
            while self.match_token(TokenKind::Virgula).is_some() {
                params.push(self.parse_parametro()?);
            }
        }
        Ok(AstNode::with_children("ListaParametros", params))
    }

    /// `parametro ::= tipo DOIS_PONTOS ID (ABRE_COLCHETE FECHA_COLCHETE)*`
    fn parse_parametro(&mut self) -> Result<AstNode, String> {
        let tipo = self.parse_tipo().map_err(|_| self.format_error("ERR-SYN-PARAMETRO"))?;
        self.expect(TokenKind::DoisPontos, "ERR-SYN-PARAMETRO")?;
        let id_tok = self.expect(TokenKind::Id, "ERR-SYN-PARAMETRO")?;

        let mut dims = 0;
        while self.match_token(TokenKind::AbreColchete).is_some() {
            self.expect(TokenKind::FechaColchete, "ERR-SYN-PARAMETRO")?;
            dims += 1;
        }

        let label = if dims > 0 {
            format!("ID: {} ({}D)", id_tok.value, dims)
        } else {
            format!("ID: {}", id_tok.value)
        };

        Ok(AstNode::with_children(
            "Parametro",
            vec![tipo, AstNode::new(label)],
        ))
    }

    // -------------------------------------------------------------------------
    // Bloco e Ações
    // -------------------------------------------------------------------------

    /// `bloco ::= acao*` (encerra ao encontrar `FIM`, `SENAO`, `ATE` ou EOF)
    fn parse_bloco(&mut self) -> Result<AstNode, String> {
        let mut acoes = Vec::new();
        while self.current_token().is_some()
            && !self.check(TokenKind::Fim)
            && !self.check(TokenKind::Senao)
            && !self.check(TokenKind::Ate)
        {
            acoes.push(self.parse_acao()?);
        }
        Ok(AstNode::with_children("Bloco", acoes))
    }

    /// `acao ::= expressao | atribuicao | declaracao_variaveis | se | repita | leia | escreva | retorna`
    fn parse_acao(&mut self) -> Result<AstNode, String> {
        if self.check(TokenKind::Se) {
            self.parse_se()
        } else if self.check(TokenKind::Repita) {
            self.parse_repita()
        } else if self.check(TokenKind::Leia) {
            self.parse_leia()
        } else if self.check(TokenKind::Escreva) {
            self.parse_escreva()
        } else if self.check(TokenKind::Retorna) {
            self.parse_retorna()
        } else if self.is_tipo_at(0) {
            self.parse_declaracao_variaveis()
        } else {
            let lhs = self.parse_lista_expressoes()?;
            if self.match_token(TokenKind::Atribuicao).is_some() {
                let rhs = self.parse_rhs_atribuicao()?;
                Ok(AstNode::with_children("Atribuicao", vec![lhs, rhs]))
            } else {
                Ok(lhs)
            }
        }
    }

    /// Trata `:= expr` e atribuições encadeadas (`a := b := 1`).
    fn parse_rhs_atribuicao(&mut self) -> Result<AstNode, String> {
        let exprs = self.parse_lista_expressoes()?;
        if self.match_token(TokenKind::Atribuicao).is_some() {
            let rest = self.parse_rhs_atribuicao()?;
            Ok(AstNode::with_children("AtribuicaoEncadeada", vec![exprs, rest]))
        } else {
            Ok(exprs)
        }
    }

    /// `se ::= SE expressao ENTAO bloco (SENAO bloco)? FIM`
    fn parse_se(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::Se, "ERR-SYN-IRRECUPERAVEL")?;
        let cond = self.parse_expressao()?;
        self.expect(TokenKind::Entao, "ERR-SYN-IRRECUPERAVEL")?;
        let then_block = self.parse_bloco()?;

        let mut children = vec![cond, then_block];
        if self.match_token(TokenKind::Senao).is_some() {
            children.push(self.parse_bloco()?);
        }
        self.expect(TokenKind::Fim, "ERR-SYN-IRRECUPERAVEL")?;

        Ok(AstNode::with_children("Se", children))
    }

    /// `repita ::= REPITA bloco ATE expressao`
    fn parse_repita(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::Repita, "ERR-SYN-IRRECUPERAVEL")?;
        let bloco = self.parse_bloco()?;
        self.expect(TokenKind::Ate, "ERR-SYN-IRRECUPERAVEL")?;
        let cond = self.parse_expressao()?;

        Ok(AstNode::with_children("Repita", vec![bloco, cond]))
    }

    /// `leia ::= LEIA ABRE_PARENTESE var FECHA_PARENTESE`
    fn parse_leia(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::Leia, "ERR-SYN-LEIA")?;
        self.expect(TokenKind::AbreParentese, "ERR-SYN-LEIA")?;
        let var = self.parse_var().map_err(|_| self.format_error("ERR-SYN-LEIA"))?;
        self.expect(TokenKind::FechaParentese, "ERR-SYN-LEIA")?;

        Ok(AstNode::with_children("Leia", vec![var]))
    }

    /// `escreva ::= ESCREVA ABRE_PARENTESE expressao FECHA_PARENTESE`
    fn parse_escreva(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::Escreva, "ERR-SYN-IRRECUPERAVEL")?;
        self.expect(TokenKind::AbreParentese, "ERR-SYN-IRRECUPERAVEL")?;
        let expr = self.parse_expressao()?;
        self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;

        Ok(AstNode::with_children("Escreva", vec![expr]))
    }

    /// `retorna ::= RETORNA ABRE_PARENTESE lista_expressoes FECHA_PARENTESE`
    fn parse_retorna(&mut self) -> Result<AstNode, String> {
        self.expect(TokenKind::Retorna, "ERR-SYN-IRRECUPERAVEL")?;
        self.expect(TokenKind::AbreParentese, "ERR-SYN-IRRECUPERAVEL")?;
        let exprs = self.parse_lista_expressoes()?;
        self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;

        Ok(AstNode::with_children("Retorna", vec![exprs]))
    }

    // -------------------------------------------------------------------------
    // Expressões (do menor para o maior nível de precedência)
    // -------------------------------------------------------------------------

    fn parse_lista_expressoes(&mut self) -> Result<AstNode, String> {
        let first = self.parse_expressao()?;
        if !self.check(TokenKind::Virgula) {
            return Ok(first);
        }
        let mut exprs = vec![first];
        while self.match_token(TokenKind::Virgula).is_some() {
            exprs.push(self.parse_expressao()?);
        }
        Ok(AstNode::with_children("ListaExpressoes", exprs))
    }

    fn parse_expressao(&mut self) -> Result<AstNode, String> {
        self.parse_expressao_logica()
    }

    /// `expressao_logica ::= expressao_simples ((E | OU) expressao_simples)*`
    fn parse_expressao_logica(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_expressao_simples()?;

        while self.check(TokenKind::E) || self.check(TokenKind::Ou) {
            let operator = self.current_token().unwrap().clone();
            self.advance();

            let right = self.parse_expressao_simples()?;
            left = AstNode::with_children(
                "ExpressionLogic",
                vec![
                    AstNode::new(format!("Operator: {:?}", operator)),
                    left,
                    right,
                ],
            );
        }
        Ok(left)
    }

    /// `expressao_simples ::= expressao_aditiva (op_relacional expressao_aditiva)*`
    fn parse_expressao_simples(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_expression_additive()?;

        while self.check(TokenKind::Menor)
            || self.check(TokenKind::MenorIgual)
            || self.check(TokenKind::Maior)
            || self.check(TokenKind::MaiorIgual)
            || self.check(TokenKind::Igual)
            || self.check(TokenKind::Diferente)
        {
            let operator = self.current_token().unwrap().clone();
            self.advance();

            let right = self.parse_expression_additive()?;
            left = AstNode::with_children(
                "ExpressionRelational",
                vec![
                    AstNode::new(format!("Operator: {:?}", operator)),
                    left,
                    right,
                ],
            );
        }
        Ok(left)
    }

    /// `expressao_aditiva ::= expressao_multiplicativa ((MAIS | MENOS) expressao_multiplicativa)*`
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
            );
        }
        Ok(left)
    }

    /// `expressao_multiplicativa ::= expressao_unaria ((VEZES | DIVIDE) expressao_unaria)*`
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
            );
        }
        Ok(left)
    }

    /// `expressao_unaria ::= (MAIS | MENOS | NAO) expressao_unaria | fator`
    fn parse_expressao_unaria(&mut self) -> Result<AstNode, String> {
        if self.check(TokenKind::Mais)
            || self.check(TokenKind::Menos)
            || self.check(TokenKind::Nao)
        {
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

    /// `fator ::= ( expressao ) | tupla | chamada_funcao | var | numero`
    fn parse_expressao_primaria(&mut self) -> Result<AstNode, String> {
        if let Some(token) = self.current_token().cloned() {
            match token.token_type.as_str() {
                "NUM_INTEIRO" | "NUM_PONTO_FLUTUANTE" | "NUM_NOTACAO_CIENTIFICA" => {
                    let node = AstNode::new(format!("Literal: {:?}", token));
                    self.advance();
                    Ok(node)
                }
                "ID" => {
                    if self.check_at(1, TokenKind::AbreParentese) {
                        self.parse_chamada_funcao()
                    } else {
                        self.parse_var()
                    }
                }
                "ABRE_PARENTESE" => {
                    self.advance(); // Consome '('
                    let first = self.parse_expressao()?;
                    if self.check(TokenKind::Virgula) {
                        let mut elems = vec![first];
                        while self.match_token(TokenKind::Virgula).is_some() {
                            elems.push(self.parse_expressao()?);
                        }
                        self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;
                        Ok(AstNode::with_children("Tupla", elems))
                    } else {
                        self.expect(TokenKind::FechaParentese, "ERR-SYN-IRRECUPERAVEL")?;
                        Ok(first)
                    }
                }
                _ => Err(self.format_error("ERR-SYN-IRRECUPERAVEL")),
            }
        } else {
            Err(self.format_error("ERR-SYN-EOF-INESPERADO"))
        }
    }

    /// `chamada_funcao ::= ID ABRE_PARENTESE lista_argumentos FECHA_PARENTESE`
    fn parse_chamada_funcao(&mut self) -> Result<AstNode, String> {
        let id_tok = self.expect(TokenKind::Id, "ERR-SYN-IRRECUPERAVEL")?;
        self.expect(TokenKind::AbreParentese, "ERR-SYN-LISTA-ARGUMENTOS")?;

        let mut args = Vec::new();
        if !self.check(TokenKind::FechaParentese) {
            args.push(self.parse_expressao()?);
            while self.match_token(TokenKind::Virgula).is_some() {
                args.push(self.parse_expressao()?);
            }
        }
        self.expect(TokenKind::FechaParentese, "ERR-SYN-LISTA-ARGUMENTOS")?;

        Ok(AstNode::with_children(
            "ChamadaFuncao",
            vec![
                AstNode::new(format!("ID: {}", id_tok.value)),
                AstNode::with_children("ListaArgumentos", args),
            ],
        ))
    }
}

use crate::myerror::MyError;
use crate::tppparser::ast::AstNode;
use crate::tppsemantic::rules;
use crate::tppsemantic::symbol_table::{FuncSymbol, SymbolTable, TppType, VarSymbol};

/// Contexto responsável por coletar erros e avisos semânticos formatados via `MyError`.
pub struct SemaContext {
    error_handler: MyError,
    check_key: bool,
    pub messages: Vec<String>,
    pub has_errors: bool,
}

impl SemaContext {
    pub fn new(check_key: bool) -> Self {
        Self {
            error_handler: MyError::new("SemaErrors"),
            check_key,
            messages: Vec::new(),
            has_errors: false,
        }
    }

    pub fn emit(&mut self, key: &str, args: &[&str], line: usize, column: usize) {
        if key.to_uppercase().starts_with("ERR-") {
            self.has_errors = true;
        }
        let line_opt = if line > 0 { Some(line) } else { None };
        let col_opt = if column > 0 { Some(column) } else { None };
        let formatted = self
            .error_handler
            .new_error(self.check_key, key, line_opt, col_opt, args, &[]);
        self.messages.push(formatted);
    }
}

/// Resultado da análise semântica.
pub struct SemaResult {
    pub table: SymbolTable,
    pub messages: Vec<String>,
    pub has_errors: bool,
    pub pruned_ast: AstNode,
}

/// Executa a análise semântica sobre a AST produzida pelo parser.
pub fn analyze(ast: &AstNode, check_key: bool) -> SemaResult {
    let mut table = SymbolTable::new();
    let mut sema = SemaContext::new(check_key);

    visit_node(ast, "global", &[], &mut table, &mut sema);

    rules::functions::verificar(ast, &mut table, &mut sema);
    check_unused_variables(&table, &mut sema);

    let pruned_ast = build_pruned_ast(ast, "global", &table);

    SemaResult {
        table,
        messages: sema.messages,
        has_errors: sema.has_errors,
        pruned_ast,
    }
}

/// Gera a Árvore Sintática Abstrata Podada e Anotada (ASTO) para facilitar a geração de código.
pub fn build_pruned_ast(node: &AstNode, scope: &str, table: &SymbolTable) -> AstNode {
    let next_scope = if node.node_type == "Cabecalho" {
        node.children
            .first()
            .and_then(parse_id_label)
            .unwrap_or(scope)
    } else {
        scope
    };

    // Colapsa wrappers de lista com apenas 1 filho (ex.: ListaExpressoes / ListaAlvos)
    if (node.node_type == "ListaExpressoes" || node.node_type == "ListaAlvos")
        && node.children.len() == 1
    {
        return build_pruned_ast(&node.children[0], next_scope, table);
    }

    let pruned_children: Vec<AstNode> = node
        .children
        .iter()
        .filter(|c| {
            !((c.node_type == "ListaParametros" || c.node_type == "ListaArgumentos")
                && c.children.is_empty())
        })
        .map(|c| build_pruned_ast(c, next_scope, table))
        .collect();

    let label = if let Some((tok_type, val)) = parse_literal_info(node) {
        let ty = match tok_type {
            "NUM_INTEIRO" => "inteiro",
            _ => "flutuante",
        };
        format!("Literal: {} [{}]", val, ty)
    } else if let Some(rest) = node.node_type.strip_prefix("Operator: ") {
        if let Some(op_val) = extract_debug_field(rest, "value") {
            format!("Op: {}", op_val)
        } else {
            node.node_type.clone()
        }
    } else if matches!(
        node.node_type.as_str(),
        "Var"
            | "ChamadaFuncao"
            | "ExpressionAdditive"
            | "ExpressionMultiplicative"
            | "ExpressionRelational"
            | "ExpressionLogic"
            | "ExpressionUnary"
    ) {
        if let Some(inferred) = infer_expr_type(node, next_scope, table) {
            format!("{} [{}]", node.node_type, inferred.as_str())
        } else {
            node.node_type.clone()
        }
    } else {
        node.node_type.clone()
    };

    AstNode::with_position(label, node.line, node.column, pruned_children)
}

fn visit_node(
    node: &AstNode,
    scope: &str,
    ret_types: &[TppType],
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    match node.node_type.as_str() {
        "Programa" | "ListaArgumentos" => {
            for child in &node.children {
                visit_node(child, scope, ret_types, table, sema);
            }
        }
        "Bloco" => handle_bloco(node, scope, ret_types, table, sema),
        "DeclaracaoVariaveis" => handle_var_decl(node, scope, table, sema),
        "InicializacaoVariaveis" => handle_var_init(node, scope, table, sema),
        "Atribuicao" => handle_atribuicao(node, scope, table, sema),
        "AtribuicaoEncadeada" => handle_atribuicao_encadeada(node, scope, table, sema),
        "DeclaracaoFuncao" => handle_func_decl(node, table, sema),
        "ChamadaFuncao" => handle_func_call(node, scope, table, sema),
        "Leia" => handle_leia(node, scope, table, sema),
        "Retorna" => handle_retorna(node, scope, ret_types, table, sema),
        "ExpressionRelational" => handle_relational(node, scope, ret_types, table, sema),
        "ExpressionAdditive"
        | "ExpressionMultiplicative"
        | "ExpressionLogic"
        | "ExpressionUnary" => handle_operator_expr(node, scope, ret_types, table, sema),
        "Tupla" => {
            sema.emit("ERR-SEM-TUPLE-NOT-ALLOWED", &[], node.line, node.column);
            for child in &node.children {
                visit_node(child, scope, ret_types, table, sema);
            }
        }
        "Var" => handle_var_read(node, scope, table, sema),
        _ => {
            for child in &node.children {
                visit_node(child, scope, ret_types, table, sema);
            }
        }
    }
}

fn is_no_effect_expr(node: &AstNode) -> bool {
    matches!(
        node.node_type.as_str(),
        "ExpressionAdditive"
            | "ExpressionMultiplicative"
            | "ExpressionRelational"
            | "ExpressionLogic"
            | "ExpressionUnary"
            | "Var"
            | "Tupla"
            | "ListaExpressoes"
    ) || node.node_type.starts_with("Literal: ")
}

fn handle_bloco(
    node: &AstNode,
    scope: &str,
    ret_types: &[TppType],
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    for stmt in &node.children {
        if is_no_effect_expr(stmt) {
            sema.emit("WAR-SEM-EXP-STMT-NO-EFFECT", &[], stmt.line, stmt.column);
        }
        visit_node(stmt, scope, ret_types, table, sema);
    }
}

fn check_multi_ret_in_operands(
    node: &AstNode,
    table: &SymbolTable,
    sema: &mut SemaContext,
) {
    for child in &node.children {
        if child.node_type == "ChamadaFuncao" {
            if let Some(func_name) = child.children.first().and_then(parse_id_label) {
                if let Some(func) = table.get_function(func_name) {
                    if func.return_types.len() > 1 {
                        let count_s = func.return_types.len().to_string();
                        sema.emit(
                            "ERR-SEM-CALL-FUNC-MULTI-RET-IN-EXP",
                            &[func_name, &count_s],
                            child.line,
                            child.column,
                        );
                    }
                }
            }
        }
    }
}

fn handle_operator_expr(
    node: &AstNode,
    scope: &str,
    ret_types: &[TppType],
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    check_multi_ret_in_operands(node, table, sema);
    check_multi_ret_in_operands(node, table, sema);
    for child in &node.children {
        visit_node(child, scope, ret_types, table, sema);
    }
}

fn parse_tipo_label(node: &AstNode) -> Option<TppType> {
    node.node_type
        .strip_prefix("Tipo: ")
        .and_then(TppType::from_str)
}

fn parse_id_label(node: &AstNode) -> Option<&str> {
    node.node_type.strip_prefix("ID: ")
}

/// Extrai `(token_type, value)` de nós gerados com `format!("Literal: {:?}", token)`.
fn parse_literal_info(node: &AstNode) -> Option<(&str, &str)> {
    let rest = node.node_type.strip_prefix("Literal: ")?;
    let token_type = extract_debug_field(rest, "token_type")?;
    let value = extract_debug_field(rest, "value")?;
    Some((token_type, value))
}

fn extract_debug_field<'a>(debug_str: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("{}: \"", field);
    let start = debug_str.find(&marker)? + marker.len();
    let end = debug_str[start..].find('"')? + start;
    Some(&debug_str[start..end])
}

fn eval_const_int(node: &AstNode) -> Option<isize> {
    if let Some((tok_type, val)) = parse_literal_info(node) {
        if tok_type == "NUM_INTEIRO" {
            return val.parse::<isize>().ok();
        }
        return None;
    }
    if node.node_type == "ExpressionUnary" {
        let op = node.children.first()?.node_type.as_str();
        let inner = eval_const_int(node.children.get(1)?)?;
        if op.contains("token_type: \"MENOS\"") {
            return Some(-inner);
        }
        if op.contains("token_type: \"MAIS\"") {
            return Some(inner);
        }
    }
    None
}

fn infer_expr_type(node: &AstNode, scope: &str, table: &SymbolTable) -> Option<TppType> {
    if let Some((tok_type, _)) = parse_literal_info(node) {
        return match tok_type {
            "NUM_INTEIRO" => Some(TppType::Inteiro),
            "NUM_PONTO_FLUTUANTE" | "NUM_NOTACAO_CIENTIFICA" => Some(TppType::Flutuante),
            _ => None,
        };
    }

    match node.node_type.as_str() {
        "Var" => {
            let name = node.children.first().and_then(parse_id_label)?;
            table.get_variable(name, scope).map(|v| v.var_type)
        }
        "ChamadaFuncao" => {
            let name = node.children.first().and_then(parse_id_label)?;
            table
                .get_function(name)
                .and_then(|f| f.return_types.first().copied())
        }
        "ExpressionUnary" => {
            let op = node.children.first()?.node_type.as_str();
            if op.contains("token_type: \"NAO\"") {
                return Some(TppType::Inteiro);
            }
            node.children
                .get(1)
                .and_then(|operand| infer_expr_type(operand, scope, table))
        }
        "ExpressionAdditive" | "ExpressionMultiplicative" => {
            let left = node
                .children
                .get(1)
                .and_then(|n| infer_expr_type(n, scope, table));
            let right = node
                .children
                .get(2)
                .and_then(|n| infer_expr_type(n, scope, table));
            match (left, right) {
                (Some(TppType::Flutuante), _) | (_, Some(TppType::Flutuante)) => {
                    Some(TppType::Flutuante)
                }
                (Some(TppType::Inteiro), Some(TppType::Inteiro)) => Some(TppType::Inteiro),
                (Some(t), None) | (None, Some(t)) => Some(t),
                _ => None,
            }
        }
        "ExpressionRelational" | "ExpressionLogic" => Some(TppType::Inteiro),
        _ => None,
    }
}

fn handle_relational(
    node: &AstNode,
    scope: &str,
    ret_types: &[TppType],
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    let left_is_rel = node
        .children
        .get(1)
        .is_some_and(|c| c.node_type == "ExpressionRelational");
    let right_is_rel = node
        .children
        .get(2)
        .is_some_and(|c| c.node_type == "ExpressionRelational");
    if left_is_rel || right_is_rel {
        sema.emit("WAR-SEM-REL-CHAIN", &[], node.line, node.column);
    }

    check_multi_ret_in_operands(node, table, sema);
    for child in &node.children {
        visit_node(child, scope, ret_types, table, sema);
    }
}

fn handle_var_decl(
    node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    let Some(tipo_node) = node.children.first() else {
        return;
    };
    let Some(var_type) = parse_tipo_label(tipo_node) else {
        return;
    };
    let Some(lista_vars) = node.children.get(1) else {
        return;
    };
    let has_initializer = node.children.len() > 2;

    if let Some(rhs) = node.children.get(2) {
        visit_node(rhs, scope, &[], table, sema);
    }

    for var_node in &lista_vars.children {
        if var_node.node_type != "Var" {
            continue;
        }
        let Some(name) = var_node.children.first().and_then(parse_id_label) else {
            continue;
        };

        let mut dimensions = Vec::new();
        for idx_node in var_node.children.iter().skip(1) {
            if idx_node.node_type == "Indice" {
                let mut dim_size = None;
                if let Some(expr) = idx_node.children.first() {
                    visit_node(expr, scope, &[], table, sema);
                    if let Some(idx_type) = infer_expr_type(expr, scope, table) {
                        if idx_type != TppType::Inteiro {
                            sema.emit(
                                "ERR-SEM-ARRAY-INDEX-NOT-INT",
                                &[name],
                                var_node.line,
                                var_node.column,
                            );
                        }
                    }
                    if let Some(val) = eval_const_int(expr) {
                        if val <= 0 {
                            sema.emit(
                                "ERR-SEM-ARRAY-INDEX-OUT-OF-RANGE",
                                &[name],
                                var_node.line,
                                var_node.column,
                            );
                        } else {
                            dim_size = Some(val as usize);
                        }
                    }
                }
                dimensions.push(dim_size);
            }
        }

        let symbol = VarSymbol {
            name: name.to_string(),
            var_type,
            dimensions,
            scope: scope.to_string(),
            initialized: has_initializer,
            used: false,
            line: var_node.line,
            column: var_node.column,
        };

        if let Err(prev_type) = table.declare_variable(symbol) {
            sema.emit(
                "WAR-SEM-VAR-DECL-PREV",
                &[name, prev_type.as_str()],
                var_node.line,
                var_node.column,
            );
        }
    }
}

fn handle_var_init(
    node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if let Some(rhs) = node.children.get(1) {
        visit_assignment_rhs(rhs, scope, table, sema);
    }

    if let Some(alvos) = node.children.first() {
        if let Some(rhs) = node.children.get(1) {
            check_assignment_arity(alvos, rhs, node, table, sema);
        }
        mark_targets_written(alvos, node.children.get(1), scope, table, sema);
    }
}

fn handle_atribuicao(
    node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if let Some(rhs) = node.children.get(1) {
        visit_assignment_rhs(rhs, scope, table, sema);
    }
    if let Some(lhs) = node.children.first() {
        let rhs_effective = unwrap_chained_rhs(node.children.get(1));
        if let Some(rhs) = rhs_effective {
            check_assignment_arity(lhs, rhs, node, table, sema);
        }
        mark_targets_written(lhs, rhs_effective, scope, table, sema);
    }
}

fn handle_atribuicao_encadeada(
    node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if let Some(rest) = node.children.get(1) {
        visit_assignment_rhs(rest, scope, table, sema);
    }
    if let Some(targets) = node.children.first() {
        let rhs_effective = unwrap_chained_rhs(node.children.get(1));
        if let Some(rhs) = rhs_effective {
            check_assignment_arity(targets, rhs, node, table, sema);
        }
        mark_targets_written(targets, rhs_effective, scope, table, sema);
    }
}

fn visit_assignment_rhs(
    rhs: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if rhs.node_type == "Tupla" || rhs.node_type == "ListaExpressoes" {
        for child in &rhs.children {
            visit_node(child, scope, &[], table, sema);
        }
    } else {
        visit_node(rhs, scope, &[], table, sema);
    }
}

fn count_targets(node: &AstNode) -> usize {
    match node.node_type.as_str() {
        "ListaExpressoes" | "ListaAlvos" | "Tupla" => {
            node.children.iter().map(count_targets).sum()
        }
        _ => 1,
    }
}

fn count_values(node: &AstNode, table: &SymbolTable) -> usize {
    match node.node_type.as_str() {
        "ListaExpressoes" | "Tupla" => node.children.iter().map(|c| count_values(c, table)).sum(),
        "ChamadaFuncao" => node
            .children
            .first()
            .and_then(parse_id_label)
            .and_then(|name| table.get_function(name))
            .map(|f| f.return_types.len().max(1))
            .unwrap_or(1),
        _ => 1,
    }
}

fn check_assignment_arity(
    lhs: &AstNode,
    rhs: &AstNode,
    atr_node: &AstNode,
    table: &SymbolTable,
    sema: &mut SemaContext,
) {
    let n_targets = count_targets(lhs);
    let n_values = count_values(rhs, table);
    if n_targets != n_values {
        let t_s = n_targets.to_string();
        let v_s = n_values.to_string();
        sema.emit(
            "ERR-SEM-ATR-DIFF-NUM-TARGETS-VALUES",
            &[&t_s, &v_s],
            atr_node.line,
            atr_node.column,
        );
    }
}

fn unwrap_chained_rhs(mut node: Option<&AstNode>) -> Option<&AstNode> {
    while let Some(n) = node {
        if n.node_type == "AtribuicaoEncadeada" {
            node = n.children.get(1);
        } else {
            return Some(n);
        }
    }
    None
}

fn mark_targets_written(
    node: &AstNode,
    rhs: Option<&AstNode>,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    match node.node_type.as_str() {
        "ListaExpressoes" | "ListaAlvos" | "Tupla" => {
            for (i, child) in node.children.iter().enumerate() {
                let child_rhs = rhs.and_then(|r| {
                    if r.node_type == "ListaExpressoes" || r.node_type == "Tupla" {
                        r.children.get(i)
                    } else {
                        Some(r)
                    }
                });
                mark_targets_written(child, child_rhs, scope, table, sema);
            }
        }
        "Var" => {
            mark_var_written(node, scope, table, sema);
            if let Some(rhs_node) = rhs {
                check_assignment_coercion(node, rhs_node, scope, table, sema);
            }
        }
        _ => {
            sema.emit(
                "ERR-SEM-ATR-INVALID-TARGET",
                &[&node.node_type],
                node.line,
                node.column,
            );
            visit_node(node, scope, &[], table, sema);
        }
    }
}

fn check_assignment_coercion(
    target_var: &AstNode,
    rhs_node: &AstNode,
    scope: &str,
    table: &SymbolTable,
    sema: &mut SemaContext,
) {
    let Some(target_name) = target_var.children.first().and_then(parse_id_label) else {
        return;
    };
    let Some(target_sym) = table.get_variable(target_name, scope) else {
        return;
    };
    let Some(rhs_type) = infer_expr_type(rhs_node, scope, table) else {
        return;
    };

    if target_sym.var_type == rhs_type {
        return;
    }

    if let Some((_, lit_val)) = parse_literal_info(rhs_node) {
        sema.emit(
            "WAR-SEM-ATR-DIFF-TYPES-IMP-COERC-OF-NUM",
            &[
                lit_val,
                rhs_type.as_str(),
                target_name,
                target_sym.var_type.as_str(),
            ],
            target_var.line,
            target_var.column,
        );
    } else if rhs_node.node_type == "Var" {
        if let Some(rhs_name) = rhs_node.children.first().and_then(parse_id_label) {
            sema.emit(
                "WAR-SEM-ATR-DIFF-TYPES-IMP-COERC-OF-VAR",
                &[
                    rhs_name,
                    rhs_type.as_str(),
                    target_name,
                    target_sym.var_type.as_str(),
                ],
                target_var.line,
                target_var.column,
            );
        }
    } else if rhs_node.node_type == "ChamadaFuncao" {
        if let Some(func_name) = rhs_node.children.first().and_then(parse_id_label) {
            sema.emit(
                "WAR-SEM-ATR-DIFF-TYPES-IMP-COERC-OF-RET-VAL",
                &[
                    func_name,
                    rhs_type.as_str(),
                    target_name,
                    target_sym.var_type.as_str(),
                ],
                target_var.line,
                target_var.column,
            );
        }
    } else {
        sema.emit(
            "WAR-SEM-ATR-DIFF-TYPES-IMP-COERC-OF-EXP",
            &[
                "expressão",
                rhs_type.as_str(),
                target_name,
                target_sym.var_type.as_str(),
            ],
            target_var.line,
            target_var.column,
        );
    }
}

fn handle_leia(node: &AstNode, scope: &str, table: &mut SymbolTable, sema: &mut SemaContext) {
    if let Some(target) = node.children.first() {
        mark_var_written(target, scope, table, sema);
    }
}

fn check_array_access_indices(
    var_node: &AstNode,
    var_name: &str,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) -> bool {
    let mut all_valid = true;
    for idx_node in var_node.children.iter().skip(1) {
        visit_node(idx_node, scope, &[], table, sema);
    }

    let dims = table
        .get_variable(var_name, scope)
        .map(|v| v.dimensions.clone());

    for (dim_idx, idx_node) in var_node.children.iter().skip(1).enumerate() {
        if idx_node.node_type != "Indice" {
            continue;
        }
        let Some(expr) = idx_node.children.first() else {
            continue;
        };
        if let Some(idx_type) = infer_expr_type(expr, scope, table) {
            if idx_type != TppType::Inteiro {
                all_valid = false;
                sema.emit(
                    "ERR-SEM-ARRAY-INDEX-NOT-INT",
                    &[var_name],
                    var_node.line,
                    var_node.column,
                );
            }
        }
        if let Some(val) = eval_const_int(expr) {
            let out_of_bounds = if val < 0 {
                true
            } else if let Some(Some(limit)) = dims.as_ref().and_then(|d| d.get(dim_idx)).copied() {
                (val as usize) >= limit
            } else {
                false
            };
            if out_of_bounds {
                all_valid = false;
                sema.emit(
                    "ERR-SEM-ARRAY-INDEX-OUT-OF-RANGE",
                    &[var_name],
                    var_node.line,
                    var_node.column,
                );
            }
        }
    }
    all_valid
}

fn mark_var_written(
    var_node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if var_node.node_type != "Var" {
        visit_node(var_node, scope, &[], table, sema);
        return;
    }

    let Some(name) = var_node.children.first().and_then(parse_id_label) else {
        return;
    };

    let indices_valid = check_array_access_indices(var_node, name, scope, table, sema);

    if let Some(var) = table.get_variable_mut(name, scope) {
        if indices_valid {
            var.initialized = true;
        }
    } else {
        sema.emit("ERR-SEM-VAR-NOT-DECL", &[name], var_node.line, var_node.column);
    }
}

fn handle_var_read(
    var_node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    let Some(name) = var_node.children.first().and_then(parse_id_label) else {
        return;
    };

    check_array_access_indices(var_node, name, scope, table, sema);

    if let Some(var) = table.get_variable_mut(name, scope) {
        var.used = true;
        if !var.initialized {
            sema.emit(
                "WAR-SEM-VAR-DECL-NOT-INIT",
                &[name],
                var_node.line,
                var_node.column,
            );
        }
    } else {
        sema.emit("ERR-SEM-VAR-NOT-DECL", &[name], var_node.line, var_node.column);
    }
}

fn handle_retorna(
    node: &AstNode,
    scope: &str,
    ret_types: &[TppType],
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    if let Some(exprs) = node.children.first() {
        if exprs.node_type == "ListaExpressoes" || exprs.node_type == "Tupla" {
            for child in &exprs.children {
                visit_node(child, scope, ret_types, table, sema);
            }
        } else {
            visit_node(exprs, scope, ret_types, table, sema);
        }

        let returned_nodes: Vec<&AstNode> =
            if exprs.node_type == "ListaExpressoes" || exprs.node_type == "Tupla" {
                exprs.children.iter().collect()
            } else {
                vec![exprs]
            };

        if ret_types.len() == 1 && ret_types[0] == TppType::Vazio {
            if let Some(first_type) = returned_nodes
                .first()
                .and_then(|n| infer_expr_type(n, scope, table))
            {
                sema.emit(
                    "ERR-SEM-FUNC-RET-TYPE-ERROR",
                    &[scope, TppType::Vazio.as_str(), first_type.as_str()],
                    node.line,
                    node.column,
                );
            }
            return;
        }

        if returned_nodes.len() != ret_types.len() {
            let expected_s = ret_types.len().to_string();
            let got_s = returned_nodes.len().to_string();
            sema.emit(
                "ERR-SEM-FUNC-RET-NUM-ERROR",
                &[scope, &expected_s, &got_s],
                node.line,
                node.column,
            );
            return;
        }

        for (ret_node, expected_type) in returned_nodes.iter().zip(ret_types.iter()) {
            if let Some(actual_type) = infer_expr_type(ret_node, scope, table) {
                if actual_type != *expected_type {
                    sema.emit(
                        "ERR-SEM-FUNC-RET-TYPE-ERROR",
                        &[scope, expected_type.as_str(), actual_type.as_str()],
                        node.line,
                        node.column,
                    );
                }
            }
        }
    }
}

fn handle_func_decl(node: &AstNode, table: &mut SymbolTable, sema: &mut SemaContext) {
    let mut return_types = Vec::new();
    let mut cabecalho_opt = None;

    for child in &node.children {
        if child.node_type.starts_with("Tipo: ") {
            if let Some(t) = parse_tipo_label(child) {
                return_types.push(t);
            }
        } else if child.node_type == "TiposRetorno" {
            for t_node in &child.children {
                if let Some(t) = parse_tipo_label(t_node) {
                    return_types.push(t);
                }
            }
        } else if child.node_type == "Cabecalho" {
            cabecalho_opt = Some(child);
        }
    }

    if return_types.is_empty() {
        return_types.push(TppType::Vazio);
    }

    let Some(cabecalho) = cabecalho_opt else {
        return;
    };
    let Some(func_name) = cabecalho.children.first().and_then(parse_id_label) else {
        return;
    };
    let func_scope = func_name.to_string();

    let mut params = Vec::new();
    if let Some(lista_params) = cabecalho.children.get(1) {
        for param_node in &lista_params.children {
            if let Some(param_sym) = parse_param(param_node, &func_scope) {
                params.push(param_sym);
            }
        }
    }

    let func_sym = FuncSymbol {
        name: func_scope.clone(),
        return_types: return_types.clone(),
        params: params.clone(),
        used: false,
        line: cabecalho.line,
        column: cabecalho.column,
    };

    let _ = table.declare_function(func_sym);

    for param in params {
        if let Err(prev_type) = table.declare_variable(param.clone()) {
            sema.emit(
                "WAR-SEM-VAR-DECL-PREV",
                &[&param.name, prev_type.as_str()],
                param.line,
                param.column,
            );
        }
    }

    if let Some(bloco) = cabecalho.children.get(2) {
        visit_node(bloco, &func_scope, &return_types, table, sema);
        let expects_value = !(return_types.len() == 1 && return_types[0] == TppType::Vazio);
        if expects_value && !has_return(bloco) {
            sema.emit(
                "ERR-SEM-FUNC-RET-TYPE-ERROR",
                &[&func_scope, return_types[0].as_str(), TppType::Vazio.as_str()],
                cabecalho.line,
                cabecalho.column,
            );
        }
    }
}

fn has_return(node: &AstNode) -> bool {
    if node.node_type == "Retorna" {
        return true;
    }
    node.children.iter().any(has_return)
}

fn parse_param(param_node: &AstNode, scope: &str) -> Option<VarSymbol> {
    let tipo_node = param_node.children.first()?;
    let var_type = parse_tipo_label(tipo_node)?;
    let id_label = param_node.children.get(1).and_then(parse_id_label)?;

    let (name, dims_count) = if let Some((base, rest)) = id_label.split_once(" (") {
        let count = rest
            .strip_suffix("D)")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        (base, count)
    } else {
        (id_label, 0)
    };

    Some(VarSymbol {
        name: name.to_string(),
        var_type,
        dimensions: vec![None; dims_count],
        scope: scope.to_string(),
        initialized: true,
        used: false,
        line: param_node.line,
        column: param_node.column,
    })
}

fn handle_func_call(
    node: &AstNode,
    scope: &str,
    table: &mut SymbolTable,
    sema: &mut SemaContext,
) {
    let Some(func_name) = node.children.first().and_then(parse_id_label) else {
        return;
    };
    let args_count = node
        .children
        .get(1)
        .map(|lista| lista.children.len())
        .unwrap_or(0);

    if func_name == "principal" {
        if scope == "principal" {
            sema.emit("WAR-SEM-CALL-REC-FUNC-MAIN", &[], node.line, node.column);
        } else {
            sema.emit(
                "ERR-SEM-CALL-FUNC-MAIN-NOT-ALLOWED",
                &[],
                node.line,
                node.column,
            );
        }
    }

    if let Some(lista_args) = node.children.get(1) {
        visit_node(lista_args, scope, &[], table, sema);
    }

    let param_types: Option<Vec<TppType>> = if let Some(func) = table.get_function_mut(func_name) {
        func.used = true;
        let expected = func.params.len();
        if args_count < expected {
            sema.emit(
                "ERR-SEM-CALL-FUNC-WITH-FEW-ARGS",
                &[func_name],
                node.line,
                node.column,
            );
        } else if args_count > expected {
            sema.emit(
                "ERR-SEM-CALL-FUNC-WITH-MANY-ARGS",
                &[func_name],
                node.line,
                node.column,
            );
        }
        Some(func.params.iter().map(|p| p.var_type).collect())
    } else {
        if func_name != "principal" {
            sema.emit(
                "ERR-SEM-CALL-FUNC-NOT-DECL",
                &[func_name],
                node.line,
                node.column,
            );
        }
        None
    };

    if let (Some(expected_types), Some(lista_args)) = (param_types, node.children.get(1)) {
        for (arg_node, expected_type) in lista_args.children.iter().zip(expected_types.iter()) {
            if let Some(arg_type) = infer_expr_type(arg_node, scope, table) {
                if arg_type != *expected_type {
                    sema.emit(
                        "WAR-SEM-ATR-DIFF-TYPES-IMP-COERC-OF-FUNC-ARG",
                        &[func_name, arg_type.as_str(), expected_type.as_str()],
                        node.line,
                        node.column,
                    );
                }
            }
        }
    }
}

fn check_unused_variables(table: &SymbolTable, sema: &mut SemaContext) {
    for var in table.global_vars.values() {
        emit_unused_var(var, sema);
    }
    for scope_vars in table.local_vars.values() {
        for var in scope_vars.values() {
            emit_unused_var(var, sema);
        }
    }
}

fn emit_unused_var(var: &VarSymbol, sema: &mut SemaContext) {
    if var.used {
        return;
    }
    if var.initialized {
        sema.emit(
            "WAR-SEM-VAR-DECL-INIT-NOT-USED",
            &[&var.name],
            var.line,
            var.column,
        );
    } else {
        sema.emit(
            "WAR-SEM-VAR-DECL-NOT-USED",
            &[&var.name],
            var.line,
            var.column,
        );
    }
}

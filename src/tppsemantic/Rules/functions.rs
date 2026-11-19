pub fn verificar(ast: &AstNode, table: &mut SymbolTable, sema: &mut SemaContext) {
    if !table.functions.contains_key("principal") {
        sema.emit("ERR-SEM-MAIN-NOT-DECL", &[], ast.line, ast.column);
    }

    for func in table.functions.values() {
        if func.name != "principal" && !func.used {
            sema.emit("WAR-SEM-FUNC-DECL-NOT-USED", &[&func.name], func.line, func.column);
        }
    }
}
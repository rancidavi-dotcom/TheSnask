use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, LiteralValue, BinaryOp, UnaryOp, LoopStmt, FuncDecl};
use crate::types::Type;
use std::collections::HashSet;

pub struct CGenerator {
}

fn is_library_native(name: &str) -> bool {
    if name.contains("::") {
        return false;
    }
    name.starts_with("sqlite_")
        || name.starts_with("gui_")
        || name.starts_with("blaze_")
        || name.starts_with("auth_")
        || name.starts_with("sfs_")
        || name.starts_with("path_")
        || name.starts_with("os_")
        || name.starts_with("s_http_")
        || name.starts_with("thread_")
        || name.starts_with("json_")
        || name.starts_with("sjson_")
        || name.starts_with("snif_")
        || name.starts_with("string_")
}

fn library_native_lib(name: &str) -> &'static str {
    if name.starts_with("sqlite_") { "sqlite" }
    else if name.starts_with("gui_") { "gui" }
    else if name.starts_with("blaze_") { "blaze" }
    else if name.starts_with("auth_") { "blaze_auth" }
    else if name.starts_with("sfs_") || name.starts_with("path_") { "sfs" }
    else if name.starts_with("os_") { "os" }
    else if name.starts_with("s_http_") { "requests" }
    else if name.starts_with("thread_") { "os" }
    else if name.starts_with("json_") { "json" }
    else if name.starts_with("sjson_") { "sjson" }
    else if name.starts_with("snif_") { "snif" }
    else if name.starts_with("string_") { "string" }
    else { "a library" }
}

impl CGenerator {
    pub fn new() -> Self {
        CGenerator { }
    }

    pub fn generate_complete(&mut self, program: Program) -> String {
        let mut prelude = String::new();
        let mut globals_decl = String::new();
        let mut functions = String::new();
        let mut main_body = String::new();

        self.emit_prelude_to(&mut prelude);

        let mut global_vars = HashSet::new();
        self.collect_vars(&program, &mut global_vars);

        for name in &global_vars {
            globals_decl.push_str(&format!("SnaskValue {};\n", name));
        }

        for stmt in program {
            match stmt.kind {
                StmtKind::FuncDeclaration(func) => {
                    self.emit_function_to(&mut functions, func);
                }
                StmtKind::VarDeclaration(decl) => {
                    let value_code = self.emit_expression_val(decl.value);
                    main_body.push_str(&format!("    {} = {};\n", decl.name, value_code));
                }
                StmtKind::MutDeclaration(decl) => {
                    let value_code = self.emit_expression_val(decl.value);
                    main_body.push_str(&format!("    {} = {};\n", decl.name, value_code));
                }
                StmtKind::ConstDeclaration(decl) => {
                    let value_code = self.emit_expression_val(decl.value);
                    main_body.push_str(&format!("    {} = {};\n", decl.name, value_code));
                }
                _ => {
                    self.emit_statement_to(&mut main_body, stmt, 1);
                }
            }
        }

        format!("{}\n{}\n{}\nint main() {{\n{}    return 0;\n}}\n", prelude, globals_decl, functions, main_body)
    }

    fn collect_vars(&self, program: &Program, vars: &mut HashSet<String>) {
        for stmt in program {
            match &stmt.kind {
                StmtKind::VarDeclaration(d) => { vars.insert(d.name.clone()); }
                StmtKind::MutDeclaration(d) => { vars.insert(d.name.clone()); }
                StmtKind::ConstDeclaration(d) => { vars.insert(d.name.clone()); }
                StmtKind::Conditional(c) => {
                    self.collect_vars(&c.if_block.body, vars);
                    for elif in &c.elif_blocks { self.collect_vars(&elif.body, vars); }
                    if let Some(else_b) = &c.else_block { self.collect_vars(else_b, vars); }
                }
                StmtKind::Loop(l) => match l {
                    LoopStmt::While { body, .. } => self.collect_vars(body, vars),
                    LoopStmt::For { iterator, body, .. } => {
                        vars.insert(iterator.clone());
                        self.collect_vars(body, vars);
                    }
                },
                _ => {}
            }
        }
    }

    fn emit_prelude_to(&self, out: &mut String) {
        out.push_str("#include <stdio.h>\n");
        out.push_str("#include <stdlib.h>\n");
        out.push_str("#include <stdbool.h>\n");
        out.push_str("#include <string.h>\n");
        out.push_str("#include <sys/stat.h>\n");
        out.push_str("#include <unistd.h>\n");
        out.push_str("#include <dirent.h>\n\n");

        out.push_str("typedef enum { SNASK_NIL, SNASK_NUMBER, SNASK_BOOL, SNASK_STRING, SNASK_LIST } SnaskType;\n\n");
        out.push_str("struct SnaskValue;\n\n");
        out.push_str("typedef struct { struct SnaskValue* items; int length; } SnaskList;\n\n");
        out.push_str("typedef struct SnaskValue {\n");
        out.push_str("    SnaskType type;\n");
        out.push_str("    union {\n");
        out.push_str("        double number;\n");
        out.push_str("        bool boolean;\n");
        out.push_str("        char* string;\n");
        out.push_str("        SnaskList list;\n");
        out.push_str("    } as;\n");
        out.push_str("} SnaskValue;\n\n");

        out.push_str("static SnaskValue snask__restricted_native(const char* name, const char* lib) {\n");
        out.push_str("    fprintf(stderr, \"Compilation error: Native function '%s' is reserved for libraries.\\\\n\\\\nHow to fix:\\\\n- Use import \\\"%s\\\" and call functions via the module namespace (e.g. %s::...).\\\\n\", name, lib, lib);\n");
        out.push_str("    exit(1);\n");
        out.push_str("    return (SnaskValue){.type = SNASK_NIL};\n");
        out.push_str("}\n\n");
        // Helpers
        out.push_str("void print_value(SnaskValue v) {\n");
        out.push_str("    switch(v.type) {\n");
        out.push_str("        case SNASK_NUMBER: printf(\"%g\", v.as.number); break;\n");
        out.push_str("        case SNASK_BOOL: printf(\"%s\", v.as.boolean ? \"true\" : \"false\"); break;\n");
        out.push_str("        case SNASK_STRING: printf(\"%s\", v.as.string); break;\n");
        out.push_str("        case SNASK_NIL: printf(\"nil\"); break;\n");
        out.push_str("        case SNASK_LIST: {\n");
        out.push_str("            printf(\"[\");\n");
        out.push_str("            for(int i=0; i < v.as.list.length; i++) {\n");
        out.push_str("                print_value(v.as.list.items[i]);\n");
        out.push_str("                if(i < v.as.list.length - 1) printf(\", \");\n");
        out.push_str("            }\n");
        out.push_str("            printf(\"]\");\n");
        out.push_str("            break;\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // IO & FS Core
        out.push_str("SnaskValue fs_read(SnaskValue path) {\n");
        out.push_str("    FILE *f = fopen(path.as.string, \"rb\");\n");
        out.push_str("    if (!f) return (SnaskValue){.type = SNASK_NIL};\n");
        out.push_str("    fseek(f, 0, SEEK_END); long fsize = ftell(f); fseek(f, 0, SEEK_SET);\n");
        out.push_str("    char *string = malloc(fsize + 1); fread(string, fsize, 1, f); fclose(f);\n");
        out.push_str("    string[fsize] = 0; return (SnaskValue){.type = SNASK_STRING, .as.string = string};\n");
        out.push_str("}\n\n");

        out.push_str("SnaskValue fs_write(SnaskValue path, SnaskValue content) {\n");
        out.push_str("    FILE *f = fopen(path.as.string, \"w\");\n");
        out.push_str("    if (!f) return (SnaskValue){.type = SNASK_BOOL, .as.boolean = false};\n");
        out.push_str("    fprintf(f, \"%s\", content.as.string); fclose(f);\n");
        out.push_str("    return (SnaskValue){.type = SNASK_BOOL, .as.boolean = true};\n");
        out.push_str("}\n\n");

        out.push_str("SnaskValue fs_exists(SnaskValue path) { return (SnaskValue){.type = SNASK_BOOL, .as.boolean = (access(path.as.string, F_OK) == 0)}; }\n");
        out.push_str("SnaskValue fs_delete(SnaskValue path) { return (SnaskValue){.type = SNASK_BOOL, .as.boolean = (remove(path.as.string) == 0)}; }\n");
        out.push_str("SnaskValue fs_mkdir(SnaskValue path) { return (SnaskValue){.type = SNASK_BOOL, .as.boolean = (mkdir(path.as.string, 0777) == 0)}; }\n");
        out.push_str("SnaskValue fs_rmdir(SnaskValue path) { return (SnaskValue){.type = SNASK_BOOL, .as.boolean = (rmdir(path.as.string) == 0)}; }\n");
        
        out.push_str("SnaskValue fs_is_dir(SnaskValue path) {\n");
        out.push_str("    struct stat st; stat(path.as.string, &st);\n");
        out.push_str("    return (SnaskValue){.type = SNASK_BOOL, .as.boolean = S_ISDIR(st.st_mode)};\n");
        out.push_str("}\n\n");

        out.push_str("SnaskValue fs_size(SnaskValue path) {\n");
        out.push_str("    struct stat st; stat(path.as.string, &st);\n");
        out.push_str("    return (SnaskValue){.type = SNASK_NUMBER, .as.number = (double)st.st_size};\n");
        out.push_str("}\n\n");

        out.push_str("SnaskValue fs_list(SnaskValue path) {\n");
        out.push_str("    DIR *d = opendir(path.as.string);\n");
        out.push_str("    if (!d) return (SnaskValue){.type = SNASK_NIL};\n");
        out.push_str("    int count = 0; struct dirent *dir; while ((dir = readdir(d)) != NULL) { if(dir->d_name[0] != '.') count++; }\n");
        out.push_str("    rewinddir(d);\n");
        out.push_str("    SnaskValue* items = malloc(sizeof(SnaskValue) * count);\n");
        out.push_str("    int i = 0; while ((dir = readdir(d)) != NULL) {\n");
        out.push_str("        if(dir->d_name[0] == '.') continue;\n");
        out.push_str("        char* s = malloc(strlen(dir->d_name) + 1); strcpy(s, dir->d_name);\n");
        out.push_str("        items[i++] = (SnaskValue){.type = SNASK_STRING, .as.string = s};\n");
        out.push_str("    }\n");
        out.push_str("    closedir(d);\n");
        out.push_str("    return (SnaskValue){.type = SNASK_LIST, .as.list = {.items = items, .length = count}};\n");
        out.push_str("}\n\n");
    }

    fn emit_function_to(&self, out: &mut String, func: FuncDecl) {
        let mut local_vars = HashSet::new();
        self.collect_vars(&func.body, &mut local_vars);
        
        out.push_str(&format!("SnaskValue {}(", func.name));
        for (i, (param_name, _)) in func.params.iter().enumerate() {
            out.push_str(&format!("SnaskValue {}", param_name));
            if i < func.params.len() - 1 { out.push_str(", "); }
        }
        out.push_str(") {\n");
        for name in local_vars {
            out.push_str(&format!("    SnaskValue {};\n", name));
        }
        
        let mut body_str = String::new();
        for stmt in func.body { self.emit_statement_to(&mut body_str, stmt, 1); }
        out.push_str(&body_str);
        out.push_str("    return (SnaskValue){.type = SNASK_NIL};\n}\n\n");
    }

    fn emit_statement_to(&self, out: &mut String, stmt: Stmt, indent_level: usize) {
        let indent = "    ".repeat(indent_level);
        match stmt.kind {
            StmtKind::VarDeclaration(d) => {
                let val = self.emit_expression_val(d.value);
                out.push_str(&format!("{}{} = {};\n", indent, d.name, val));
            }
            StmtKind::MutDeclaration(d) => {
                let val = self.emit_expression_val(d.value);
                out.push_str(&format!("{}{} = {};\n", indent, d.name, val));
            }
            StmtKind::ConstDeclaration(d) => {
                let val = self.emit_expression_val(d.value);
                out.push_str(&format!("{}{} = {};\n", indent, d.name, val));
            }
            StmtKind::VarAssignment(d) => {
                let val = self.emit_expression_val(d.value);
                out.push_str(&format!("{}{} = {};\n", indent, d.name, val));
            }
            StmtKind::Input { name, var_type } => {
                let func = if matches!(var_type, Type::Float | Type::Int) { "input_number()" } else { "input_string()" };
                out.push_str(&format!("{}{} = {};\n", indent, name, func));
            }
            StmtKind::Print(exprs) => {
                for (i, expr) in exprs.iter().enumerate() {
                    let code = self.emit_expression_val(expr.clone());
                    out.push_str(&format!("{}print_value({});\n", indent, code));
                    if i < exprs.len() - 1 { out.push_str(&format!("{}printf(\" \");\n", indent)); }
                }
                out.push_str(&format!("{}printf(\"\\n\");\n", indent));
            }
            StmtKind::Return(expr) => {
                let code = self.emit_expression_val(expr);
                out.push_str(&format!("{}return {};\n", indent, code));
            }
            StmtKind::Conditional(cond) => {
                let cond_code = self.emit_expression_val(cond.if_block.condition);
                out.push_str(&format!("{}if ({}.as.boolean) {{\n", indent, cond_code));
                for s in cond.if_block.body { self.emit_statement_to(out, s, indent_level + 1); }
                out.push_str(&format!("{}}} ", indent));
                for elif in cond.elif_blocks {
                    let elif_cond = self.emit_expression_val(elif.condition);
                    out.push_str(&format!("else if ({}.as.boolean) {{\n", elif_cond));
                    for s in elif.body { self.emit_statement_to(out, s, indent_level + 1); }
                    out.push_str(&format!("{}}} ", indent));
                }
                if let Some(else_body) = cond.else_block {
                    out.push_str("else {\n");
                    for s in else_body { self.emit_statement_to(out, s, indent_level + 1); }
                    out.push_str(&format!("{}}}\n", indent));
                } else { out.push_str("\n"); }
            }
            StmtKind::Loop(loop_stmt) => match loop_stmt {
                LoopStmt::While { condition, body } => {
                    out.push_str(&format!("{}while (1) {{\n", indent));
                    let cond_code = self.emit_expression_val(condition);
                    out.push_str(&format!("{}    if (!({}.as.boolean)) break;\n", indent, cond_code));
                    for s in body {
                        self.emit_statement_to(out, s, indent_level + 1);
                    }
                    out.push_str(&format!("{}}}\n", indent));
                }
                LoopStmt::For { iterator, iterable, body } => {
                    let iter_code = self.emit_expression_val(iterable);
                    let iter_name = format!("iter_{}", iterator);
                    out.push_str(&format!("{}SnaskValue {} = {};\n", indent, iter_name, iter_code));
                    out.push_str(&format!("{}for (int i=0; i < {}.as.list.length; i++) {{\n", indent, iter_name));
                    out.push_str(&format!("{}    {} = {}.as.list.items[i];\n", indent, iterator, iter_name));
                    for s in body {
                        self.emit_statement_to(out, s, indent_level + 1);
                    }
                    out.push_str(&format!("{}}}\n", indent));
                }
            },
            StmtKind::FuncCall(expr) => {
                let code = self.emit_expression_val(expr);
                out.push_str(&format!("{}{};\n", indent, code));
            }
            _ => {}
        }
    }

    fn emit_expression_val(&self, expr: Expr) -> String {
        match expr.kind {
            ExprKind::Literal(lit) => match lit {
                LiteralValue::Number(n) => format!("(SnaskValue){{.type = SNASK_NUMBER, .as.number = {}}}", n),
                LiteralValue::Boolean(b) => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}}}", b),
                LiteralValue::String(s) => format!("(SnaskValue){{.type = SNASK_STRING, .as.string = \"{}\"}}", s),
                LiteralValue::Nil => format!("(SnaskValue){{.type = SNASK_NIL}}"),
                LiteralValue::List(items) => {
                    let mut init = String::from("(SnaskValue[]){");
                    for (i, item) in items.iter().enumerate() {
                        init.push_str(&self.emit_expression_val(item.clone()));
                        if i < items.len() - 1 { init.push_str(", "); }
                    }
                    init.push('}');
                    format!("(SnaskValue){{.type = SNASK_LIST, .as.list = {{.items = {}, .length = {}}}}}", init, items.len())
                }
                _ => "((SnaskValue){0})".to_string(),
            },
            ExprKind::Variable(name) => name,
            ExprKind::Binary { op, left, right } => {
                let l = self.emit_expression_val(*left);
                let r = self.emit_expression_val(*right);
                match op {
                    BinaryOp::Add => format!("(SnaskValue){{.type = SNASK_NUMBER, .as.number = {}.as.number + {}.as.number}}", l, r),
                    BinaryOp::Subtract => format!("(SnaskValue){{.type = SNASK_NUMBER, .as.number = {}.as.number - {}.as.number}}", l, r),
                    BinaryOp::Multiply => format!("(SnaskValue){{.type = SNASK_NUMBER, .as.number = {}.as.number * {}.as.number}}", l, r),
                    BinaryOp::Divide => format!("(SnaskValue){{.type = SNASK_NUMBER, .as.number = {}.as.number / {}.as.number}}", l, r),
                    BinaryOp::Equals => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number == {}.as.number}}", l, r),
                    BinaryOp::NotEquals => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number != {}.as.number}}", l, r),
                    BinaryOp::LessThan => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number < {}.as.number}}", l, r),
                    BinaryOp::GreaterThan => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number > {}.as.number}}", l, r),
                    BinaryOp::LessThanOrEquals => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number <= {}.as.number}}", l, r),
                    BinaryOp::GreaterThanOrEquals => format!("(SnaskValue){{.type = SNASK_BOOL, .as.boolean = {}.as.number >= {}.as.number}}", l, r),
                }
            }
            ExprKind::FunctionCall { callee, args } => {
                if let ExprKind::Variable(name) = callee.kind {
                    if !name.starts_with("__") && is_library_native(name.as_str()) {
                        let lib = library_native_lib(name.as_str());
                        return format!("snask__restricted_native(\"{}\", \"{}\")", name, lib);
                    }
                    let mut call = format!("{}(", name);
                    for (i, arg) in args.iter().enumerate() {
                        call.push_str(&self.emit_expression_val(arg.clone()));
                        if i < args.len() - 1 { call.push_str(", "); }
                    }
                    call.push(')');
                    call
                } else { "((SnaskValue){0})".to_string() }
            }
            _ => "((SnaskValue){0})".to_string(),
        }
    }
}

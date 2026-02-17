use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, PointerValue, StructValue};
use inkwell::types::StructType;
use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, LiteralValue, BinaryOp, FuncDecl};
use std::collections::HashMap;

const TYPE_NIL: u32 = 0;
const TYPE_NUM: u32 = 1;
const TYPE_STR: u32 = 3;

pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, PointerValue<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    value_type: StructType<'ctx>,
    ptr_type: inkwell::types::PointerType<'ctx>,
    current_func: Option<FunctionValue<'ctx>>,
    local_vars: HashMap<String, PointerValue<'ctx>>,
    classes: HashMap<String, crate::ast::ClassDecl>,
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let _i32_type = context.i32_type();
        let f64_type = context.f64_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::from(0));
        let value_type = context.struct_type(&[f64_type.into(), f64_type.into(), ptr_type.into()], false);

        LLVMGenerator { context, module, builder, variables: HashMap::new(), functions: HashMap::new(), value_type, ptr_type, current_func: None, local_vars: HashMap::new(), classes: HashMap::new() }
    }

    pub fn generate(&mut self, program: Program) -> Result<String, String> {
        self.declare_runtime();
        
        // Declara funções globais e preenche mapa de classes
        for stmt in &program { 
            if let StmtKind::FuncDeclaration(func) = &stmt.kind { self.declare_function(func)?; } 
            if let StmtKind::ClassDeclaration(class) = &stmt.kind {
                let mut c = class.clone();
                for method in &mut c.methods {
                    // Adiciona 'self' como primeiro parâmetro se não existir
                    if !method.params.iter().any(|p| p.0 == "self") {
                        method.params.insert(0, ("self".to_string(), crate::types::Type::Any));
                    }
                    let mut m = method.clone();
                    m.name = format!("{}::{}", c.name, m.name);
                    self.declare_function(&m)?;
                }
                self.classes.insert(c.name.clone(), c);
            }
        }

        let i32_type = self.context.i32_type();
        let main_func = self.module.add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry);
        self.current_func = Some(main_func);

        // Se houver uma class main com método start, chama ele!
        let mut has_start = false;
        for stmt in &program {
            if let StmtKind::ClassDeclaration(class) = &stmt.kind {
                if class.name == "main" {
                    for method in &class.methods {
                        if method.name == "start" {
                            let f_name = format!("main::start");
                            if let Some(f) = self.functions.get(&f_name) {
                                let mut l_args = Vec::new();
                                let r_a = self.builder.build_alloca(self.value_type, "ra").unwrap();
                                l_args.push(r_a.into());
                                self.builder.build_call(*f, &l_args, "call_start").unwrap();
                                has_start = true;
                            }
                        }
                    }
                }
            }
        }

        if !has_start {
            // Se não tiver main::start, executa top-level statements
            for stmt in &program { 
                if !matches!(stmt.kind, StmtKind::FuncDeclaration(_) | StmtKind::ClassDeclaration(_)) { 
                    self.generate_statement(stmt.clone())?; 
                } 
            }
        }
        
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_return(Some(&i32_type.const_int(0, false))).unwrap();
        }

        // Gera o corpo das funções
        for stmt in program { 
            if let StmtKind::FuncDeclaration(func) = &stmt.kind { self.generate_function_body(func.clone())?; } 
            if let StmtKind::ClassDeclaration(class) = &stmt.kind {
                // Pega a versão atualizada da classe (com o self injetado)
                let c = self.classes.get(&class.name).unwrap().clone();
                for mut method in c.methods {
                    method.name = format!("{}::{}", c.name, method.name);
                    self.generate_function_body(method)?;
                }
            }
        }
        Ok(self.module.print_to_string().to_string())
    }

    fn declare_runtime(&mut self) {
        let _i32_type = self.context.i32_type();
        let void_type = self.context.void_type();
        
        // Novas funções de print
        self.module.add_function("s_print", void_type.fn_type(&[self.ptr_type.into()], false), None);
        self.module.add_function("s_println", void_type.fn_type(&[], false), None);

        let fn_1 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false);
        let fn_2 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into()], false);
        let fn_3 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into()], false);
        let fn_alloc = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into()], false);
        
        self.functions.insert("sfs_read".to_string(), self.module.add_function("sfs_read", fn_1, None));
        self.functions.insert("sfs_write".to_string(), self.module.add_function("sfs_write", fn_2, None));
        self.functions.insert("sfs_append".to_string(), self.module.add_function("sfs_append", fn_2, None));
        self.functions.insert("sfs_delete".to_string(), self.module.add_function("sfs_delete", fn_1, None));
        self.functions.insert("sfs_exists".to_string(), self.module.add_function("sfs_exists", fn_1, None));
        self.functions.insert("sfs_copy".to_string(), self.module.add_function("sfs_copy", fn_2, None));
        self.functions.insert("sfs_move".to_string(), self.module.add_function("sfs_move", fn_2, None));
        self.functions.insert("sfs_mkdir".to_string(), self.module.add_function("sfs_mkdir", fn_1, None));
        self.functions.insert("sfs_is_file".to_string(), self.module.add_function("sfs_is_file", fn_1, None));
        self.functions.insert("sfs_is_dir".to_string(), self.module.add_function("sfs_is_dir", fn_1, None));
        self.functions.insert("sfs_listdir".to_string(), self.module.add_function("sfs_listdir", fn_1, None));
        self.functions.insert("s_http_get".to_string(), self.module.add_function("s_http_get", fn_1, None));
        self.functions.insert("s_http_post".to_string(), self.module.add_function("s_http_post", fn_2, None));
        self.functions.insert("s_http_put".to_string(), self.module.add_function("s_http_put", fn_2, None));
        self.functions.insert("s_http_delete".to_string(), self.module.add_function("s_http_delete", fn_1, None));
        self.functions.insert("s_http_patch".to_string(), self.module.add_function("s_http_patch", fn_2, None));
        let f_concat = self.module.add_function("s_concat", fn_2, None);
        self.functions.insert("s_concat".to_string(), f_concat);
        self.functions.insert("concat".to_string(), f_concat);

        let f_abs = self.module.add_function("s_abs", fn_1, None);
        self.functions.insert("s_abs".to_string(), f_abs);
        self.functions.insert("abs".to_string(), f_abs);

        let f_max = self.module.add_function("s_max", fn_2, None);
        self.functions.insert("s_max".to_string(), f_max);
        self.functions.insert("max".to_string(), f_max);

        let f_min = self.module.add_function("s_min", fn_2, None);
        self.functions.insert("s_min".to_string(), f_min);
        self.functions.insert("min".to_string(), f_min);

        let f_len = self.module.add_function("s_len", fn_1, None);
        self.functions.insert("s_len".to_string(), f_len);
        self.functions.insert("len".to_string(), f_len);

        let f_upper = self.module.add_function("s_upper", fn_1, None);
        self.functions.insert("s_upper".to_string(), f_upper);
        self.functions.insert("upper".to_string(), f_upper);

        let f_time = self.module.add_function("s_time", void_type.fn_type(&[self.ptr_type.into()], false), None);
        self.functions.insert("s_time".to_string(), f_time);
        self.functions.insert("time".to_string(), f_time);

        let f_sleep = self.module.add_function("s_sleep", fn_1, None);
        self.functions.insert("s_sleep".to_string(), f_sleep);
        self.functions.insert("sleep".to_string(), f_sleep);

        let f_exit = self.module.add_function("s_exit", fn_1, None);
        self.functions.insert("exit".to_string(), f_exit);
        self.functions.insert("is_nil".to_string(), self.module.add_function("is_nil", fn_1, None));
        self.functions.insert("is_str".to_string(), self.module.add_function("is_str", fn_1, None));
        self.functions.insert("is_obj".to_string(), self.module.add_function("is_obj", fn_1, None));
        self.functions.insert("os_cwd".to_string(), self.module.add_function("os_cwd", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("os_platform".to_string(), self.module.add_function("os_platform", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("os_arch".to_string(), self.module.add_function("os_arch", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("os_getenv".to_string(), self.module.add_function("os_getenv", fn_1, None));
        self.functions.insert("os_setenv".to_string(), self.module.add_function("os_setenv", fn_2, None));
        self.functions.insert("os_random_hex".to_string(), self.module.add_function("os_random_hex", fn_1, None));

        self.functions.insert("sfs_size".to_string(), self.module.add_function("sfs_size", fn_1, None));
        self.functions.insert("sfs_mtime".to_string(), self.module.add_function("sfs_mtime", fn_1, None));
        self.functions.insert("sfs_rmdir".to_string(), self.module.add_function("sfs_rmdir", fn_1, None));

        self.functions.insert("path_basename".to_string(), self.module.add_function("path_basename", fn_1, None));
        self.functions.insert("path_dirname".to_string(), self.module.add_function("path_dirname", fn_1, None));
        self.functions.insert("path_extname".to_string(), self.module.add_function("path_extname", fn_1, None));
        self.functions.insert("path_join".to_string(), self.module.add_function("path_join", fn_2, None));
        self.functions.insert("blaze_run".to_string(), self.module.add_function("blaze_run", fn_2, None));
        self.functions.insert("blaze_qs_get".to_string(), self.module.add_function("blaze_qs_get", fn_2, None));
        self.functions.insert("blaze_cookie_get".to_string(), self.module.add_function("blaze_cookie_get", fn_2, None));
        self.functions.insert("auth_random_hex".to_string(), self.module.add_function("auth_random_hex", fn_1, None));
        self.functions.insert("auth_now".to_string(), self.module.add_function("auth_now", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("auth_const_time_eq".to_string(), self.module.add_function("auth_const_time_eq", fn_2, None));
        self.functions.insert("auth_hash_password".to_string(), self.module.add_function("auth_hash_password", fn_1, None));
        self.functions.insert("auth_verify_password".to_string(), self.module.add_function("auth_verify_password", fn_2, None));
        self.functions.insert("auth_session_id".to_string(), self.module.add_function("auth_session_id", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("auth_csrf_token".to_string(), self.module.add_function("auth_csrf_token", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("auth_cookie_kv".to_string(), self.module.add_function("auth_cookie_kv", fn_2, None));
        self.functions.insert("auth_cookie_session".to_string(), self.module.add_function("auth_cookie_session", fn_1, None));
        self.functions.insert("auth_cookie_delete".to_string(), self.module.add_function("auth_cookie_delete", fn_1, None));
        self.functions.insert("auth_bearer_header".to_string(), self.module.add_function("auth_bearer_header", fn_1, None));
        self.functions.insert("auth_ok".to_string(), self.module.add_function("auth_ok", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("auth_fail".to_string(), self.module.add_function("auth_fail", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("auth_version".to_string(), self.module.add_function("auth_version", void_type.fn_type(&[self.ptr_type.into()], false), None));

        // GUI (GTK) - handles are strings
        self.functions.insert("gui_init".to_string(), self.module.add_function("gui_init", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_run".to_string(), self.module.add_function("gui_run", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_quit".to_string(), self.module.add_function("gui_quit", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_window".to_string(), self.module.add_function("gui_window", fn_3, None));
        self.functions.insert("gui_vbox".to_string(), self.module.add_function("gui_vbox", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_hbox".to_string(), self.module.add_function("gui_hbox", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_set_child".to_string(), self.module.add_function("gui_set_child", fn_2, None));
        self.functions.insert("gui_add".to_string(), self.module.add_function("gui_add", fn_2, None));
        self.functions.insert("gui_label".to_string(), self.module.add_function("gui_label", fn_1, None));
        self.functions.insert("gui_entry".to_string(), self.module.add_function("gui_entry", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("gui_button".to_string(), self.module.add_function("gui_button", fn_1, None));
        self.functions.insert("gui_show_all".to_string(), self.module.add_function("gui_show_all", fn_1, None));
        self.functions.insert("gui_set_text".to_string(), self.module.add_function("gui_set_text", fn_2, None));
        self.functions.insert("gui_get_text".to_string(), self.module.add_function("gui_get_text", fn_1, None));
        self.functions.insert("gui_on_click".to_string(), self.module.add_function("gui_on_click", fn_2, None));
        self.functions.insert("gui_on_click_ctx".to_string(), self.module.add_function("gui_on_click_ctx", fn_3, None));

        self.functions.insert("str_to_num".to_string(), self.module.add_function("str_to_num", fn_1, None));
        self.functions.insert("num_to_str".to_string(), self.module.add_function("num_to_str", fn_1, None));
        self.functions.insert("calc_eval".to_string(), self.module.add_function("calc_eval", fn_1, None));

        self.functions.insert("s_alloc_obj".to_string(), self.module.add_function("s_alloc_obj", fn_alloc, None));
        self.functions.insert("s_json_stringify".to_string(), self.module.add_function("s_json_stringify", fn_1, None));
        self.functions.insert("json_stringify".to_string(), self.module.add_function("json_stringify", fn_1, None));
        self.functions.insert("json_stringify_pretty".to_string(), self.module.add_function("json_stringify_pretty", fn_1, None));
        self.functions.insert("json_parse".to_string(), self.module.add_function("json_parse", fn_1, None));
        self.functions.insert("json_get".to_string(), self.module.add_function("json_get", fn_2, None));
        self.functions.insert("json_has".to_string(), self.module.add_function("json_has", fn_2, None));
        self.functions.insert("json_len".to_string(), self.module.add_function("json_len", fn_1, None));
        self.functions.insert("json_index".to_string(), self.module.add_function("json_index", fn_2, None));
        self.functions.insert("json_set".to_string(), self.module.add_function("json_set", fn_3, None));
        self.functions.insert("sjson_new_object".to_string(), self.module.add_function("sjson_new_object", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("sjson_new_array".to_string(), self.module.add_function("sjson_new_array", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("sjson_type".to_string(), self.module.add_function("sjson_type", fn_1, None));
        self.functions.insert("sjson_arr_len".to_string(), self.module.add_function("sjson_arr_len", fn_1, None));
        self.functions.insert("sjson_arr_get".to_string(), self.module.add_function("sjson_arr_get", fn_2, None));
        self.functions.insert("sjson_arr_set".to_string(), self.module.add_function("sjson_arr_set", fn_3, None));
        self.functions.insert("sjson_arr_push".to_string(), self.module.add_function("sjson_arr_push", fn_2, None));
        self.functions.insert("sjson_path_get".to_string(), self.module.add_function("sjson_path_get", fn_2, None));
        self.functions.insert("json_parse_ex".to_string(), self.module.add_function("json_parse_ex", fn_1, None));
        self.functions.insert("s_get_member".to_string(), self.module.add_function("s_get_member", fn_2, None));
        self.functions.insert("s_set_member".to_string(), self.module.add_function("s_set_member", void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into()], false), None));
    }

    fn sanitize_name(&self, name: &str) -> String {
        name.replace("::", "_NS_")
    }

    fn declare_function(&mut self, func: &FuncDecl) -> Result<(), String> {
        let mut p_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![self.ptr_type.into()];
        for _ in &func.params { p_types.push(self.ptr_type.into()); }
        let f_name = format!("f_{}", self.sanitize_name(&func.name));
        let function = self.module.add_function(&f_name, self.context.void_type().fn_type(&p_types, false), None);
        self.functions.insert(func.name.clone(), function);
        Ok(())
    }

    fn generate_function_body(&mut self, func: FuncDecl) -> Result<(), String> {
        let function = *self.functions.get(&func.name).unwrap();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.current_func = Some(function);
        let old_vars = self.variables.clone();
        self.local_vars.clear();
        let r_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
        
        // Parâmetros começam no índice 1, porque o 0 é o RA (Return Address/Pointer)
        for (i, (name, _)) in func.params.iter().enumerate() {
            let p_ptr = function.get_nth_param((i + 1) as u32).unwrap().into_pointer_value();
            self.local_vars.insert(name.clone(), p_ptr);
        }
        for stmt in func.body { self.generate_statement(stmt)?; }
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            let mut s = self.value_type.get_undef();
            s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_NIL as f64), 0, "t").unwrap().into_struct_value();
            self.builder.build_store(r_ptr, s).unwrap();
            self.builder.build_return(None).unwrap();
        }
        self.local_vars.clear();
        self.variables = old_vars;
        Ok(())
    }

    fn generate_statement(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt.kind {
            StmtKind::VarDeclaration(d) => {
                let v = self.evaluate_expression(d.value)?;
                let a = self.builder.build_alloca(self.value_type, &d.name).unwrap();
                self.builder.build_store(a, v).unwrap();
                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" { self.variables.insert(d.name, a); }
                else { self.local_vars.insert(d.name, a); }
            }
            StmtKind::MutDeclaration(d) => {
                let v = self.evaluate_expression(d.value)?;
                let a = self.builder.build_alloca(self.value_type, &d.name).unwrap();
                self.builder.build_store(a, v).unwrap();
                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" { self.variables.insert(d.name, a); }
                else { self.local_vars.insert(d.name, a); }
            }
            StmtKind::ConstDeclaration(d) => {
                let v = self.evaluate_expression(d.value)?;
                let a = self.builder.build_alloca(self.value_type, &d.name).unwrap();
                self.builder.build_store(a, v).unwrap();
                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" { self.variables.insert(d.name, a); }
                else { self.local_vars.insert(d.name, a); }
            }
            StmtKind::VarAssignment(s) => {
                let v = self.evaluate_expression(s.value)?;
                let p = self.local_vars.get(&s.name).or_else(|| self.variables.get(&s.name)).ok_or_else(|| format!("Var {} not found.", s.name))?;
                self.builder.build_store(*p, v).unwrap();
            }
            StmtKind::Print(exprs) => {
                let p_func = self.module.get_function("s_print").unwrap();
                let nl_func = self.module.get_function("s_println").unwrap();
                for expr in exprs {
                    let v = self.evaluate_expression(expr)?;
                    let v_ptr = self.builder.build_alloca(self.value_type, "pv").unwrap();
                    self.builder.build_store(v_ptr, v).unwrap();
                    self.builder.build_call(p_func, &[v_ptr.into()], "c").unwrap();
                }
                self.builder.build_call(nl_func, &[], "nl").unwrap();
            }
            StmtKind::Return(expr) => {
                let _v = self.evaluate_expression(expr)?;
                if self.current_func.unwrap().get_name().to_str().unwrap() != "main" {
                    let op = self.current_func.unwrap().get_nth_param(0).unwrap().into_pointer_value();
                    self.builder.build_store(op, _v).unwrap();
                    self.builder.build_return(None).unwrap();
                } else { 
                    let i32_type = self.context.i32_type();
                    self.builder.build_return(Some(&i32_type.const_int(0, false))).unwrap(); 
                }
            }
            StmtKind::Conditional(c) => {
                let parent = self.current_func.unwrap();
                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let merge_bb = self.context.append_basic_block(parent, "merge");

                let cond_val = self.evaluate_expression(c.if_block.condition)?;
                let n = self.builder.build_extract_value(cond_val, 1, "n").unwrap().into_float_value();
                let is_true = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, n, self.context.f64_type().const_float(0.0), "is_true").unwrap();
                
                self.builder.build_conditional_branch(is_true, then_bb, else_bb).unwrap();

                // THEN block
                self.builder.position_at_end(then_bb);
                for s in c.if_block.body { self.generate_statement(s)?; }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                // ELSE block
                self.builder.position_at_end(else_bb);
                if let Some(else_body) = c.else_block {
                    for s in else_body { self.generate_statement(s)?; }
                }
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
            }
            StmtKind::FuncCall(expr) => {
                self.evaluate_expression(expr)?;
            }
            StmtKind::Expression(expr) => {
                self.evaluate_expression(expr)?;
            }
            StmtKind::ClassDeclaration(_) => {
                // TODO: Implement LLVM class generation
            }
            _ => {}
        }
        Ok(())
    }

    fn evaluate_expression(&self, expr: Expr) -> Result<StructValue<'ctx>, String> {
        let nil_p = self.ptr_type.const_null();
        match expr.kind {
            ExprKind::Literal(lit) => match lit {
                LiteralValue::Number(n) => {
                    let mut s = self.value_type.get_undef();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(n), 1, "v").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, nil_p, 2, "p").unwrap().into_struct_value();
                    Ok(s)
                },
                LiteralValue::String(str_v) => {
                    let g = self.builder.build_global_string_ptr(&str_v, "s").unwrap();
                    let mut s = self.value_type.get_undef();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_STR as f64), 0, "t").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(0.0), 1, "v").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, g.as_pointer_value(), 2, "p").unwrap().into_struct_value();
                    Ok(s)
                },
                LiteralValue::Boolean(b) => {
                    let mut s = self.value_type.get_undef();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(2.0), 0, "t").unwrap().into_struct_value(); // 2 = BOOL
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(if b { 1.0 } else { 0.0 }), 1, "v").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, nil_p, 2, "p").unwrap().into_struct_value();
                    Ok(s)
                },
                LiteralValue::Nil => {
                    let mut s = self.value_type.get_undef();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(0.0), 0, "t").unwrap().into_struct_value(); // 0 = NIL
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(0.0), 1, "v").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, nil_p, 2, "p").unwrap().into_struct_value();
                    Ok(s)
                },
                _ => Err(format!("Lit not supported: {:?}", lit)),
            },
            ExprKind::Variable(name) => {
                if let Some(p) = self.local_vars.get(&name).or_else(|| self.variables.get(&name)) { 
                    return Ok(self.builder.build_load(self.value_type, *p, &name).unwrap().into_struct_value()); 
                }
                Err(format!("Var {} not found.", name))
            }
            ExprKind::Binary { op, left, right } => {
                let lhs = self.evaluate_expression(*left)?;
                let rhs = self.evaluate_expression(*right)?;
                let lt_f = self.builder.build_extract_value(lhs, 0, "lt").unwrap().into_float_value();
                let rt_f = self.builder.build_extract_value(rhs, 0, "rt").unwrap().into_float_value();
                let lt = self.builder.build_float_to_unsigned_int(lt_f, self.context.i32_type(), "lt_i").unwrap();
                let rt = self.builder.build_float_to_unsigned_int(rt_f, self.context.i32_type(), "rt_i").unwrap();
                if matches!(op, BinaryOp::Add) {
                    let is_l_s = self.builder.build_int_compare(inkwell::IntPredicate::EQ, lt, self.context.i32_type().const_int(TYPE_STR as u64, false), "ils").unwrap();
                    let is_r_s = self.builder.build_int_compare(inkwell::IntPredicate::EQ, rt, self.context.i32_type().const_int(TYPE_STR as u64, false), "irs").unwrap();
                    let is_any_s = self.builder.build_or(is_l_s, is_r_s, "ias").unwrap();
                    let parent = self.current_func.unwrap();
                    let cat_bb = self.context.append_basic_block(parent, "cat");
                    let add_bb = self.context.append_basic_block(parent, "add");
                    let m_bb = self.context.append_basic_block(parent, "m");
                    let res_p = self.builder.build_alloca(self.value_type, "rp").unwrap();
                    self.builder.build_conditional_branch(is_any_s, cat_bb, add_bb).unwrap();
                    self.builder.position_at_end(cat_bb);
                    let f = self.module.get_function("s_concat").unwrap();
                    let lp = self.builder.build_alloca(self.value_type, "lp").unwrap();
                    let rp = self.builder.build_alloca(self.value_type, "rp").unwrap();
                    self.builder.build_store(lp, lhs).unwrap(); self.builder.build_store(rp, rhs).unwrap();
                    self.builder.build_call(f, &[res_p.into(), lp.into(), rp.into()], "c").unwrap();
                    self.builder.build_unconditional_branch(m_bb).unwrap();
                    self.builder.position_at_end(add_bb);
                    let lv = self.builder.build_extract_value(lhs, 1, "lv").unwrap().into_float_value();
                    let rv = self.builder.build_extract_value(rhs, 1, "rv").unwrap().into_float_value();
                    let res_v = self.builder.build_float_add(lv, rv, "v").unwrap();
                    let mut s = self.value_type.get_undef();
                    s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                    s = self.builder.build_insert_value(s, res_v, 1, "v").unwrap().into_struct_value();
                    self.builder.build_store(res_p, s).unwrap();
                    self.builder.build_unconditional_branch(m_bb).unwrap();
                    self.builder.position_at_end(m_bb);
                    return Ok(self.builder.build_load(self.value_type, res_p, "f").unwrap().into_struct_value());
                }
                let lv = self.builder.build_extract_value(lhs, 1, "lv").unwrap().into_float_value();
                let rv = self.builder.build_extract_value(rhs, 1, "rv").unwrap().into_float_value();
                let res = match op {
                    BinaryOp::Subtract => self.builder.build_float_sub(lv, rv, "s").unwrap(),
                    BinaryOp::Multiply => self.builder.build_float_mul(lv, rv, "m").unwrap(),
                    BinaryOp::Divide => self.builder.build_float_div(lv, rv, "d").unwrap(),
                    BinaryOp::And => {
                        let lz = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lv, self.context.f64_type().const_float(0.0), "lz").unwrap();
                        let rz = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, rv, self.context.f64_type().const_float(0.0), "rz").unwrap();
                        let both = self.builder.build_and(lz, rz, "and").unwrap();
                        self.builder.build_unsigned_int_to_float(both, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::Or => {
                        let lz = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, lv, self.context.f64_type().const_float(0.0), "lz").unwrap();
                        let rz = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, rv, self.context.f64_type().const_float(0.0), "rz").unwrap();
                        let any = self.builder.build_or(lz, rz, "or").unwrap();
                        self.builder.build_unsigned_int_to_float(any, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::LessThan => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OLT, lv, rv, "lt").unwrap();
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::GreaterThan => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OGT, lv, rv, "gt").unwrap();
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::GreaterThanOrEquals => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OGE, lv, rv, "ge").unwrap();
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::LessThanOrEquals => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OLE, lv, rv, "le").unwrap();
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "bf").unwrap()
                    }
                    BinaryOp::Equals => {
                        let cmp = self.builder.build_float_compare(inkwell::FloatPredicate::OEQ, lv, rv, "eq").unwrap();
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "bf").unwrap()
                    }
                    _ => self.context.f64_type().const_float(0.0),
                };
                let mut s = self.value_type.get_undef();
                s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                s = self.builder.build_insert_value(s, res, 1, "v").unwrap().into_struct_value();
                Ok(s)
            }
            ExprKind::PropertyAccess { target, property } => {
                let obj = self.evaluate_expression(*target)?;
                // Por simplicidade, assumimos que o alvo é um objeto e procuramos o índice da propriedade.
                // Em um compilador completo, precisaríamos saber o tipo real do objeto.
                // Como não temos um sistema de tipos forte no backend ainda, vamos inferir ou fixar.
                
                // Vamos tentar achar a propriedade em qualquer classe (hack temporário)
                let mut index = -1;
                for class in self.classes.values() {
                    if let Some(i) = class.properties.iter().position(|p| p.name == property) {
                        index = i as i32;
                        break;
                    }
                }

                if index == -1 { return Err(format!("Propriedade '{}' não encontrada em nenhuma classe.", property)); }

                let get_f = self.functions.get("s_get_member").unwrap();
                let idx_val = self.context.f64_type().const_float(index as f64);
                let mut idx_struct = self.value_type.get_undef();
                idx_struct = self.builder.build_insert_value(idx_struct, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                idx_struct = self.builder.build_insert_value(idx_struct, idx_val, 1, "v").unwrap().into_struct_value();
                
                let obj_p = self.builder.build_alloca(self.value_type, "objp").unwrap();
                self.builder.build_store(obj_p, obj).unwrap();
                let idx_p = self.builder.build_alloca(self.value_type, "idxp").unwrap();
                self.builder.build_store(idx_p, idx_struct).unwrap();
                
                let res_p = self.builder.build_alloca(self.value_type, "rp").unwrap();
                self.builder.build_call(*get_f, &[res_p.into(), obj_p.into(), idx_p.into()], "get").unwrap();
                
                Ok(self.builder.build_load(self.value_type, res_p, "r").unwrap().into_struct_value())
            }
            ExprKind::FunctionCall { callee, args } => {
                if let ExprKind::PropertyAccess { target, property } = &callee.kind {
                    // É uma chamada de método! obj.metodo()
                    let obj = self.evaluate_expression(*target.clone())?;
                    
                    // Procura o método em qualquer classe (hack temporário até termos tipos fortes)
                    let mut method_full_name = String::new();
                    for class in self.classes.values() {
                        if class.methods.iter().any(|m| m.name == *property) {
                            method_full_name = format!("{}::{}", class.name, property);
                            break;
                        }
                    }

                    if method_full_name.is_empty() { return Err(format!("Método '{}' não encontrado.", property)); }

                    let f_name = format!("f_{}", self.sanitize_name(&method_full_name));
                    let f = self.module.get_function(&f_name).ok_or_else(|| format!("Função {} não encontrada.", f_name))?;
                    
                    let mut l_args = Vec::new();
                    let r_a = self.builder.build_alloca(self.value_type, "ra").unwrap();
                    l_args.push(r_a.into());
                    
                    // Adiciona o 'self' como primeiro argumento
                    let obj_p = self.builder.build_alloca(self.value_type, "selfp").unwrap();
                    self.builder.build_store(obj_p, obj).unwrap();
                    l_args.push(obj_p.into());

                    // Adiciona os outros argumentos
                    for arg in args {
                        let v = self.evaluate_expression(arg.clone())?;
                        let arg_a = self.builder.build_alloca(self.value_type, "a").unwrap();
                        self.builder.build_store(arg_a, v).unwrap();
                        l_args.push(arg_a.into());
                    }

                    self.builder.build_call(f, &l_args, "mc").unwrap();
                    return Ok(self.builder.build_load(self.value_type, r_a, "r").unwrap().into_struct_value());
                }

                if let ExprKind::Variable(name) = &callee.kind {
                    if let Some(class) = self.classes.get(name).cloned() {
                        // É uma instanciação de classe!
                        let alloc_f = self.functions.get("s_alloc_obj").unwrap();
                        let size_val = self.context.f64_type().const_float(class.properties.len() as f64);
                        let mut size_struct = self.value_type.get_undef();
                        size_struct = self.builder.build_insert_value(size_struct, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                        size_struct = self.builder.build_insert_value(size_struct, size_val, 1, "v").unwrap().into_struct_value();
                        
                        let size_ptr = self.builder.build_alloca(self.value_type, "szp").unwrap();
                        self.builder.build_store(size_ptr, size_struct).unwrap();
                        
                        let res_p = self.builder.build_alloca(self.value_type, "objp").unwrap();
                        // Passa também a tabela de nomes das propriedades (char**), usada por JSON/stringify e introspecção.
                        let names_arg = if class.properties.is_empty() {
                            self.ptr_type.const_null()
                        } else {
                            let global_name = format!("__snask_class_names_{}", self.sanitize_name(&class.name));
                            let names_global = if let Some(g) = self.module.get_global(&global_name) {
                                g
                            } else {
                                let i8_ptr = self.context.i8_type().ptr_type(inkwell::AddressSpace::from(0));
                                let arr_ty = i8_ptr.array_type(class.properties.len() as u32);
                                let g = self.module.add_global(arr_ty, None, &global_name);
                                let mut elems = Vec::new();
                                for prop in &class.properties {
                                    let sp = self.builder.build_global_string_ptr(&prop.name, "prop_name").unwrap();
                                    elems.push(sp.as_pointer_value());
                                }
                                let init = i8_ptr.const_array(&elems);
                                g.set_initializer(&init);
                                g.set_constant(true);
                                g
                            };
                            let arr_ty = names_global.get_value_type().into_array_type();
                            let zero = self.context.i32_type().const_int(0, false);
                            let names_ptr = unsafe {
                                self.builder
                                    .build_in_bounds_gep(arr_ty, names_global.as_pointer_value(), &[zero, zero], "names_ptr")
                                    .unwrap()
                            };
                            self.builder
                                .build_pointer_cast(names_ptr, self.ptr_type, "names_voidp")
                                .unwrap()
                        };

                        self.builder
                            .build_call(*alloc_f, &[res_p.into(), size_ptr.into(), names_arg.into()], "alloc")
                            .unwrap();
                        
                        let obj = self.builder.build_load(self.value_type, res_p, "obj").unwrap().into_struct_value();
                        
                        // Inicializa propriedades com valores padrão
                        for (i, prop) in class.properties.iter().enumerate() {
                            let val = self.evaluate_expression(prop.value.clone())?;
                            let set_f = self.functions.get("s_set_member").unwrap();
                            let idx_val = self.context.f64_type().const_float(i as f64);
                            let mut idx_struct = self.value_type.get_undef();
                            idx_struct = self.builder.build_insert_value(idx_struct, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                            idx_struct = self.builder.build_insert_value(idx_struct, idx_val, 1, "v").unwrap().into_struct_value();
                            
                            let ip = self.builder.build_alloca(self.value_type, "ip").unwrap();
                            let vp = self.builder.build_alloca(self.value_type, "vp").unwrap();
                            self.builder.build_store(ip, idx_struct).unwrap();
                            self.builder.build_store(vp, val).unwrap();
                            self.builder.build_call(*set_f, &[res_p.into(), ip.into(), vp.into()], "set").unwrap();
                        }
                        
                        return Ok(obj);
                    }

                    let s_name = self.sanitize_name(name);
                    let f_name = format!("f_{}", s_name);
                    // Ordem de busca: f_nome (usuário/lib), depois nome (nativas do runtime)
                    let f = self.module.get_function(&f_name)
                        .or_else(|| self.module.get_function(&s_name))
                        .or_else(|| self.module.get_function(name))
                        .or_else(|| self.functions.get(name).cloned())
                        .ok_or_else(|| format!("Função {} não encontrada.", name))?;
                    let mut l_args = Vec::new();
                    let r_a = self.builder.build_alloca(self.value_type, "ra").unwrap();
                    l_args.push(r_a.into());
                    for arg in args {
                        let v = self.evaluate_expression(arg.clone())?;
                        let arg_a = self.builder.build_alloca(self.value_type, "a").unwrap();
                        self.builder.build_store(arg_a, v).unwrap();
                        l_args.push(arg_a.into());
                    }
                    let _call_site = self.builder.build_call(f, &l_args, "c").unwrap();
                    return Ok(self.builder.build_load(self.value_type, r_a, "r").unwrap().into_struct_value());
                } else { Err("Indirect not supported.".to_string()) }
            }
            ExprKind::Unary { op, expr } => {
                let v = self.evaluate_expression(*expr)?;
                let n = self.builder.build_extract_value(v, 1, "n").unwrap().into_float_value();
                let res_n = match op {
                    crate::ast::UnaryOp::Negative => self.builder.build_float_mul(n, self.context.f64_type().const_float(-1.0), "neg").unwrap(),
                    crate::ast::UnaryOp::Not => {
                        let is_true = self.builder.build_float_compare(inkwell::FloatPredicate::ONE, n, self.context.f64_type().const_float(0.0), "is_true").unwrap();
                        let is_false = self.builder.build_not(is_true, "not").unwrap();
                        self.builder.build_unsigned_int_to_float(is_false, self.context.f64_type(), "bf").unwrap()
                    }
                };
                let mut s = self.value_type.get_undef();
                s = self.builder.build_insert_value(s, self.context.f64_type().const_float(TYPE_NUM as f64), 0, "t").unwrap().into_struct_value();
                s = self.builder.build_insert_value(s, res_n, 1, "v").unwrap().into_struct_value();
                s = self.builder.build_insert_value(s, nil_p, 2, "p").unwrap().into_struct_value();
                Ok(s)
            }
            _ => Err(format!("Expr not supported: {:?}", expr.kind)),
        }
    }

    pub fn emit_to_file(&self, path: &str) -> Result<(), String> {
        self.module.print_to_file(std::path::Path::new(path)).map_err(|e| e.to_string())
    }
}

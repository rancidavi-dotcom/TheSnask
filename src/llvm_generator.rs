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
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let f64_type = context.f64_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::from(0));
        let value_type = context.struct_type(&[f64_type.into(), f64_type.into(), ptr_type.into()], false);

        LLVMGenerator { context, module, builder, variables: HashMap::new(), functions: HashMap::new(), value_type, ptr_type, current_func: None, local_vars: HashMap::new() }
    }

    pub fn generate(&mut self, program: Program) -> Result<String, String> {
        self.declare_runtime();
        for stmt in &program { if let StmtKind::FuncDeclaration(func) = &stmt.kind { self.declare_function(func)?; } }
        let i32_type = self.context.i32_type();
        let main_func = self.module.add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry);
        self.current_func = Some(main_func);
        for stmt in &program { if !matches!(stmt.kind, StmtKind::FuncDeclaration(_)) { self.generate_statement(stmt.clone())?; } }
        
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_return(Some(&i32_type.const_int(0, false))).unwrap();
        }

        for stmt in program { if let StmtKind::FuncDeclaration(func) = stmt.kind { self.generate_function_body(func)?; } }
        Ok(self.module.print_to_string().to_string())
    }

    fn declare_runtime(&mut self) {
        let i32_type = self.context.i32_type();
        let void_type = self.context.void_type();
        
        // Novas funções de print
        self.module.add_function("s_print", void_type.fn_type(&[self.ptr_type.into()], false), None);
        self.module.add_function("s_println", void_type.fn_type(&[], false), None);

        let fn_1 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false);
        let fn_2 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into(), self.ptr_type.into()], false);
        
        self.functions.insert("sfs_read".to_string(), self.module.add_function("sfs_read", fn_1, None));
        self.functions.insert("sfs_write".to_string(), self.module.add_function("sfs_write", fn_2, None));
        self.functions.insert("sfs_delete".to_string(), self.module.add_function("sfs_delete", fn_1, None));
        self.functions.insert("sfs_exists".to_string(), self.module.add_function("sfs_exists", fn_1, None));
        self.functions.insert("s_http_get".to_string(), self.module.add_function("s_http_get", fn_1, None));
        self.functions.insert("s_http_post".to_string(), self.module.add_function("s_http_post", fn_2, None));
        self.functions.insert("s_http_put".to_string(), self.module.add_function("s_http_put", fn_2, None));
        self.functions.insert("s_http_delete".to_string(), self.module.add_function("s_http_delete", fn_1, None));
        self.functions.insert("s_http_patch".to_string(), self.module.add_function("s_http_patch", fn_2, None));
        self.functions.insert("s_concat".to_string(), self.module.add_function("s_concat", fn_2, None));
        self.functions.insert("s_abs".to_string(), self.module.add_function("s_abs", fn_1, None));
        self.functions.insert("s_max".to_string(), self.module.add_function("s_max", fn_2, None));
        self.functions.insert("s_min".to_string(), self.module.add_function("s_min", fn_2, None));
        self.functions.insert("s_len".to_string(), self.module.add_function("s_len", fn_1, None));
        self.functions.insert("s_upper".to_string(), self.module.add_function("s_upper", fn_1, None));
        self.functions.insert("s_time".to_string(), self.module.add_function("s_time", void_type.fn_type(&[self.ptr_type.into()], false), None));
        self.functions.insert("s_sleep".to_string(), self.module.add_function("s_sleep", fn_1, None));
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
                _ => Err("Lit not supported.".to_string()),
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
            ExprKind::FunctionCall { callee, args } => {
                if let ExprKind::Variable(name) = callee.kind {
                    let s_name = self.sanitize_name(&name);
                    let f_name = format!("f_{}", s_name);
                    // Ordem de busca: f_nome (usuário/lib), depois nome (nativas do runtime)
                    let f = self.module.get_function(&f_name)
                        .or_else(|| self.module.get_function(&s_name))
                        .or_else(|| self.module.get_function(&name))
                        .or_else(|| self.functions.get(&name).cloned())
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
                    let call_site = self.builder.build_call(f, &l_args, "c").unwrap();
                    return Ok(self.builder.build_load(self.value_type, r_a, "r").unwrap().into_struct_value());
                } else { Err("Indirect not supported.".to_string()) }
            }
            ExprKind::Unary { op, expr } => {
                let v = self.evaluate_expression(*expr)?;
                let n = self.builder.build_extract_value(v, 1, "n").unwrap().into_float_value();
                let res_n = match op {
                    crate::ast::UnaryOp::Negative => self.builder.build_float_mul(n, self.context.f64_type().const_float(-1.0), "neg").unwrap(),
                    _ => n,
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

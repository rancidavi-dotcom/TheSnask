use crate::ast::{
    BinaryOp, Expr, ExprKind, FuncDecl, LiteralValue, Program, Stmt, StmtKind, VarDecl,
};
use crate::om_contract::{OmContract, OmFunctionContract, OmResourceContract};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, IntType, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue, StructValue,
};
use std::collections::HashMap;

const TYPE_NIL: u32 = 0;
const TYPE_NUM: u32 = 1;
const TYPE_BOOL: u32 = 2;
const TYPE_STR: u32 = 3;
const TYPE_OBJ: u32 = 4;
const TYPE_RESOURCE: u32 = 5;

fn is_library_native(name: &str) -> bool {
    if name.contains("::") {
        return false;
    }
    name.starts_with("sqlite_")
        || name.starts_with("zlib_")
        || name.starts_with("gui_")
        || name.starts_with("skia_")
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

fn library_native_help(name: &str) -> String {
    let lib = if name.starts_with("sqlite_") {
        "sqlite"
    } else if name.starts_with("zlib_") {
        "zlib"
    } else if name.starts_with("gui_") {
        "gui"
    } else if name.starts_with("skia_") {
        "snask_skia"
    } else if name.starts_with("blaze_") {
        "blaze"
    } else if name.starts_with("auth_") {
        "blaze_auth"
    } else if name.starts_with("sfs_") || name.starts_with("path_") {
        "sfs"
    } else if name.starts_with("os_") {
        "os"
    } else if name.starts_with("s_http_") {
        "requests"
    } else if name.starts_with("thread_") {
        "os"
    } else if name.starts_with("json_") {
        "json"
    } else if name.starts_with("sjson_") {
        "sjson"
    } else if name.starts_with("snif_") {
        "snif"
    } else if name.starts_with("string_") {
        "string"
    } else {
        "a library"
    };

    format!(
        "Native function '{name}' is reserved for libraries.\n\nHow to fix:\n- Use `import \"{lib}\"` and call functions via the module namespace (e.g. `{lib}::...`).",
        name = name,
        lib = lib
    )
}

pub struct LLVMGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, crate::types::Type)>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    value_type: StructType<'ctx>,
    ptr_type: inkwell::types::PointerType<'ctx>,
    i32_type: inkwell::types::IntType<'ctx>,
    i64_type: inkwell::types::IntType<'ctx>,
    f64_type: inkwell::types::FloatType<'ctx>,
    bool_type: inkwell::types::IntType<'ctx>,
    current_func: Option<FunctionValue<'ctx>>,
    local_vars: HashMap<String, (PointerValue<'ctx>, crate::types::Type)>,
    classes: HashMap<String, crate::ast::ClassDecl>,
    active_zone_depth: usize,
    om_contracts: HashMap<String, OmContract>,
}

impl<'ctx> LLVMGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let i32_type = context.i32_type();
        let i64_type = context.i64_type();
        let f64_type = context.f64_type();
        let bool_type = context.bool_type();
        let ptr_type = context.ptr_type(inkwell::AddressSpace::from(0));
        let value_type =
            context.struct_type(&[f64_type.into(), f64_type.into(), ptr_type.into()], false);

        LLVMGenerator {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            value_type,
            ptr_type,
            i32_type,
            i64_type,
            f64_type,
            bool_type,
            current_func: None,
            local_vars: HashMap::new(),
            classes: HashMap::new(),
            active_zone_depth: 0,
            om_contracts: HashMap::new(),
        }
    }

    fn snask_type_to_llvm(&self, ty: &crate::types::Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        use crate::types::Type;
        match ty {
            Type::Int | Type::I64 | Type::U64 | Type::Usize | Type::Isize => self.i64_type.into(),
            Type::I32 => self.i32_type.into(),
            Type::U32 => self.context.i32_type().into(),
            Type::I16 | Type::U16 => self.context.i16_type().into(),
            Type::I8 => self.context.i8_type().into(),
            Type::U8 => self.context.i8_type().into(),
            Type::F32 => self.context.f32_type().into(),
            Type::Float | Type::F64 => self.f64_type.into(),
            Type::Bool => self.bool_type.into(),
            Type::String | Type::Ptr | Type::User(_) => self.ptr_type.into(),
            _ => self.value_type.into(), // Fallback para Any/Complexos
        }
    }

    fn cast_basic_value(
        &self,
        value: BasicValueEnum<'ctx>,
        from: crate::types::Type,
        to: &crate::types::Type,
    ) -> BasicValueEnum<'ctx> {
        if &from == to {
            return value;
        }

        let target = self.snask_type_to_llvm(to);
        if from.is_integer() && to.is_integer() {
            return self
                .builder
                .build_int_cast(value.into_int_value(), target.into_int_type(), "int_cast")
                .unwrap()
                .into();
        }
        if from.is_integer() && to.is_float() {
            return self
                .builder
                .build_signed_int_to_float(
                    value.into_int_value(),
                    target.into_float_type(),
                    "int_to_float",
                )
                .unwrap()
                .into();
        }
        if from.is_float() && to.is_integer() {
            return self
                .builder
                .build_float_to_signed_int(
                    value.into_float_value(),
                    target.into_int_type(),
                    "float_to_int",
                )
                .unwrap()
                .into();
        }
        if from.is_float() && to.is_float() {
            return self
                .builder
                .build_float_cast(
                    value.into_float_value(),
                    target.into_float_type(),
                    "float_cast",
                )
                .unwrap()
                .into();
        }
        value
    }

    fn int_type_for_bits(&self, bits: u32) -> IntType<'ctx> {
        match bits {
            1 => self.bool_type,
            8 => self.context.i8_type(),
            16 => self.context.i16_type(),
            32 => self.context.i32_type(),
            64 => self.context.i64_type(),
            _ => self.context.custom_width_int_type(bits),
        }
    }

    fn systems_type_from_suffix(&self, suffix: &str) -> Option<crate::types::Type> {
        match suffix {
            "u8" => Some(crate::types::Type::U8),
            "u16" => Some(crate::types::Type::U16),
            "u32" => Some(crate::types::Type::U32),
            "u64" => Some(crate::types::Type::U64),
            "i8" => Some(crate::types::Type::I8),
            "i16" => Some(crate::types::Type::I16),
            "i32" => Some(crate::types::Type::I32),
            "i64" => Some(crate::types::Type::I64),
            "usize" => Some(crate::types::Type::Usize),
            "isize" => Some(crate::types::Type::Isize),
            _ => None,
        }
    }

    fn int_value_as(
        &self,
        value: BasicValueEnum<'ctx>,
        from: crate::types::Type,
        target_ty: &crate::types::Type,
    ) -> IntValue<'ctx> {
        self.cast_basic_value(value, from, target_ty)
            .into_int_value()
    }

    fn bool_from_int_compare_zero(
        &self,
        value: IntValue<'ctx>,
        pred: inkwell::IntPredicate,
    ) -> IntValue<'ctx> {
        self.builder
            .build_int_compare(pred, value, value.get_type().const_zero(), "cmp_zero")
            .unwrap()
    }

    fn malloc_function(&self) -> FunctionValue<'ctx> {
        self.module.get_function("malloc").unwrap_or_else(|| {
            self.module.add_function(
                "malloc",
                self.ptr_type.fn_type(&[self.i64_type.into()], false),
                None,
            )
        })
    }

    fn free_function(&self) -> FunctionValue<'ctx> {
        self.module.get_function("free").unwrap_or_else(|| {
            self.module.add_function(
                "free",
                self.context
                    .void_type()
                    .fn_type(&[self.ptr_type.into()], false),
                None,
            )
        })
    }

    fn memset_function(&self) -> FunctionValue<'ctx> {
        self.module.get_function("memset").unwrap_or_else(|| {
            self.module.add_function(
                "memset",
                self.ptr_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.context.i32_type().into(),
                        self.i64_type.into(),
                    ],
                    false,
                ),
                None,
            )
        })
    }

    fn memcpy_function(&self) -> FunctionValue<'ctx> {
        self.module.get_function("memcpy").unwrap_or_else(|| {
            self.module.add_function(
                "memcpy",
                self.ptr_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.i64_type.into(),
                    ],
                    false,
                ),
                None,
            )
        })
    }

    fn ptr_offset(
        &self,
        base: PointerValue<'ctx>,
        offset: IntValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let offset = self
            .builder
            .build_int_cast(offset, self.i64_type, "ptr_offset_i64")
            .unwrap();
        unsafe {
            self.builder
                .build_gep(self.context.i8_type(), base, &[offset], name)
                .unwrap()
        }
    }

    fn load_u8_at(&self, base: PointerValue<'ctx>, offset: IntValue<'ctx>) -> IntValue<'ctx> {
        let ptr = self.ptr_offset(base, offset, "u8_ptr");
        self.builder
            .build_load(self.context.i8_type(), ptr, "load_u8")
            .unwrap()
            .into_int_value()
    }

    fn emit_systems_low_level_builtin(
        &self,
        name: &str,
        args: &[Expr],
    ) -> Result<Option<(BasicValueEnum<'ctx>, crate::types::Type)>, String> {
        if let Some(suffix) = name.strip_prefix("as_") {
            if let Some(target_ty) = self.systems_type_from_suffix(suffix) {
                if args.len() != 1 {
                    return Err(format!("{name} expects 1 argument."));
                }
                let (value, from_ty) = self.evaluate_expression(args[0].clone())?;
                let casted = self.cast_basic_value(value, from_ty, &target_ty);
                return Ok(Some((casted, target_ty)));
            }
        }

        match name {
            "lo_u8" | "hi_u8" => {
                if args.len() != 1 {
                    return Err(format!("{name} expects 1 argument."));
                }
                let (value, ty) = self.evaluate_expression(args[0].clone())?;
                let v = self.int_value_as(value, ty, &crate::types::Type::U16);
                let shifted = if name == "hi_u8" {
                    self.builder
                        .build_right_shift(
                            v,
                            self.context.i16_type().const_int(8, false),
                            false,
                            "hi_shift",
                        )
                        .unwrap()
                } else {
                    v
                };
                let out = self
                    .builder
                    .build_int_cast(shifted, self.context.i8_type(), "to_u8")
                    .unwrap();
                Ok(Some((out.into(), crate::types::Type::U8)))
            }
            "make_u16" => {
                if args.len() != 2 {
                    return Err("make_u16 expects 2 arguments.".to_string());
                }
                let (lo, lo_ty) = self.evaluate_expression(args[0].clone())?;
                let (hi, hi_ty) = self.evaluate_expression(args[1].clone())?;
                let lo = self.int_value_as(lo, lo_ty, &crate::types::Type::U16);
                let hi = self.int_value_as(hi, hi_ty, &crate::types::Type::U16);
                let hi = self
                    .builder
                    .build_left_shift(hi, self.context.i16_type().const_int(8, false), "hi")
                    .unwrap();
                let out = self.builder.build_or(lo, hi, "make_u16").unwrap();
                Ok(Some((out.into(), crate::types::Type::U16)))
            }
            "is_zero_u8" | "is_negative_u8" => {
                if args.len() != 1 {
                    return Err(format!("{name} expects 1 argument."));
                }
                let (value, ty) = self.evaluate_expression(args[0].clone())?;
                let v = self.int_value_as(value, ty, &crate::types::Type::U8);
                let out = if name == "is_zero_u8" {
                    self.bool_from_int_compare_zero(v, inkwell::IntPredicate::EQ)
                } else {
                    let masked = self
                        .builder
                        .build_and(v, self.context.i8_type().const_int(0x80, false), "neg_mask")
                        .unwrap();
                    self.bool_from_int_compare_zero(masked, inkwell::IntPredicate::NE)
                };
                Ok(Some((out.into(), crate::types::Type::Bool)))
            }
            "bit_test" | "bit_set" | "bit_clear" | "bit_toggle" | "bit_write" => {
                if args.len() < 2 || (name == "bit_write" && args.len() != 3) {
                    return Err(format!(
                        "{name} expects {} arguments.",
                        if name == "bit_write" { 3 } else { 2 }
                    ));
                }
                let (value, value_ty) = self.evaluate_expression(args[0].clone())?;
                let (bit, bit_ty) = self.evaluate_expression(args[1].clone())?;
                let result_ty = if value_ty.is_integer() {
                    value_ty.clone()
                } else {
                    crate::types::Type::Int
                };
                let bits = result_ty.bit_width().unwrap_or(64);
                let int_ty = self.int_type_for_bits(bits);
                let value = self.int_value_as(value, value_ty, &result_ty);
                let bit = self.int_value_as(bit, bit_ty, &crate::types::Type::U8);
                let bit = self
                    .builder
                    .build_int_cast(bit, int_ty, "bit_index")
                    .unwrap();
                let one = int_ty.const_int(1, false);
                let mask = self.builder.build_left_shift(one, bit, "bit_mask").unwrap();
                let out = match name {
                    "bit_test" => {
                        let masked = self
                            .builder
                            .build_and(value, mask, "bit_test_mask")
                            .unwrap();
                        return Ok(Some((
                            self.bool_from_int_compare_zero(masked, inkwell::IntPredicate::NE)
                                .into(),
                            crate::types::Type::Bool,
                        )));
                    }
                    "bit_set" => self.builder.build_or(value, mask, "bit_set").unwrap(),
                    "bit_clear" => {
                        let inv = self.builder.build_not(mask, "bit_clear_mask").unwrap();
                        self.builder.build_and(value, inv, "bit_clear").unwrap()
                    }
                    "bit_toggle" => self.builder.build_xor(value, mask, "bit_toggle").unwrap(),
                    "bit_write" => {
                        let (enabled, enabled_ty) = self.evaluate_expression(args[2].clone())?;
                        let enabled =
                            self.int_value_as(enabled, enabled_ty, &crate::types::Type::Bool);
                        let set = self.builder.build_or(value, mask, "bit_write_set").unwrap();
                        let inv = self.builder.build_not(mask, "bit_write_inv").unwrap();
                        let clear = self
                            .builder
                            .build_and(value, inv, "bit_write_clear")
                            .unwrap();
                        self.builder
                            .build_select(enabled, set, clear, "bit_write")
                            .unwrap()
                            .into_int_value()
                    }
                    _ => unreachable!(),
                };
                Ok(Some((out.into(), result_ty)))
            }
            "flag_has" | "flag_set" | "flag_clear" | "flag_write" => {
                let mapped = match name {
                    "flag_has" => "bit_test",
                    "flag_set" => "bit_set",
                    "flag_clear" => "bit_clear",
                    "flag_write" => "bit_write",
                    _ => unreachable!(),
                };
                self.emit_systems_low_level_builtin(mapped, args)
            }
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add"
            | "wrapping_inc" | "wrapping_dec" => {
                let expected = if matches!(name, "wrapping_inc" | "wrapping_dec") {
                    1
                } else {
                    2
                };
                if args.len() != expected {
                    return Err(format!("{name} expects {expected} argument(s)."));
                }
                let (lhs, lty) = self.evaluate_expression(args[0].clone())?;
                let result_ty = if lty.is_integer() {
                    lty.clone()
                } else {
                    crate::types::Type::Int
                };
                let li = self.int_value_as(lhs, lty, &result_ty);
                let ri = if expected == 1 {
                    li.get_type().const_int(1, false)
                } else {
                    let (rhs, rty) = self.evaluate_expression(args[1].clone())?;
                    self.int_value_as(rhs, rty, &result_ty)
                };
                let raw = match name {
                    "wrapping_add" | "wrapping_inc" => {
                        self.builder.build_int_add(li, ri, "wrap_add").unwrap()
                    }
                    "wrapping_sub" | "wrapping_dec" => {
                        self.builder.build_int_sub(li, ri, "wrap_sub").unwrap()
                    }
                    "wrapping_mul" => self.builder.build_int_mul(li, ri, "wrap_mul").unwrap(),
                    "saturating_add" => {
                        let sum = self.builder.build_int_add(li, ri, "sat_add").unwrap();
                        let pred = if result_ty.is_unsigned_integer() {
                            inkwell::IntPredicate::ULT
                        } else {
                            inkwell::IntPredicate::SLT
                        };
                        let overflow = self
                            .builder
                            .build_int_compare(pred, sum, li, "sat_overflow")
                            .unwrap();
                        let max = sum.get_type().const_all_ones();
                        self.builder
                            .build_select(overflow, max, sum, "sat_select")
                            .unwrap()
                            .into_int_value()
                    }
                    _ => unreachable!(),
                };
                Ok(Some((raw.into(), result_ty)))
            }
            "carry_add_u8" | "borrow_sub_u8" | "overflow_add_i8" | "overflow_sub_i8" => {
                if args.len() != 3 {
                    return Err(format!("{name} expects 3 arguments."));
                }
                let (a, aty) = self.evaluate_expression(args[0].clone())?;
                let (b, bty) = self.evaluate_expression(args[1].clone())?;
                let (c, cty) = self.evaluate_expression(args[2].clone())?;
                let a16 = self.int_value_as(a, aty, &crate::types::Type::U16);
                let b16 = self.int_value_as(b, bty, &crate::types::Type::U16);
                let c16 = self.int_value_as(c, cty, &crate::types::Type::U16);
                let out = match name {
                    "carry_add_u8" => {
                        let sum = self.builder.build_int_add(a16, b16, "carry_ab").unwrap();
                        let sum = self.builder.build_int_add(sum, c16, "carry_abc").unwrap();
                        self.builder
                            .build_int_compare(
                                inkwell::IntPredicate::UGT,
                                sum,
                                self.context.i16_type().const_int(0xFF, false),
                                "carry",
                            )
                            .unwrap()
                    }
                    "borrow_sub_u8" => {
                        let b_plus = self.builder.build_int_add(b16, c16, "borrow_bc").unwrap();
                        self.builder
                            .build_int_compare(inkwell::IntPredicate::ULT, a16, b_plus, "borrow")
                            .unwrap()
                    }
                    "overflow_add_i8" => {
                        let a8 = self
                            .builder
                            .build_int_cast(a16, self.context.i8_type(), "a8")
                            .unwrap();
                        let b8 = self
                            .builder
                            .build_int_cast(b16, self.context.i8_type(), "b8")
                            .unwrap();
                        let c8 = self
                            .builder
                            .build_int_cast(c16, self.context.i8_type(), "c8")
                            .unwrap();
                        let sum = self.builder.build_int_add(a8, b8, "ov_ab").unwrap();
                        let sum = self.builder.build_int_add(sum, c8, "ov_abc").unwrap();
                        let xor1 = self.builder.build_xor(a8, sum, "ov_x1").unwrap();
                        let xor2 = self.builder.build_xor(a8, b8, "ov_x2").unwrap();
                        let not_xor2 = self.builder.build_not(xor2, "ov_not").unwrap();
                        let masked = self.builder.build_and(xor1, not_xor2, "ov_mask").unwrap();
                        let sign = self
                            .builder
                            .build_and(
                                masked,
                                self.context.i8_type().const_int(0x80, false),
                                "ov_sign",
                            )
                            .unwrap();
                        self.bool_from_int_compare_zero(sign, inkwell::IntPredicate::NE)
                    }
                    "overflow_sub_i8" => {
                        let a8 = self
                            .builder
                            .build_int_cast(a16, self.context.i8_type(), "a8")
                            .unwrap();
                        let b8 = self
                            .builder
                            .build_int_cast(b16, self.context.i8_type(), "b8")
                            .unwrap();
                        let c8 = self
                            .builder
                            .build_int_cast(c16, self.context.i8_type(), "c8")
                            .unwrap();
                        let sub = self.builder.build_int_add(b8, c8, "sub_bc").unwrap();
                        let res = self.builder.build_int_sub(a8, sub, "sub_res").unwrap();
                        let xor1 = self.builder.build_xor(a8, res, "sub_ov_x1").unwrap();
                        let xor2 = self.builder.build_xor(a8, b8, "sub_ov_x2").unwrap();
                        let masked = self.builder.build_and(xor1, xor2, "sub_ov_mask").unwrap();
                        let sign = self
                            .builder
                            .build_and(
                                masked,
                                self.context.i8_type().const_int(0x80, false),
                                "sub_ov_sign",
                            )
                            .unwrap();
                        self.bool_from_int_compare_zero(sign, inkwell::IntPredicate::NE)
                    }
                    _ => unreachable!(),
                };
                Ok(Some((out.into(), crate::types::Type::Bool)))
            }
            "mem_alloc" | "mem_alloc_zero" => {
                if args.len() != 1 {
                    return Err(format!("{name} expects 1 argument."));
                }
                let (size, size_ty) = self.evaluate_expression(args[0].clone())?;
                let size = self.int_value_as(size, size_ty, &crate::types::Type::Usize);
                let call = self
                    .builder
                    .build_call(self.malloc_function(), &[size.into()], "mem_alloc")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "malloc did not return a pointer".to_string())?;
                if name == "mem_alloc_zero" {
                    self.builder
                        .build_call(
                            self.memset_function(),
                            &[
                                call.into_pointer_value().into(),
                                self.context.i32_type().const_zero().into(),
                                size.into(),
                            ],
                            "mem_zero",
                        )
                        .unwrap();
                }
                Ok(Some((call, crate::types::Type::Ptr)))
            }
            "mem_free" => {
                if args.len() != 1 {
                    return Err("mem_free expects 1 argument.".to_string());
                }
                let (ptr, ty) = self.evaluate_expression(args[0].clone())?;
                if ty != crate::types::Type::Ptr {
                    return Err("mem_free expects a ptr.".to_string());
                }
                self.builder
                    .build_call(
                        self.free_function(),
                        &[ptr.into_pointer_value().into()],
                        "mem_free",
                    )
                    .unwrap();
                Ok(Some((
                    self.i64_type.const_zero().into(),
                    crate::types::Type::Void,
                )))
            }
            "ptr_add" => {
                if args.len() != 2 {
                    return Err("ptr_add expects 2 arguments.".to_string());
                }
                let (ptr, ptr_ty) = self.evaluate_expression(args[0].clone())?;
                if ptr_ty != crate::types::Type::Ptr {
                    return Err("ptr_add expects ptr as first argument.".to_string());
                }
                let (off, off_ty) = self.evaluate_expression(args[1].clone())?;
                let off = self.int_value_as(off, off_ty, &crate::types::Type::Isize);
                let out = self.ptr_offset(ptr.into_pointer_value(), off, "ptr_add");
                Ok(Some((out.into(), crate::types::Type::Ptr)))
            }
            "mem_read_u8" | "mem_read_u16" | "mem_read_u32" => {
                if args.len() != 2 {
                    return Err(format!("{name} expects 2 arguments."));
                }
                let (ptr, ptr_ty) = self.evaluate_expression(args[0].clone())?;
                if ptr_ty != crate::types::Type::Ptr {
                    return Err(format!("{name} expects ptr as first argument."));
                }
                let (off, off_ty) = self.evaluate_expression(args[1].clone())?;
                let off = self.int_value_as(off, off_ty, &crate::types::Type::Isize);
                let base = ptr.into_pointer_value();
                let b0 = self.load_u8_at(base, off);
                let out = match name {
                    "mem_read_u8" => return Ok(Some((b0.into(), crate::types::Type::U8))),
                    "mem_read_u16" => {
                        let b1 = self.load_u8_at(
                            base,
                            self.builder
                                .build_int_add(off, self.i64_type.const_int(1, false), "off1")
                                .unwrap(),
                        );
                        let lo = self
                            .builder
                            .build_int_z_extend(b0, self.context.i16_type(), "lo16")
                            .unwrap();
                        let hi = self
                            .builder
                            .build_int_z_extend(b1, self.context.i16_type(), "hi16")
                            .unwrap();
                        let hi = self
                            .builder
                            .build_left_shift(
                                hi,
                                self.context.i16_type().const_int(8, false),
                                "hi16s",
                            )
                            .unwrap();
                        self.builder.build_or(lo, hi, "read_u16").unwrap().into()
                    }
                    "mem_read_u32" => {
                        let mut acc = self
                            .builder
                            .build_int_z_extend(b0, self.context.i32_type(), "b0_32")
                            .unwrap();
                        for i in 1..4 {
                            let bi = self.load_u8_at(
                                base,
                                self.builder
                                    .build_int_add(off, self.i64_type.const_int(i, false), "offi")
                                    .unwrap(),
                            );
                            let bi = self
                                .builder
                                .build_int_z_extend(bi, self.context.i32_type(), "bi32")
                                .unwrap();
                            let bi = self
                                .builder
                                .build_left_shift(
                                    bi,
                                    self.context.i32_type().const_int((i * 8) as u64, false),
                                    "bis",
                                )
                                .unwrap();
                            acc = self.builder.build_or(acc, bi, "read_u32_acc").unwrap();
                        }
                        acc.into()
                    }
                    _ => unreachable!(),
                };
                let ret_ty = if name == "mem_read_u16" {
                    crate::types::Type::U16
                } else {
                    crate::types::Type::U32
                };
                Ok(Some((out, ret_ty)))
            }
            "mem_write_u8" | "mem_write_u16" | "mem_write_u32" => {
                if args.len() != 3 {
                    return Err(format!("{name} expects 3 arguments."));
                }
                let (ptr, ptr_ty) = self.evaluate_expression(args[0].clone())?;
                if ptr_ty != crate::types::Type::Ptr {
                    return Err(format!("{name} expects ptr as first argument."));
                }
                let (off, off_ty) = self.evaluate_expression(args[1].clone())?;
                let (val, val_ty) = self.evaluate_expression(args[2].clone())?;
                let base = ptr.into_pointer_value();
                let off = self.int_value_as(off, off_ty, &crate::types::Type::Isize);
                let bytes = match name {
                    "mem_write_u8" => 1,
                    "mem_write_u16" => 2,
                    "mem_write_u32" => 4,
                    _ => unreachable!(),
                };
                let value_ty = match bytes {
                    1 => crate::types::Type::U8,
                    2 => crate::types::Type::U16,
                    _ => crate::types::Type::U32,
                };
                let val = self.int_value_as(val, val_ty, &value_ty);
                for i in 0..bytes {
                    let byte = if i == 0 {
                        self.builder
                            .build_int_cast(val, self.context.i8_type(), "w_b0")
                            .unwrap()
                    } else {
                        let shifted = self
                            .builder
                            .build_right_shift(
                                val,
                                val.get_type().const_int((i * 8) as u64, false),
                                false,
                                "w_shift",
                            )
                            .unwrap();
                        self.builder
                            .build_int_cast(shifted, self.context.i8_type(), "w_b")
                            .unwrap()
                    };
                    let dst = self.ptr_offset(
                        base,
                        self.builder
                            .build_int_add(off, self.i64_type.const_int(i as u64, false), "w_off")
                            .unwrap(),
                        "w_ptr",
                    );
                    self.builder.build_store(dst, byte).unwrap();
                }
                Ok(Some((
                    self.i64_type.const_zero().into(),
                    crate::types::Type::Void,
                )))
            }
            "mem_fill_u8" => {
                if args.len() != 3 {
                    return Err("mem_fill_u8 expects 3 arguments.".to_string());
                }
                let (ptr, ptr_ty) = self.evaluate_expression(args[0].clone())?;
                if ptr_ty != crate::types::Type::Ptr {
                    return Err("mem_fill_u8 expects ptr as first argument.".to_string());
                }
                let (value, value_ty) = self.evaluate_expression(args[1].clone())?;
                let (len, len_ty) = self.evaluate_expression(args[2].clone())?;
                let value = self.int_value_as(value, value_ty, &crate::types::Type::U8);
                let value = self
                    .builder
                    .build_int_z_extend(value, self.context.i32_type(), "fill_i32")
                    .unwrap();
                let len = self.int_value_as(len, len_ty, &crate::types::Type::Usize);
                self.builder
                    .build_call(
                        self.memset_function(),
                        &[ptr.into_pointer_value().into(), value.into(), len.into()],
                        "mem_fill",
                    )
                    .unwrap();
                Ok(Some((
                    self.i64_type.const_zero().into(),
                    crate::types::Type::Void,
                )))
            }
            "mem_copy" => {
                if args.len() != 3 {
                    return Err("mem_copy expects 3 arguments.".to_string());
                }
                let (dst, dst_ty) = self.evaluate_expression(args[0].clone())?;
                let (src, src_ty) = self.evaluate_expression(args[1].clone())?;
                let (len, len_ty) = self.evaluate_expression(args[2].clone())?;
                if dst_ty != crate::types::Type::Ptr || src_ty != crate::types::Type::Ptr {
                    return Err("mem_copy expects ptr, ptr, len.".to_string());
                }
                let len = self.int_value_as(len, len_ty, &crate::types::Type::Usize);
                self.builder
                    .build_call(
                        self.memcpy_function(),
                        &[
                            dst.into_pointer_value().into(),
                            src.into_pointer_value().into(),
                            len.into(),
                        ],
                        "mem_copy",
                    )
                    .unwrap();
                Ok(Some((
                    self.i64_type.const_zero().into(),
                    crate::types::Type::Void,
                )))
            }
            _ => Ok(None),
        }
    }

    pub fn set_om_contracts(&mut self, contracts: Vec<OmContract>) {
        self.om_contracts = contracts
            .into_iter()
            .map(|contract| (contract.library.clone(), contract))
            .collect();
    }

    fn create_entry_block_alloca<T: inkwell::types::BasicType<'ctx>>(
        &self,
        ty: T,
        name: &str,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.current_func.unwrap().get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(inst) => builder.position_before(&inst),
            None => builder.position_at_end(entry),
        }
        builder.build_alloca(ty, name).unwrap()
    }

    pub fn generate(&mut self, program: Program) -> Result<String, String> {
        self.declare_runtime();

        let want_auto_use_skia = program.iter().any(|st| match &st.kind {
            StmtKind::VarDeclaration(d) => d.name == "USE_SKIA",
            StmtKind::MutDeclaration(d) => d.name == "USE_SKIA",
            StmtKind::ConstDeclaration(d) => d.name == "USE_SKIA",
            _ => false,
        });

        // Declara funções globais e preenche mapa de classes
        for stmt in &program {
            if let StmtKind::FuncDeclaration(func) = &stmt.kind {
                self.declare_function(func)?;
            }
            if let StmtKind::ClassDeclaration(class) = &stmt.kind {
                let mut c = class.clone();
                for method in &mut c.methods {
                    // Adiciona 'self' como primeiro parâmetro se não existir
                    if !method.params.iter().any(|p| p.0 == "self") {
                        method
                            .params
                            .insert(0, ("self".to_string(), crate::types::Type::Any));
                    }
                    let mut m = method.clone();
                    m.name = format!("{}::{}", c.name, m.name);
                    self.declare_function(&m)?;
                }
                self.classes.insert(c.name.clone(), c);
            }
        }

        let i32_type = self.context.i32_type();
        let main_func = self
            .module
            .add_function("main", i32_type.fn_type(&[], false), None);
        let entry = self.context.append_basic_block(main_func, "entry");
        self.builder.position_at_end(entry);
        self.current_func = Some(main_func);

        // Always execute top-level statements (globals) before start.
        // Isso permite que módulos importados inicializem estado (mut/let/const) mesmo quando existe main::start.
        for stmt in &program {
            if !matches!(
                stmt.kind,
                StmtKind::FuncDeclaration(_) | StmtKind::ClassDeclaration(_)
            ) {
                self.generate_statement(stmt.clone())?;
            }
        }

        // DX: optional real Skia backend switch.
        // If the user defines `USE_SKIA = 1` at top-level, enable Skia before calling main::start.
        // Default remains Cairo.
        if want_auto_use_skia {
            if let Some((p, _)) = self.variables.get("USE_SKIA") {
                let v = self
                    .builder
                    .build_load(self.value_type, *p, "use_skia_val")
                    .unwrap()
                    .into_struct_value();
                let n = self
                    .builder
                    .build_extract_value(v, 1, "use_skia_n")
                    .unwrap()
                    .into_float_value();
                let is_true = self
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        n,
                        self.context.f64_type().const_float(0.0),
                        "use_skia_true",
                    )
                    .unwrap();
                let b = self
                    .builder
                    .build_unsigned_int_to_float(is_true, self.context.f64_type(), "use_skia_bf")
                    .unwrap();
                let mut bs = self.value_type.get_undef();
                bs = self
                    .builder
                    .build_insert_value(bs, self.context.f64_type().const_float(2.0), 0, "t")
                    .unwrap()
                    .into_struct_value(); // BOOL
                bs = self
                    .builder
                    .build_insert_value(bs, b, 1, "v")
                    .unwrap()
                    .into_struct_value();
                bs = self
                    .builder
                    .build_insert_value(bs, self.ptr_type.const_null(), 2, "p")
                    .unwrap()
                    .into_struct_value();
                let b_ptr = self.create_entry_block_alloca(self.value_type, "use_skia_arg");
                self.builder.build_store(b_ptr, bs).unwrap();

                let f = self.functions.get("skia_use_real").unwrap();
                let r_a = self.create_entry_block_alloca(self.value_type, "ra_use_skia");
                self.builder
                    .build_call(*f, &[r_a.into(), b_ptr.into()], "use_skia")
                    .unwrap();
            }
        }

        // Se houver uma class main, chama o ponto de entrada (prioriza 'start' ou o primeiro método se start não existir)
        let mut entry_point_found = false;
        for stmt in &program {
            if let StmtKind::ClassDeclaration(class) = &stmt.kind {
                if class.name == "main" && !class.methods.is_empty() {
                    // Busca 'start' ou pega o primeiro método
                    let method = class
                        .methods
                        .iter()
                        .find(|m| m.name == "start")
                        .or_else(|| class.methods.get(0))
                        .unwrap();

                    let f_name = format!("main::{}", method.name);
                    if let Some(f) = self.functions.get(&f_name) {
                        let mut l_args = Vec::new();
                        let r_a = self.create_entry_block_alloca(self.value_type, "ra");
                        l_args.push(r_a.into());
                        self.builder.build_call(*f, &l_args, "call_entry").unwrap();
                        entry_point_found = true;
                    }
                    break;
                }
            }
        }
        let _ = entry_point_found;

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_return(Some(&i32_type.const_int(0, false)))
                .unwrap();
        }

        // Gera o corpo das funções
        for stmt in program {
            if let StmtKind::FuncDeclaration(func) = &stmt.kind {
                self.generate_function_body(func.clone())?;
            }
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
        self.module.add_function(
            "s_print",
            void_type.fn_type(&[self.ptr_type.into()], false),
            None,
        );
        self.module
            .add_function("s_println", void_type.fn_type(&[], false), None);

        let fn_1 = void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false);
        let fn_2 = void_type.fn_type(
            &[
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
            ],
            false,
        );
        let fn_3 = void_type.fn_type(
            &[
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
            ],
            false,
        );
        let fn_4 = void_type.fn_type(
            &[
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
            ],
            false,
        );
        let fn_5 = void_type.fn_type(
            &[
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
            ],
            false,
        );
        let fn_alloc = void_type.fn_type(
            &[
                self.ptr_type.into(),
                self.ptr_type.into(),
                self.ptr_type.into(),
            ],
            false,
        );

        self.functions.insert(
            "sfs_read".to_string(),
            self.module.add_function("sfs_read", fn_1, None),
        );
        self.functions.insert(
            "sfs_write".to_string(),
            self.module.add_function("sfs_write", fn_2, None),
        );
        self.functions.insert(
            "sfs_append".to_string(),
            self.module.add_function("sfs_append", fn_2, None),
        );
        self.functions.insert(
            "sfs_write_mb".to_string(),
            self.module.add_function("sfs_write_mb", fn_2, None),
        );
        self.functions.insert(
            "sfs_count_bytes".to_string(),
            self.module.add_function("sfs_count_bytes", fn_1, None),
        );
        self.functions.insert(
            "sfs_bench_create_small_files".to_string(),
            self.module
                .add_function("sfs_bench_create_small_files", fn_3, None),
        );
        self.functions.insert(
            "sfs_bench_count_entries".to_string(),
            self.module
                .add_function("sfs_bench_count_entries", fn_1, None),
        );
        self.functions.insert(
            "sfs_bench_delete_small_files".to_string(),
            self.module
                .add_function("sfs_bench_delete_small_files", fn_2, None),
        );
        self.functions.insert(
            "sfs_delete".to_string(),
            self.module.add_function("sfs_delete", fn_1, None),
        );
        self.functions.insert(
            "sfs_exists".to_string(),
            self.module.add_function("sfs_exists", fn_1, None),
        );
        self.functions.insert(
            "sfs_copy".to_string(),
            self.module.add_function("sfs_copy", fn_2, None),
        );
        self.functions.insert(
            "sfs_move".to_string(),
            self.module.add_function("sfs_move", fn_2, None),
        );
        self.functions.insert(
            "sfs_mkdir".to_string(),
            self.module.add_function("sfs_mkdir", fn_1, None),
        );
        self.functions.insert(
            "sfs_is_file".to_string(),
            self.module.add_function("sfs_is_file", fn_1, None),
        );
        self.functions.insert(
            "sfs_is_dir".to_string(),
            self.module.add_function("sfs_is_dir", fn_1, None),
        );
        self.functions.insert(
            "sfs_listdir".to_string(),
            self.module.add_function("sfs_listdir", fn_1, None),
        );
        self.functions.insert(
            "s_http_get".to_string(),
            self.module.add_function("s_http_get", fn_1, None),
        );
        self.functions.insert(
            "s_http_post".to_string(),
            self.module.add_function("s_http_post", fn_2, None),
        );
        self.functions.insert(
            "s_http_put".to_string(),
            self.module.add_function("s_http_put", fn_2, None),
        );
        self.functions.insert(
            "s_http_delete".to_string(),
            self.module.add_function("s_http_delete", fn_1, None),
        );
        self.functions.insert(
            "s_http_patch".to_string(),
            self.module.add_function("s_http_patch", fn_2, None),
        );
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

        let f_sqrt = self.module.add_function("s_sqrt", fn_1, None);
        self.functions.insert("s_sqrt".to_string(), f_sqrt);
        self.functions.insert("sqrt".to_string(), f_sqrt);

        let f_sin = self.module.add_function("s_sin", fn_1, None);
        self.functions.insert("s_sin".to_string(), f_sin);
        self.functions.insert("sin".to_string(), f_sin);

        let f_cos = self.module.add_function("s_cos", fn_1, None);
        self.functions.insert("s_cos".to_string(), f_cos);
        self.functions.insert("cos".to_string(), f_cos);

        let f_floor = self.module.add_function("s_floor", fn_1, None);
        self.functions.insert("s_floor".to_string(), f_floor);
        self.functions.insert("floor".to_string(), f_floor);

        let f_ceil = self.module.add_function("s_ceil", fn_1, None);
        self.functions.insert("s_ceil".to_string(), f_ceil);
        self.functions.insert("ceil".to_string(), f_ceil);

        let f_round = self.module.add_function("s_round", fn_1, None);
        self.functions.insert("s_round".to_string(), f_round);
        self.functions.insert("round".to_string(), f_round);

        let f_pow = self.module.add_function("s_pow", fn_2, None);
        self.functions.insert("s_pow".to_string(), f_pow);
        self.functions.insert("pow".to_string(), f_pow);

        let f_len = self.module.add_function("s_len", fn_1, None);
        self.functions.insert("s_len".to_string(), f_len);
        self.functions.insert("len".to_string(), f_len);

        // strict equality helper (===)
        self.functions.insert(
            "s_eq".to_string(),
            self.module.add_function("s_eq", fn_2, None),
        );
        self.functions.insert(
            "s_ne".to_string(),
            self.module.add_function("s_ne", fn_2, None),
        );

        let f_upper = self.module.add_function("s_upper", fn_1, None);
        self.functions.insert("s_upper".to_string(), f_upper);
        self.functions.insert("upper".to_string(), f_upper);

        self.functions.insert(
            "mod".to_string(),
            self.module.add_function("mod", fn_2, None),
        );

        self.functions.insert(
            "substring".to_string(),
            self.module.add_function("substring", fn_3, None),
        );

        let f_time = self.module.add_function(
            "s_time",
            void_type.fn_type(&[self.ptr_type.into()], false),
            None,
        );
        self.functions.insert("s_time".to_string(), f_time);
        self.functions.insert("time".to_string(), f_time);

        let f_sleep = self.module.add_function("s_sleep", fn_1, None);
        self.functions.insert("s_sleep".to_string(), f_sleep);
        self.functions.insert("sleep".to_string(), f_sleep);

        let f_exit = self.module.add_function("s_exit", fn_1, None);
        self.functions.insert("exit".to_string(), f_exit);
        self.functions.insert(
            "s_system".to_string(),
            self.module.add_function("s_system", fn_1, None),
        );
        self.functions.insert(
            "is_nil".to_string(),
            self.module.add_function("is_nil", fn_1, None),
        );
        self.functions.insert(
            "is_str".to_string(),
            self.module.add_function("is_str", fn_1, None),
        );
        self.functions.insert(
            "is_obj".to_string(),
            self.module.add_function("is_obj", fn_1, None),
        );
        self.functions.insert(
            "os_cwd".to_string(),
            self.module.add_function(
                "os_cwd",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "os_platform".to_string(),
            self.module.add_function(
                "os_platform",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "os_arch".to_string(),
            self.module.add_function(
                "os_arch",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "os_getenv".to_string(),
            self.module.add_function("os_getenv", fn_1, None),
        );
        self.functions.insert(
            "os_setenv".to_string(),
            self.module.add_function("os_setenv", fn_2, None),
        );
        self.functions.insert(
            "os_random_hex".to_string(),
            self.module.add_function("os_random_hex", fn_1, None),
        );

        self.functions.insert(
            "sfs_size".to_string(),
            self.module.add_function("sfs_size", fn_1, None),
        );
        self.functions.insert(
            "sfs_mtime".to_string(),
            self.module.add_function("sfs_mtime", fn_1, None),
        );
        self.functions.insert(
            "sfs_rmdir".to_string(),
            self.module.add_function("sfs_rmdir", fn_1, None),
        );

        self.functions.insert(
            "path_basename".to_string(),
            self.module.add_function("path_basename", fn_1, None),
        );
        self.functions.insert(
            "path_dirname".to_string(),
            self.module.add_function("path_dirname", fn_1, None),
        );
        self.functions.insert(
            "path_extname".to_string(),
            self.module.add_function("path_extname", fn_1, None),
        );
        self.functions.insert(
            "path_join".to_string(),
            self.module.add_function("path_join", fn_2, None),
        );
        self.functions.insert(
            "blaze_run".to_string(),
            self.module.add_function("blaze_run", fn_2, None),
        );
        self.functions.insert(
            "blaze_qs_get".to_string(),
            self.module.add_function("blaze_qs_get", fn_2, None),
        );
        self.functions.insert(
            "blaze_cookie_get".to_string(),
            self.module.add_function("blaze_cookie_get", fn_2, None),
        );
        self.functions.insert(
            "auth_random_hex".to_string(),
            self.module.add_function("auth_random_hex", fn_1, None),
        );
        self.functions.insert(
            "auth_now".to_string(),
            self.module.add_function(
                "auth_now",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "auth_const_time_eq".to_string(),
            self.module.add_function("auth_const_time_eq", fn_2, None),
        );
        self.functions.insert(
            "auth_hash_password".to_string(),
            self.module.add_function("auth_hash_password", fn_1, None),
        );
        self.functions.insert(
            "auth_verify_password".to_string(),
            self.module.add_function("auth_verify_password", fn_2, None),
        );
        self.functions.insert(
            "auth_session_id".to_string(),
            self.module.add_function(
                "auth_session_id",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "auth_csrf_token".to_string(),
            self.module.add_function(
                "auth_csrf_token",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "auth_cookie_kv".to_string(),
            self.module.add_function("auth_cookie_kv", fn_2, None),
        );
        self.functions.insert(
            "auth_cookie_session".to_string(),
            self.module.add_function("auth_cookie_session", fn_1, None),
        );
        self.functions.insert(
            "auth_cookie_delete".to_string(),
            self.module.add_function("auth_cookie_delete", fn_1, None),
        );
        self.functions.insert(
            "auth_bearer_header".to_string(),
            self.module.add_function("auth_bearer_header", fn_1, None),
        );
        self.functions.insert(
            "auth_ok".to_string(),
            self.module.add_function(
                "auth_ok",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "auth_fail".to_string(),
            self.module.add_function(
                "auth_fail",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "auth_version".to_string(),
            self.module.add_function(
                "auth_version",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );

        // GUI (GTK) - handles are strings
        self.functions.insert(
            "gui_init".to_string(),
            self.module.add_function(
                "gui_init",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_run".to_string(),
            self.module.add_function(
                "gui_run",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_quit".to_string(),
            self.module.add_function(
                "gui_quit",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_window".to_string(),
            self.module.add_function("gui_window", fn_3, None),
        );
        self.functions.insert(
            "gui_set_title".to_string(),
            self.module.add_function("gui_set_title", fn_2, None),
        );
        self.functions.insert(
            "gui_set_resizable".to_string(),
            self.module.add_function("gui_set_resizable", fn_2, None),
        );
        self.functions.insert(
            "gui_autosize".to_string(),
            self.module.add_function("gui_autosize", fn_1, None),
        );
        self.functions.insert(
            "gui_vbox".to_string(),
            self.module.add_function(
                "gui_vbox",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_hbox".to_string(),
            self.module.add_function(
                "gui_hbox",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_eventbox".to_string(),
            self.module.add_function(
                "gui_eventbox",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_scrolled".to_string(),
            self.module.add_function(
                "gui_scrolled",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_flowbox".to_string(),
            self.module.add_function(
                "gui_flowbox",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_flow_add".to_string(),
            self.module.add_function("gui_flow_add", fn_2, None),
        );
        self.functions.insert(
            "gui_frame".to_string(),
            self.module.add_function(
                "gui_frame",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_set_margin".to_string(),
            self.module.add_function("gui_set_margin", fn_2, None),
        );
        self.functions.insert(
            "gui_icon".to_string(),
            self.module.add_function("gui_icon", fn_2, None),
        );
        self.functions.insert(
            "gui_css".to_string(),
            self.module.add_function("gui_css", fn_1, None),
        );
        self.functions.insert(
            "gui_add_class".to_string(),
            self.module.add_function("gui_add_class", fn_2, None),
        );
        self.functions.insert(
            "gui_listbox".to_string(),
            self.module.add_function(
                "gui_listbox",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_list_add_text".to_string(),
            self.module.add_function("gui_list_add_text", fn_2, None),
        );
        self.functions.insert(
            "gui_on_select_ctx".to_string(),
            self.module.add_function("gui_on_select_ctx", fn_3, None),
        );
        self.functions.insert(
            "gui_set_child".to_string(),
            self.module.add_function("gui_set_child", fn_2, None),
        );
        self.functions.insert(
            "gui_add".to_string(),
            self.module.add_function("gui_add", fn_2, None),
        );
        self.functions.insert(
            "gui_add_expand".to_string(),
            self.module.add_function("gui_add_expand", fn_2, None),
        );
        self.functions.insert(
            "gui_label".to_string(),
            self.module.add_function("gui_label", fn_1, None),
        );
        self.functions.insert(
            "gui_entry".to_string(),
            self.module.add_function(
                "gui_entry",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_textview".to_string(),
            self.module.add_function(
                "gui_textview",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_set_placeholder".to_string(),
            self.module.add_function("gui_set_placeholder", fn_2, None),
        );
        self.functions.insert(
            "gui_set_editable".to_string(),
            self.module.add_function("gui_set_editable", fn_2, None),
        );
        self.functions.insert(
            "gui_button".to_string(),
            self.module.add_function("gui_button", fn_1, None),
        );
        self.functions.insert(
            "gui_set_enabled".to_string(),
            self.module.add_function("gui_set_enabled", fn_2, None),
        );
        self.functions.insert(
            "gui_set_visible".to_string(),
            self.module.add_function("gui_set_visible", fn_2, None),
        );
        self.functions.insert(
            "gui_show_all".to_string(),
            self.module.add_function("gui_show_all", fn_1, None),
        );
        self.functions.insert(
            "gui_set_text".to_string(),
            self.module.add_function("gui_set_text", fn_2, None),
        );
        self.functions.insert(
            "gui_get_text".to_string(),
            self.module.add_function("gui_get_text", fn_1, None),
        );
        self.functions.insert(
            "gui_on_click".to_string(),
            self.module.add_function("gui_on_click", fn_2, None),
        );
        self.functions.insert(
            "gui_on_click_ctx".to_string(),
            self.module.add_function("gui_on_click_ctx", fn_3, None),
        );
        self.functions.insert(
            "gui_on_tap_ctx".to_string(),
            self.module.add_function("gui_on_tap_ctx", fn_3, None),
        );
        self.functions.insert(
            "gui_separator_h".to_string(),
            self.module.add_function(
                "gui_separator_h",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_separator_v".to_string(),
            self.module.add_function(
                "gui_separator_v",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "gui_msg_info".to_string(),
            self.module.add_function("gui_msg_info", fn_2, None),
        );
        self.functions.insert(
            "gui_msg_error".to_string(),
            self.module.add_function("gui_msg_error", fn_2, None),
        );

        // Skia (experimental)
        self.functions.insert(
            "skia_version".to_string(),
            self.module.add_function(
                "skia_version",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "skia_use_real".to_string(),
            self.module.add_function(
                "skia_use_real",
                void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "skia_surface".to_string(),
            self.module.add_function(
                "skia_surface",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_surface_width".to_string(),
            self.module.add_function(
                "skia_surface_width",
                void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "skia_surface_height".to_string(),
            self.module.add_function(
                "skia_surface_height",
                void_type.fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "skia_surface_clear".to_string(),
            self.module.add_function(
                "skia_surface_clear",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_surface_set_color".to_string(),
            self.module.add_function(
                "skia_surface_set_color",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_draw_rect".to_string(),
            self.module.add_function(
                "skia_draw_rect",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_draw_circle".to_string(),
            self.module.add_function(
                "skia_draw_circle",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_draw_line".to_string(),
            self.module.add_function(
                "skia_draw_line",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_draw_text".to_string(),
            self.module.add_function(
                "skia_draw_text",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "skia_save_png".to_string(),
            self.module.add_function(
                "skia_save_png",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );

        self.functions.insert(
            "str_to_num".to_string(),
            self.module.add_function("str_to_num", fn_1, None),
        );
        self.functions.insert(
            "num_to_str".to_string(),
            self.module.add_function("num_to_str", fn_1, None),
        );
        self.functions.insert(
            "calc_eval".to_string(),
            self.module.add_function("calc_eval", fn_1, None),
        );

        // SQLite (optional at link time)
        self.functions.insert(
            "sqlite_open".to_string(),
            self.module.add_function("sqlite_open", fn_1, None),
        );
        self.functions.insert(
            "sqlite_close".to_string(),
            self.module.add_function("sqlite_close", fn_1, None),
        );
        self.functions.insert(
            "sqlite_exec".to_string(),
            self.module.add_function("sqlite_exec", fn_2, None),
        );
        self.functions.insert(
            "sqlite_query".to_string(),
            self.module.add_function("sqlite_query", fn_2, None),
        );
        self.functions.insert(
            "sqlite_prepare".to_string(),
            self.module.add_function("sqlite_prepare", fn_2, None),
        );
        self.functions.insert(
            "sqlite_finalize".to_string(),
            self.module.add_function("sqlite_finalize", fn_1, None),
        );
        self.functions.insert(
            "sqlite_reset".to_string(),
            self.module.add_function("sqlite_reset", fn_1, None),
        );
        self.functions.insert(
            "sqlite_bind_text".to_string(),
            self.module.add_function("sqlite_bind_text", fn_3, None),
        );
        self.functions.insert(
            "sqlite_bind_num".to_string(),
            self.module.add_function("sqlite_bind_num", fn_3, None),
        );
        self.functions.insert(
            "sqlite_bind_null".to_string(),
            self.module.add_function("sqlite_bind_null", fn_2, None),
        );
        self.functions.insert(
            "sqlite_step".to_string(),
            self.module.add_function("sqlite_step", fn_1, None),
        );
        self.functions.insert(
            "sqlite_column".to_string(),
            self.module.add_function("sqlite_column", fn_2, None),
        );
        self.functions.insert(
            "sqlite_column_count".to_string(),
            self.module.add_function("sqlite_column_count", fn_1, None),
        );
        self.functions.insert(
            "sqlite_column_name".to_string(),
            self.module.add_function("sqlite_column_name", fn_2, None),
        );

        // zlib via OM contract. Public Snask surface is zlib.compress/decompress only.
        self.functions.insert(
            "zlib_compress".to_string(),
            self.module.add_function("zlib_compress", fn_1, None),
        );
        self.functions.insert(
            "zlib_decompress".to_string(),
            self.module.add_function("zlib_decompress", fn_1, None),
        );

        // Biblioteca String (Nativas)
        self.functions.insert(
            "string_len".to_string(),
            self.module.add_function("string_len", fn_1, None),
        );
        self.functions.insert(
            "string_upper".to_string(),
            self.module.add_function("string_upper", fn_1, None),
        );
        self.functions.insert(
            "string_lower".to_string(),
            self.module.add_function("string_lower", fn_1, None),
        );
        self.functions.insert(
            "string_trim".to_string(),
            self.module.add_function("string_trim", fn_1, None),
        );
        self.functions.insert(
            "string_split".to_string(),
            self.module.add_function("string_split", fn_2, None),
        );
        self.functions.insert(
            "string_join".to_string(),
            self.module.add_function("string_join", fn_2, None),
        );
        self.functions.insert(
            "string_replace".to_string(),
            self.module.add_function("string_replace", fn_3, None),
        );
        self.functions.insert(
            "string_contains".to_string(),
            self.module.add_function("string_contains", fn_2, None),
        );
        self.functions.insert(
            "string_starts_with".to_string(),
            self.module.add_function("string_starts_with", fn_2, None),
        );
        self.functions.insert(
            "string_ends_with".to_string(),
            self.module.add_function("string_ends_with", fn_2, None),
        );
        self.functions.insert(
            "string_chars".to_string(),
            self.module.add_function("string_chars", fn_1, None),
        );
        self.functions.insert(
            "string_substring".to_string(),
            self.module.add_function("string_substring", fn_3, None),
        );
        self.functions.insert(
            "string_format".to_string(),
            self.module.add_function(
                "string_format",
                self.context.void_type().fn_type(
                    &[
                        self.ptr_type.into(),
                        self.context.i32_type().into(),
                        self.context.ptr_type(inkwell::AddressSpace::from(0)).into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "string_index_of".to_string(),
            self.module.add_function("string_index_of", fn_2, None),
        );
        self.functions.insert(
            "string_last_index_of".to_string(),
            self.module.add_function("string_last_index_of", fn_2, None),
        );
        self.functions.insert(
            "string_repeat".to_string(),
            self.module.add_function("string_repeat", fn_2, None),
        );
        self.functions.insert(
            "string_is_empty".to_string(),
            self.module.add_function("string_is_empty", fn_1, None),
        );
        self.functions.insert(
            "string_is_blank".to_string(),
            self.module.add_function("string_is_blank", fn_1, None),
        );
        self.functions.insert(
            "string_pad_start".to_string(),
            self.module.add_function("string_pad_start", fn_3, None),
        );
        self.functions.insert(
            "string_pad_end".to_string(),
            self.module.add_function("string_pad_end", fn_3, None),
        );
        self.functions.insert(
            "string_capitalize".to_string(),
            self.module.add_function("string_capitalize", fn_1, None),
        );
        self.functions.insert(
            "string_title".to_string(),
            self.module.add_function("string_title", fn_1, None),
        );
        self.functions.insert(
            "string_swapcase".to_string(),
            self.module.add_function("string_swapcase", fn_1, None),
        );
        self.functions.insert(
            "string_count".to_string(),
            self.module.add_function("string_count", fn_2, None),
        );
        self.functions.insert(
            "string_is_numeric".to_string(),
            self.module.add_function("string_is_numeric", fn_1, None),
        );
        self.functions.insert(
            "string_is_alpha".to_string(),
            self.module.add_function("string_is_alpha", fn_1, None),
        );
        self.functions.insert(
            "string_is_alphanumeric".to_string(),
            self.module
                .add_function("string_is_alphanumeric", fn_1, None),
        );
        self.functions.insert(
            "string_is_ascii".to_string(),
            self.module.add_function("string_is_ascii", fn_1, None),
        );
        self.functions.insert(
            "string_hex".to_string(),
            self.module.add_function("string_hex", fn_1, None),
        );
        self.functions.insert(
            "string_from_char_code".to_string(),
            self.module
                .add_function("string_from_char_code", fn_1, None),
        );
        self.functions.insert(
            "string_to_char_code".to_string(),
            self.module.add_function("string_to_char_code", fn_2, None),
        );
        self.functions.insert(
            "string_reverse".to_string(),
            self.module.add_function("string_reverse", fn_1, None),
        );

        // Multithreading (pthread)
        self.functions.insert(
            "thread_spawn".to_string(),
            self.module.add_function("thread_spawn", fn_2, None),
        );
        self.functions.insert(
            "thread_join".to_string(),
            self.module.add_function("thread_join", fn_1, None),
        );
        self.functions.insert(
            "thread_detach".to_string(),
            self.module.add_function("thread_detach", fn_1, None),
        );

        self.functions.insert(
            "s_alloc_obj".to_string(),
            self.module.add_function("s_alloc_obj", fn_alloc, None),
        );
        self.functions.insert(
            "s_json_stringify".to_string(),
            self.module.add_function("s_json_stringify", fn_1, None),
        );
        self.functions.insert(
            "json_stringify".to_string(),
            self.module.add_function("json_stringify", fn_1, None),
        );
        self.functions.insert(
            "json_stringify_pretty".to_string(),
            self.module
                .add_function("json_stringify_pretty", fn_1, None),
        );
        self.functions.insert(
            "json_parse".to_string(),
            self.module.add_function("json_parse", fn_1, None),
        );
        self.functions.insert(
            "json_get".to_string(),
            self.module.add_function("json_get", fn_2, None),
        );
        self.functions.insert(
            "snask_iter_get".to_string(),
            self.module.add_function("snask_iter_get", fn_2, None),
        );
        self.functions.insert(
            "json_set".to_string(),
            self.module.add_function("json_set", fn_3, None),
        );
        self.functions.insert(
            "json_keys".to_string(),
            self.module.add_function("json_keys", fn_1, None),
        );
        self.functions.insert(
            "json_parse_ex".to_string(),
            self.module.add_function("json_parse_ex", fn_1, None),
        );

        // SJSON
        self.functions.insert(
            "sjson_type".to_string(),
            self.module.add_function("sjson_type", fn_1, None),
        );
        self.functions.insert(
            "sjson_new_object".to_string(),
            self.module.add_function(
                "sjson_new_object",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "sjson_new_array".to_string(),
            self.module.add_function(
                "sjson_new_array",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "sjson_arr_len".to_string(),
            self.module.add_function("sjson_arr_len", fn_1, None),
        );
        self.functions.insert(
            "sjson_arr_get".to_string(),
            self.module.add_function("sjson_arr_get", fn_2, None),
        );
        self.functions.insert(
            "sjson_arr_set".to_string(),
            self.module.add_function("sjson_arr_set", fn_3, None),
        );
        self.functions.insert(
            "sjson_arr_push".to_string(),
            self.module.add_function("sjson_arr_push", fn_2, None),
        );
        self.functions.insert(
            "sjson_path_get".to_string(),
            self.module.add_function("sjson_path_get", fn_2, None),
        );

        self.functions.insert(
            "is_0".to_string(),
            self.module.add_function("is_0", fn_1, None),
        );

        self.functions.insert(
            "snif_new_object".to_string(),
            self.module.add_function(
                "snif_new_object",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "snif_new_array".to_string(),
            self.module.add_function(
                "snif_new_array",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "snif_parse_ex".to_string(),
            self.module.add_function("snif_parse_ex", fn_1, None),
        );
        self.functions.insert(
            "snif_type".to_string(),
            self.module.add_function("snif_type", fn_1, None),
        );
        self.functions.insert(
            "snif_arr_len".to_string(),
            self.module.add_function("snif_arr_len", fn_1, None),
        );
        self.functions.insert(
            "snif_arr_get".to_string(),
            self.module.add_function("snif_arr_get", fn_2, None),
        );
        self.functions.insert(
            "snif_arr_set".to_string(),
            self.module.add_function("snif_arr_set", fn_3, None),
        );
        self.functions.insert(
            "snif_arr_push".to_string(),
            self.module.add_function("snif_arr_push", fn_2, None),
        );
        self.functions.insert(
            "snif_path_get".to_string(),
            self.module.add_function("snif_path_get", fn_2, None),
        );
        self.functions.insert(
            "s_get_member".to_string(),
            self.module.add_function("s_get_member", fn_2, None),
        );
        self.functions.insert(
            "s_set_member".to_string(),
            self.module.add_function(
                "s_set_member",
                void_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );

        // OM Arena functions
        self.functions.insert(
            "s_arena_alloc_obj".to_string(),
            self.module
                .add_function("s_arena_alloc_obj", fn_alloc, None),
        );
        self.functions.insert(
            "s_arena_reset".to_string(),
            self.module
                .add_function("s_arena_reset", void_type.fn_type(&[], false), None),
        );
        self.functions.insert(
            "s_zone_enter".to_string(),
            self.module.add_function(
                "s_zone_enter",
                void_type.fn_type(&[self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "s_zone_leave".to_string(),
            self.module
                .add_function("s_zone_leave", void_type.fn_type(&[], false), None),
        );
        self.functions.insert(
            "s_zone_register".to_string(),
            self.module.add_function(
                "s_zone_register",
                self.ptr_type.fn_type(
                    &[
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                        self.ptr_type.into(),
                    ],
                    false,
                ),
                None,
            ),
        );
        self.functions.insert(
            "s_om_resource_ptr".to_string(),
            self.module.add_function(
                "s_om_resource_ptr",
                self.ptr_type
                    .fn_type(&[self.ptr_type.into(), self.ptr_type.into()], false),
                None,
            ),
        );
        self.functions.insert(
            "free".to_string(),
            self.module
                .get_function("s_free_obj")
                .unwrap_or_else(|| self.module.add_function("s_free_obj", fn_1, None)),
        );
        self.functions.insert(
            "__s_call_by_name".to_string(),
            self.module.add_function("s_call_by_name", fn_4, None),
        );

        // Aliases "__" para nativas que devem ser acessadas via bibliotecas.
        // O compilador reescreve chamadas dentro de módulos importados para usar "__<nome>".
        for n in [
            // Filesystem / Path / OS / HTTP
            "sfs_read",
            "sfs_write",
            "sfs_append",
            "sfs_write_mb",
            "sfs_count_bytes",
            "sfs_bench_create_small_files",
            "sfs_bench_count_entries",
            "sfs_bench_delete_small_files",
            "sfs_delete",
            "sfs_exists",
            "sfs_copy",
            "sfs_move",
            "sfs_mkdir",
            "sfs_is_file",
            "sfs_is_dir",
            "sfs_listdir",
            "sfs_size",
            "sfs_mtime",
            "sfs_rmdir",
            "path_basename",
            "path_dirname",
            "path_extname",
            "path_join",
            "os_cwd",
            "os_platform",
            "os_arch",
            "os_getenv",
            "os_setenv",
            "os_random_hex",
            "s_http_get",
            "s_http_post",
            "s_http_put",
            "s_http_delete",
            "s_http_patch",
            // Frameworks / GUI / DB / Threads
            "blaze_run",
            "blaze_qs_get",
            "blaze_cookie_get",
            "auth_random_hex",
            "auth_now",
            "auth_const_time_eq",
            "auth_hash_password",
            "auth_verify_password",
            "auth_session_id",
            "auth_csrf_token",
            "auth_cookie_kv",
            "auth_cookie_session",
            "auth_cookie_delete",
            "auth_bearer_header",
            "auth_ok",
            "auth_fail",
            "auth_version",
            "gui_init",
            "gui_run",
            "gui_quit",
            "gui_window",
            "gui_set_title",
            "gui_set_resizable",
            "gui_autosize",
            "gui_vbox",
            "gui_hbox",
            "gui_eventbox",
            "gui_scrolled",
            "gui_flowbox",
            "gui_flow_add",
            "gui_frame",
            "gui_set_margin",
            "gui_icon",
            "gui_css",
            "gui_add_class",
            "gui_listbox",
            "gui_list_add_text",
            "gui_on_select_ctx",
            "gui_set_child",
            "gui_add",
            "gui_add_expand",
            "gui_label",
            "gui_entry",
            "gui_set_placeholder",
            "gui_set_editable",
            "gui_button",
            "gui_set_enabled",
            "gui_set_visible",
            "gui_show_all",
            "gui_set_text",
            "gui_get_text",
            "gui_on_click",
            "gui_on_click_ctx",
            "gui_on_tap_ctx",
            "gui_separator_h",
            "gui_separator_v",
            "gui_msg_info",
            "gui_msg_error",
            "skia_version",
            "skia_use_real",
            "skia_surface",
            "skia_surface_width",
            "skia_surface_height",
            "skia_surface_clear",
            "skia_surface_set_color",
            "skia_draw_rect",
            "skia_draw_circle",
            "skia_draw_line",
            "skia_draw_text",
            "skia_save_png",
            "sqlite_open",
            "sqlite_close",
            "sqlite_exec",
            "sqlite_query",
            "sqlite_prepare",
            "sqlite_finalize",
            "sqlite_reset",
            "sqlite_bind_text",
            "sqlite_bind_num",
            "sqlite_bind_null",
            "sqlite_step",
            "sqlite_column",
            "sqlite_column_count",
            "sqlite_column_name",
            "thread_spawn",
            "thread_join",
            "thread_detach",
            // JSON / SNIF
            "json_stringify",
            "json_stringify_pretty",
            "json_parse",
            "json_get",
            "json_has",
            "json_len",
            "json_index",
            "json_set",
            "json_keys",
            "json_parse_ex",
            "snif_new_object",
            "snif_new_array",
            "snif_parse_ex",
            "snif_type",
            "snif_arr_len",
            "snif_arr_get",
            "snif_arr_set",
            "snif_arr_push",
            "snif_path_get",
            // String (Novas)
            "string_len",
            "string_upper",
            "string_lower",
            "string_trim",
            "string_split",
            "string_join",
            "string_replace",
            "string_contains",
            "string_starts_with",
            "string_ends_with",
            "string_chars",
            "string_substring",
            "string_format",
            "string_index_of",
            "string_last_index_of",
            "string_repeat",
            "string_is_empty",
            "string_is_blank",
            "string_pad_start",
            "string_pad_end",
            "string_capitalize",
            "string_title",
            "string_swapcase",
            "string_count",
            "string_is_numeric",
            "string_is_alpha",
            "string_is_alphanumeric",
            "string_is_ascii",
            "string_hex",
            "string_from_char_code",
            "string_to_char_code",
            "string_reverse",
        ] {
            if let Some(f) = self.module.get_function(n) {
                let alias = format!("__{}", n);
                if self.module.get_function(&alias).is_none() {
                    self.module.add_function(&alias, f.get_type(), None);
                }
            }
        }
    }

    fn sanitize_name(&self, name: &str) -> String {
        name.replace("::", "_NS_")
    }

    fn expr_path(expr: &Expr) -> Option<Vec<String>> {
        match &expr.kind {
            ExprKind::Variable(name) => Some(vec![name.clone()]),
            ExprKind::PropertyAccess { target, property } => {
                let mut parts = Self::expr_path(target)?;
                parts.push(property.clone());
                Some(parts)
            }
            _ => None,
        }
    }

    fn om_resource_for_surface(
        &self,
        library: &str,
        surface_type: &str,
    ) -> Result<&OmResourceContract, String> {
        let contract = self
            .om_contracts
            .get(library)
            .ok_or_else(|| format!("OM contract for library `{library}` was not loaded."))?;
        contract
            .resource_by_surface_type(surface_type)
            .ok_or_else(|| {
                format!(
                    "OM contract for `{library}` does not define surface type `{surface_type}`."
                )
            })
    }

    fn om_function_for_surface(
        &self,
        library: &str,
        surface: &str,
    ) -> Result<&OmFunctionContract, String> {
        let contract = self
            .om_contracts
            .get(library)
            .ok_or_else(|| format!("OM contract for library `{library}` was not loaded."))?;
        contract.function_by_surface(surface).ok_or_else(|| {
            format!("OM contract for `{library}` does not define surface function `{surface}`.")
        })
    }

    fn om_constant_for_surface(&self, library: &str, surface: &str) -> Option<i64> {
        self.om_contracts
            .get(library)?
            .constant_by_surface(surface)
            .map(|constant| constant.value)
    }

    fn ensure_om_function_exposed(&self, function: &OmFunctionContract) -> Result<(), String> {
        match function.safety.as_deref() {
            Some("SAFE") | Some("COPY_ONLY") => Ok(()),
            Some("BLOCKED") => Err(format!(
                "OM import blocked `{}`: {}",
                function.surface,
                function
                    .reason
                    .as_deref()
                    .unwrap_or("the scanner could not prove this API is safe")
            )),
            Some(other) => Err(format!(
                "OM contract for `{}` has unknown safety `{}`.",
                function.surface, other
            )),
            None => Ok(()),
        }
    }

    fn om_resource_for_constructor(
        &self,
        library: &str,
        constructor: &str,
    ) -> Option<&OmResourceContract> {
        self.om_contracts
            .get(library)?
            .resources
            .iter()
            .find(|resource| resource.constructor == constructor)
    }

    fn build_string_value(&self, value: &str, global_name: &str) -> StructValue<'ctx> {
        let g = self
            .builder
            .build_global_string_ptr(value, global_name)
            .unwrap();
        let mut s = self.value_type.get_undef();
        s = self
            .builder
            .build_insert_value(
                s,
                self.context.f64_type().const_float(TYPE_STR as f64),
                0,
                "str_t",
            )
            .unwrap()
            .into_struct_value();
        s = self
            .builder
            .build_insert_value(s, self.context.f64_type().const_float(0.0), 1, "str_v")
            .unwrap()
            .into_struct_value();
        self.builder
            .build_insert_value(s, g.as_pointer_value(), 2, "str_p")
            .unwrap()
            .into_struct_value()
    }

    fn c_type_to_llvm(&self, c_type: &str) -> Option<BasicTypeEnum<'ctx>> {
        let ty = c_type.trim();
        if ty.contains('*') {
            return Some(self.ptr_type.into());
        }

        match ty {
            "char" | "unsigned char" | "Uint8" | "Sint8" => Some(self.context.i8_type().into()),
            "short" | "unsigned short" | "Uint16" | "Sint16" => {
                Some(self.context.i16_type().into())
            }
            "int" | "unsigned int" | "signed int" | "Sint32" | "Uint32" | "SDL_bool" => {
                Some(self.context.i32_type().into())
            }
            "long" | "unsigned long" | "long long" | "unsigned long long" | "size_t" | "Sint64"
            | "Uint64" => Some(self.context.i64_type().into()),
            "float" => Some(self.context.f32_type().into()),
            "double" => Some(self.context.f64_type().into()),
            _ => None,
        }
    }

    fn declare_c_function(
        &self,
        contract_fn: &OmFunctionContract,
    ) -> Result<FunctionValue<'ctx>, String> {
        if let Some(existing) = self.module.get_function(&contract_fn.c_function) {
            return Ok(existing);
        }

        let param_types: Vec<BasicMetadataTypeEnum> = contract_fn
            .c_param_types
            .iter()
            .map(|ty| {
                self.c_type_to_llvm(ty)
                    .map(Into::into)
                    .ok_or_else(|| format!("OM cannot map C parameter type `{ty}` yet."))
            })
            .collect::<Result<_, _>>()?;

        let return_ty = contract_fn
            .c_return_type
            .as_deref()
            .unwrap_or("void")
            .trim();

        let fn_type = if return_ty == "void" {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            self.c_type_to_llvm(return_ty)
                .ok_or_else(|| format!("OM cannot map C return type `{return_ty}` yet."))?
                .fn_type(&param_types, false)
        };

        Ok(self
            .module
            .add_function(&contract_fn.c_function, fn_type, None))
    }

    fn declare_c_destructor(&self, name: &str) -> FunctionValue<'ctx> {
        self.module.get_function(name).unwrap_or_else(|| {
            self.module.add_function(
                name,
                self.context
                    .void_type()
                    .fn_type(&[self.ptr_type.into()], false),
                None,
            )
        })
    }

    fn emit_sdl_poll_event_type(
        &self,
    ) -> Result<(BasicValueEnum<'ctx>, crate::types::Type), String> {
        let poll_fn = self
            .module
            .get_function("SDL_PollEvent")
            .unwrap_or_else(|| {
                self.module.add_function(
                    "SDL_PollEvent",
                    self.context
                        .i32_type()
                        .fn_type(&[self.ptr_type.into()], false),
                    None,
                )
            });
        let event_storage = self
            .create_entry_block_alloca(self.context.i8_type().array_type(64), "sdl_event_storage");
        let event_ptr = self
            .builder
            .build_pointer_cast(event_storage, self.ptr_type, "sdl_event_ptr")
            .unwrap();
        let has_event = self
            .builder
            .build_call(poll_fn, &[event_ptr.into()], "sdl_poll_event")
            .unwrap()
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "SDL_PollEvent unexpectedly returned void.".to_string())?
            .into_int_value();
        let event_type_ptr = self
            .builder
            .build_pointer_cast(
                event_ptr,
                self.context.ptr_type(inkwell::AddressSpace::from(0)),
                "sdl_event_type_ptr",
            )
            .unwrap();
        let raw_event_type = self
            .builder
            .build_load(self.context.i32_type(), event_type_ptr, "sdl_event_type")
            .unwrap()
            .into_int_value();
        let has_event_bool = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                has_event,
                self.context.i32_type().const_int(0, false),
                "sdl_has_event",
            )
            .unwrap();
        let selected = self
            .builder
            .build_select(
                has_event_bool,
                raw_event_type,
                self.context.i32_type().const_int(0, false),
                "sdl_event_or_none",
            )
            .unwrap()
            .into_int_value();
        Ok((
            self.builder
                .build_int_z_extend(selected, self.i64_type, "sdl_event_i64")
                .unwrap()
                .into(),
            crate::types::Type::Int,
        ))
    }

    fn convert_to_c_arg(
        &self,
        value: BasicValueEnum<'ctx>,
        value_ty: crate::types::Type,
        c_type: &str,
    ) -> Result<BasicMetadataValueEnum<'ctx>, String> {
        let target = self
            .c_type_to_llvm(c_type)
            .ok_or_else(|| format!("OM cannot map C parameter type `{c_type}` yet."))?;

        if c_type.contains('*') {
            let ptr = match value_ty {
                crate::types::Type::String
                | crate::types::Type::Ptr
                | crate::types::Type::User(_) => value.into_pointer_value(),
                crate::types::Type::Any => {
                    let value_ptr = self.create_entry_block_alloca(self.value_type, "om_res_arg");
                    self.builder
                        .build_store(value_ptr, value.into_struct_value())
                        .unwrap();
                    let resource_ptr_fn = *self.functions.get("s_om_resource_ptr").unwrap();
                    self.builder
                        .build_call(
                            resource_ptr_fn,
                            &[value_ptr.into(), self.ptr_type.const_null().into()],
                            "om_resource_ptr",
                        )
                        .unwrap()
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_pointer_value()
                }
                other => {
                    return Err(format!(
                        "OM cannot pass Snask type `{:?}` to C pointer `{}` safely.",
                        other, c_type
                    ))
                }
            };
            return Ok(self
                .builder
                .build_pointer_cast(ptr, target.into_pointer_type(), "om_ptr_cast")
                .unwrap()
                .into());
        }

        if target.is_float_type() {
            let out = match value_ty {
                crate::types::Type::Float => {
                    let f = value.into_float_value();
                    if target.into_float_type() == self.context.f32_type() {
                        self.builder
                            .build_float_cast(f, self.context.f32_type(), "om_f32_arg")
                            .unwrap()
                    } else {
                        f
                    }
                }
                crate::types::Type::Int
                | crate::types::Type::I32
                | crate::types::Type::I64
                | crate::types::Type::U8
                | crate::types::Type::Bool => self
                    .builder
                    .build_signed_int_to_float(
                        value.into_int_value(),
                        target.into_float_type(),
                        "om_int_to_float_arg",
                    )
                    .unwrap(),
                crate::types::Type::Any => {
                    let boxed = value.into_struct_value();
                    let f = self
                        .builder
                        .build_extract_value(boxed, 1, "om_num_arg")
                        .unwrap()
                        .into_float_value();
                    self.builder
                        .build_float_cast(f, target.into_float_type(), "om_any_float_arg")
                        .unwrap()
                }
                other => {
                    return Err(format!(
                        "OM cannot pass Snask type `{:?}` to C numeric `{}`.",
                        other, c_type
                    ))
                }
            };
            return Ok(out.into());
        }

        let int_ty = target.into_int_type();
        let out = match value_ty {
            crate::types::Type::Bool
            | crate::types::Type::Int
            | crate::types::Type::I32
            | crate::types::Type::I64
            | crate::types::Type::U8 => self
                .builder
                .build_int_cast(value.into_int_value(), int_ty, "om_int_arg")
                .unwrap(),
            crate::types::Type::Float => self
                .builder
                .build_float_to_signed_int(value.into_float_value(), int_ty, "om_float_int_arg")
                .unwrap(),
            crate::types::Type::Any => {
                let boxed = value.into_struct_value();
                let f = self
                    .builder
                    .build_extract_value(boxed, 1, "om_any_num_arg")
                    .unwrap()
                    .into_float_value();
                self.builder
                    .build_float_to_signed_int(f, int_ty, "om_any_int_arg")
                    .unwrap()
            }
            other => {
                return Err(format!(
                    "OM cannot pass Snask type `{:?}` to C integer `{}`.",
                    other, c_type
                ))
            }
        };
        Ok(out.into())
    }

    fn register_om_resource(
        &self,
        c_ptr: PointerValue<'ctx>,
        resource: &OmResourceContract,
    ) -> Result<StructValue<'ctx>, String> {
        let register_fn = *self
            .functions
            .get("s_zone_register")
            .ok_or_else(|| "OM runtime function `s_zone_register` is not declared.".to_string())?;
        let destructor = self.declare_c_destructor(&resource.destructor);
        let destructor_ptr = self
            .builder
            .build_pointer_cast(
                destructor.as_global_value().as_pointer_value(),
                self.ptr_type,
                "om_destructor_ptr",
            )
            .unwrap();
        let type_name = self
            .builder
            .build_global_string_ptr(&resource.surface_type, "om_resource_type")
            .unwrap();
        let handle = self
            .builder
            .build_call(
                register_fn,
                &[
                    c_ptr.into(),
                    destructor_ptr.into(),
                    type_name.as_pointer_value().into(),
                ],
                "om_register_resource",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "OM resource registration returned void.".to_string())?
            .into_pointer_value();

        let mut s = self.value_type.get_undef();
        s = self
            .builder
            .build_insert_value(
                s,
                self.context.f64_type().const_float(TYPE_RESOURCE as f64),
                0,
                "om_resource_tag",
            )
            .unwrap()
            .into_struct_value();
        s = self
            .builder
            .build_insert_value(
                s,
                self.context.f64_type().const_float(0.0),
                1,
                "om_resource_num",
            )
            .unwrap()
            .into_struct_value();
        Ok(self
            .builder
            .build_insert_value(s, handle, 2, "om_resource_handle")
            .unwrap()
            .into_struct_value())
    }

    fn c_return_to_snask(
        &self,
        value: Option<BasicValueEnum<'ctx>>,
        c_type: &str,
    ) -> Result<(BasicValueEnum<'ctx>, crate::types::Type), String> {
        let ty = c_type.trim();
        if ty == "void" {
            return Ok((self.ptr_type.const_null().into(), crate::types::Type::Void));
        }

        let value = value.ok_or_else(|| format!("C function returned no value for `{ty}`."))?;
        if ty.contains('*') {
            return Ok((value, crate::types::Type::Ptr));
        }
        if matches!(ty, "float" | "double") {
            let f = value.into_float_value();
            let f = if ty == "float" {
                self.builder
                    .build_float_cast(f, self.f64_type, "om_ret_f64")
                    .unwrap()
            } else {
                f
            };
            return Ok((f.into(), crate::types::Type::Float));
        }
        Ok((
            self.builder
                .build_int_cast(value.into_int_value(), self.i64_type, "om_ret_i64")
                .unwrap()
                .into(),
            crate::types::Type::Int,
        ))
    }

    fn emit_zone_leave(&self) {
        if let Some(leave_fn) = self.functions.get("s_zone_leave") {
            self.builder
                .build_call(*leave_fn, &[], "zone_leave")
                .unwrap();
        }
        if let Some(reset_fn) = self.functions.get("s_arena_reset") {
            self.builder
                .build_call(*reset_fn, &[], "arena_reset")
                .unwrap();
        }
    }

    fn emit_active_zone_cleanups(&self) {
        for _ in 0..self.active_zone_depth {
            self.emit_zone_leave();
        }
    }

    fn collect_class_properties(&self, class_name: &str) -> Result<Vec<VarDecl>, String> {
        let class = self
            .classes
            .get(class_name)
            .cloned()
            .ok_or_else(|| format!("Classe '{}' não encontrada.", class_name))?;
        let mut properties = if let Some(parent) = class.parent.clone() {
            self.collect_class_properties(&parent)?
        } else {
            Vec::new()
        };
        for prop in class.properties {
            if let Some(existing) = properties.iter_mut().find(|p| p.name == prop.name) {
                *existing = prop;
            } else {
                properties.push(prop);
            }
        }
        Ok(properties)
    }

    fn build_class_names_arg(
        &self,
        class_name: &str,
        properties: &[VarDecl],
    ) -> Result<PointerValue<'ctx>, String> {
        if properties.is_empty() {
            return Ok(self.ptr_type.const_null());
        }

        let global_name = format!("__snask_class_names_{}", self.sanitize_name(class_name));
        let names_global = if let Some(g) = self.module.get_global(&global_name) {
            g
        } else {
            let i8_ptr = self.context.ptr_type(inkwell::AddressSpace::from(0));
            let arr_ty = i8_ptr.array_type(properties.len() as u32);
            let g = self.module.add_global(arr_ty, None, &global_name);
            let mut elems = Vec::new();
            for prop in properties {
                let sp = self
                    .builder
                    .build_global_string_ptr(&prop.name, "prop_name")
                    .unwrap();
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
                .build_in_bounds_gep(
                    arr_ty,
                    names_global.as_pointer_value(),
                    &[zero, zero],
                    "names_ptr",
                )
                .unwrap()
        };
        Ok(self
            .builder
            .build_pointer_cast(names_ptr, self.ptr_type, "names_voidp")
            .unwrap())
    }

    fn declare_function(&mut self, func: &FuncDecl) -> Result<(), String> {
        let mut p_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![self.ptr_type.into()];
        for _ in &func.params {
            p_types.push(self.ptr_type.into());
        }
        let f_name = format!("f_{}", self.sanitize_name(&func.name));
        let function = self.module.add_function(
            &f_name,
            self.context.void_type().fn_type(&p_types, false),
            None,
        );
        self.functions.insert(func.name.clone(), function);
        Ok(())
    }

    fn generate_function_body(&mut self, func: FuncDecl) -> Result<(), String> {
        let function = *self.functions.get(&func.name).unwrap();
        let entry = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry);
        self.current_func = Some(function);
        let old_vars = self.variables.clone();
        let old_zone_depth = self.active_zone_depth;
        self.active_zone_depth = 0;
        self.local_vars.clear();
        let r_ptr = function.get_nth_param(0).unwrap().into_pointer_value();

        // Parâmetros começam no índice 1, porque o 0 é o RA (Return Address/Pointer)
        for (i, (name, param_ty)) in func.params.iter().enumerate() {
            let p_ptr = function
                .get_nth_param((i + 1) as u32)
                .unwrap()
                .into_pointer_value();
            self.local_vars
                .insert(name.clone(), (p_ptr, param_ty.clone()));
        }
        for stmt in func.body {
            self.generate_statement(stmt)?;
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some()
            {
                break;
            }
        }
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            let mut s = self.value_type.get_undef();
            s = self
                .builder
                .build_insert_value(
                    s,
                    self.context.f64_type().const_float(TYPE_NIL as f64),
                    0,
                    "t",
                )
                .unwrap()
                .into_struct_value();
            self.builder.build_store(r_ptr, s).unwrap();
            self.builder.build_return(None).unwrap();
        }
        self.local_vars.clear();
        self.variables = old_vars;
        self.active_zone_depth = old_zone_depth;
        Ok(())
    }

    fn generate_statement(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt.kind {
            StmtKind::VarDeclaration(d) => {
                let (v, ty) = self.evaluate_expression(d.value)?;
                let actual_ty = d.var_type.clone().unwrap_or(ty.clone());
                let llvm_ty = self.snask_type_to_llvm(&actual_ty);
                let stored_v = self.cast_basic_value(v, ty, &actual_ty);

                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" {
                    let gv = self
                        .module
                        .add_global(llvm_ty, None, &format!("g_{}", d.name));
                    gv.set_initializer(&llvm_ty.const_zero());
                    let p = gv.as_pointer_value();
                    self.builder.build_store(p, stored_v).unwrap();
                    self.variables.insert(d.name, (p, actual_ty));
                } else {
                    let a = self.create_entry_block_alloca(llvm_ty, &d.name);
                    self.builder.build_store(a, stored_v).unwrap();
                    self.local_vars.insert(d.name, (a, actual_ty));
                }
            }
            StmtKind::MutDeclaration(d) => {
                let (v, ty) = self.evaluate_expression(d.value)?;
                let actual_ty = d.var_type.clone().unwrap_or(ty.clone());
                let llvm_ty = self.snask_type_to_llvm(&actual_ty);
                let stored_v = self.cast_basic_value(v, ty, &actual_ty);

                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" {
                    let gv = self
                        .module
                        .add_global(llvm_ty, None, &format!("g_{}", d.name));
                    gv.set_initializer(&llvm_ty.const_zero());
                    let p = gv.as_pointer_value();
                    self.builder.build_store(p, stored_v).unwrap();
                    self.variables.insert(d.name, (p, actual_ty));
                } else {
                    let a = self.create_entry_block_alloca(llvm_ty, &d.name);
                    self.builder.build_store(a, stored_v).unwrap();
                    self.local_vars.insert(d.name, (a, actual_ty));
                }
            }
            StmtKind::ConstDeclaration(d) => {
                let (v, ty) = self.evaluate_expression(d.value)?;
                let actual_ty = d.var_type.clone().unwrap_or(ty.clone());
                let llvm_ty = self.snask_type_to_llvm(&actual_ty);
                let stored_v = self.cast_basic_value(v, ty, &actual_ty);

                if self.current_func.unwrap().get_name().to_str().unwrap() == "main" {
                    let gv = self
                        .module
                        .add_global(llvm_ty, None, &format!("g_{}", d.name));
                    gv.set_initializer(&llvm_ty.const_zero());
                    let p = gv.as_pointer_value();
                    self.builder.build_store(p, stored_v).unwrap();
                    self.variables.insert(d.name, (p, actual_ty));
                } else {
                    let a = self.create_entry_block_alloca(llvm_ty, &d.name);
                    self.builder.build_store(a, stored_v).unwrap();
                    self.local_vars.insert(d.name, (a, actual_ty));
                }
            }
            StmtKind::VarAssignment(s) => {
                let (v, ty) = self.evaluate_expression(s.value)?;
                let (p, target_ty) = self
                    .local_vars
                    .get(&s.name)
                    .or_else(|| self.variables.get(&s.name))
                    .ok_or_else(|| format!("Var {} not found.", s.name))?;
                let stored_v = self.cast_basic_value(v, ty, target_ty);
                self.builder.build_store(*p, stored_v).unwrap();
            }
            StmtKind::PropertyAssignment(p) => {
                let (obj, obj_ty) = self.evaluate_expression(p.target)?;
                let (val, val_ty) = self.evaluate_expression(p.value)?;

                let obj_boxed = self.box_value(obj, obj_ty);
                let val_boxed = self.box_value(val, val_ty);
                let key_boxed = self.box_value(
                    self.builder
                        .build_global_string_ptr(&p.property, "prop_key")
                        .unwrap()
                        .as_pointer_value()
                        .into(),
                    crate::types::Type::String,
                );

                let set_f = self.functions.get("json_set").unwrap();

                let obj_p = self.create_entry_block_alloca(self.value_type, "objp");
                self.builder.build_store(obj_p, obj_boxed).unwrap();
                let idx_p = self.create_entry_block_alloca(self.value_type, "prop_key_ptr");
                self.builder.build_store(idx_p, key_boxed).unwrap();
                let val_p = self.create_entry_block_alloca(self.value_type, "valp");
                self.builder.build_store(val_p, val_boxed).unwrap();
                let out_p = self.create_entry_block_alloca(self.value_type, "prop_set_out");

                self.builder
                    .build_call(
                        *set_f,
                        &[out_p.into(), obj_p.into(), idx_p.into(), val_p.into()],
                        "set",
                    )
                    .unwrap();
            }
            StmtKind::IndexAssignment(i) => {
                let (obj, obj_ty) = self.evaluate_expression(i.target)?;
                let (idx, idx_ty) = self.evaluate_expression(i.index)?;
                let (val, val_ty) = self.evaluate_expression(i.value)?;

                let obj_boxed = self.box_value(obj, obj_ty);
                let idx_boxed = self.box_value(idx, idx_ty);
                let val_boxed = self.box_value(val, val_ty);

                let set_f = self.functions.get("json_set").unwrap();

                let obj_p = self.create_entry_block_alloca(self.value_type, "objp");
                self.builder.build_store(obj_p, obj_boxed).unwrap();
                let idx_p = self.create_entry_block_alloca(self.value_type, "idxp");
                self.builder.build_store(idx_p, idx_boxed).unwrap();
                let val_p = self.create_entry_block_alloca(self.value_type, "valp");
                self.builder.build_store(val_p, val_boxed).unwrap();

                let out_p = self.create_entry_block_alloca(self.value_type, "outp");
                self.builder
                    .build_call(
                        *set_f,
                        &[out_p.into(), obj_p.into(), idx_p.into(), val_p.into()],
                        "idx_set",
                    )
                    .unwrap();
            }
            StmtKind::Print(exprs) => {
                let p_func = self.module.get_function("s_print").unwrap();
                let nl_func = self.module.get_function("s_println").unwrap();
                for expr in exprs {
                    let (v, ty) = self.evaluate_expression(expr)?;
                    let boxed = self.box_value(v, ty);
                    let v_ptr = self.create_entry_block_alloca(self.value_type, "pv");
                    self.builder.build_store(v_ptr, boxed).unwrap();
                    self.builder
                        .build_call(p_func, &[v_ptr.into()], "c")
                        .unwrap();
                }
                self.builder.build_call(nl_func, &[], "nl").unwrap();
            }
            StmtKind::Return(expr) => {
                let (v, ty) = self.evaluate_expression(expr)?;
                if self.current_func.unwrap().get_name().to_str().unwrap() != "main" {
                    let op = self
                        .current_func
                        .unwrap()
                        .get_nth_param(0)
                        .unwrap()
                        .into_pointer_value();
                    let boxed = self.box_value(v, ty);
                    self.builder.build_store(op, boxed).unwrap();
                    self.emit_active_zone_cleanups();
                    self.builder.build_return(None).unwrap();
                } else {
                    self.emit_active_zone_cleanups();
                    let i32_type = self.context.i32_type();
                    self.builder
                        .build_return(Some(&i32_type.const_int(0, false)))
                        .unwrap();
                }
            }
            StmtKind::Conditional(c) => {
                let parent = self.current_func.unwrap();
                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let merge_bb = self.context.append_basic_block(parent, "merge");

                let (cond_val, cond_ty) = self.evaluate_expression(c.if_block.condition)?;
                let is_true = match cond_ty {
                    crate::types::Type::Bool => cond_val.into_int_value(),
                    crate::types::Type::Float => self
                        .builder
                        .build_float_compare(
                            inkwell::FloatPredicate::ONE,
                            cond_val.into_float_value(),
                            self.f64_type.const_float(0.0),
                            "is_true",
                        )
                        .unwrap(),
                    _ => {
                        let boxed = self.box_value(cond_val, cond_ty);
                        let n = self
                            .builder
                            .build_extract_value(boxed, 1, "n")
                            .unwrap()
                            .into_float_value();
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                n,
                                self.f64_type.const_float(0.0),
                                "is_true",
                            )
                            .unwrap()
                    }
                };

                self.builder
                    .build_conditional_branch(is_true, then_bb, else_bb)
                    .unwrap();

                // THEN block
                self.builder.position_at_end(then_bb);
                for s in c.if_block.body {
                    self.generate_statement(s)?;
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                // ELSE block
                self.builder.position_at_end(else_bb);
                if let Some(else_body) = c.else_block {
                    for s in else_body {
                        self.generate_statement(s)?;
                    }
                }
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.builder.build_unconditional_branch(merge_bb).unwrap();
                }

                self.builder.position_at_end(merge_bb);
            }
            StmtKind::Loop(l) => match l {
                crate::ast::LoopStmt::While { condition, body } => {
                    let parent = self.current_func.unwrap();
                    let cond_bb = self.context.append_basic_block(parent, "while_cond");
                    let body_bb = self.context.append_basic_block(parent, "while_body");
                    let end_bb = self.context.append_basic_block(parent, "while_end");

                    self.builder.build_unconditional_branch(cond_bb).unwrap();

                    self.builder.position_at_end(cond_bb);
                    let (cond_val, cond_ty) = self.evaluate_expression(condition)?;
                    let is_true = match cond_ty {
                        crate::types::Type::Bool => cond_val.into_int_value(),
                        _ => {
                            let boxed = self.box_value(cond_val, cond_ty);
                            let n = self
                                .builder
                                .build_extract_value(boxed, 1, "wn")
                                .unwrap()
                                .into_float_value();
                            self.builder
                                .build_float_compare(
                                    inkwell::FloatPredicate::ONE,
                                    n,
                                    self.context.f64_type().const_float(0.0),
                                    "wtrue",
                                )
                                .unwrap()
                        }
                    };
                    self.builder
                        .build_conditional_branch(is_true, body_bb, end_bb)
                        .unwrap();

                    self.builder.position_at_end(body_bb);
                    for s in body {
                        self.generate_statement(s)?;
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }
                    }
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_none()
                    {
                        self.builder.build_unconditional_branch(cond_bb).unwrap();
                    }

                    self.builder.position_at_end(end_bb);
                }
                crate::ast::LoopStmt::For {
                    iterator,
                    iterable,
                    body,
                } => {
                    let parent = self.current_func.unwrap();
                    let cond_bb = self.context.append_basic_block(parent, "for_cond");
                    let body_bb = self.context.append_basic_block(parent, "for_body");
                    let step_bb = self.context.append_basic_block(parent, "for_step");
                    let end_bb = self.context.append_basic_block(parent, "for_end");

                    let (iterable_raw, iterable_ty) = self.evaluate_expression(iterable)?;
                    let iterable_val = self.box_value(iterable_raw, iterable_ty);
                    let iterable_ptr =
                        self.create_entry_block_alloca(self.value_type, "for_iterable");
                    self.builder
                        .build_store(iterable_ptr, iterable_val)
                        .unwrap();

                    let index_ptr = self.create_entry_block_alloca(self.value_type, "for_index");
                    let mut zero = self.value_type.get_undef();
                    zero = self
                        .builder
                        .build_insert_value(
                            zero,
                            self.context.f64_type().const_float(TYPE_NUM as f64),
                            0,
                            "for_idx_t",
                        )
                        .unwrap()
                        .into_struct_value();
                    zero = self
                        .builder
                        .build_insert_value(
                            zero,
                            self.context.f64_type().const_float(0.0),
                            1,
                            "for_idx_v",
                        )
                        .unwrap()
                        .into_struct_value();
                    zero = self
                        .builder
                        .build_insert_value(zero, self.ptr_type.const_null(), 2, "for_idx_p")
                        .unwrap()
                        .into_struct_value();
                    self.builder.build_store(index_ptr, zero).unwrap();

                    let iter_ptr =
                        self.create_entry_block_alloca(self.value_type, iterator.as_str());
                    let previous_iter = self
                        .local_vars
                        .insert(iterator.clone(), (iter_ptr, crate::types::Type::Any));

                    self.builder.build_unconditional_branch(cond_bb).unwrap();

                    self.builder.position_at_end(cond_bb);
                    let len_f = *self.functions.get("s_len").unwrap();
                    let len_out_ptr = self.create_entry_block_alloca(self.value_type, "for_len");
                    self.builder
                        .build_call(
                            len_f,
                            &[len_out_ptr.into(), iterable_ptr.into()],
                            "for_len_call",
                        )
                        .unwrap();
                    let len_val = self
                        .builder
                        .build_load(self.value_type, len_out_ptr, "for_len_value")
                        .unwrap()
                        .into_struct_value();
                    let len_num = self
                        .builder
                        .build_extract_value(len_val, 1, "for_len_num")
                        .unwrap()
                        .into_float_value();

                    let index_val = self
                        .builder
                        .build_load(self.value_type, index_ptr, "for_index_value")
                        .unwrap()
                        .into_struct_value();
                    let index_num = self
                        .builder
                        .build_extract_value(index_val, 1, "for_index_num")
                        .unwrap()
                        .into_float_value();
                    let has_more = self
                        .builder
                        .build_float_compare(
                            inkwell::FloatPredicate::OLT,
                            index_num,
                            len_num,
                            "for_has_more",
                        )
                        .unwrap();
                    self.builder
                        .build_conditional_branch(has_more, body_bb, end_bb)
                        .unwrap();

                    self.builder.position_at_end(body_bb);
                    let get_f = *self.functions.get("snask_iter_get").unwrap();
                    let item_out_ptr = self.create_entry_block_alloca(self.value_type, "for_item");
                    self.builder
                        .build_call(
                            get_f,
                            &[item_out_ptr.into(), iterable_ptr.into(), index_ptr.into()],
                            "for_get_item",
                        )
                        .unwrap();
                    let item_val = self
                        .builder
                        .build_load(self.value_type, item_out_ptr, "for_item_value")
                        .unwrap()
                        .into_struct_value();
                    self.builder.build_store(iter_ptr, item_val).unwrap();

                    for s in body {
                        self.generate_statement(s)?;
                        if self
                            .builder
                            .get_insert_block()
                            .unwrap()
                            .get_terminator()
                            .is_some()
                        {
                            break;
                        }
                    }
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_none()
                    {
                        self.builder.build_unconditional_branch(step_bb).unwrap();
                    }

                    self.builder.position_at_end(step_bb);
                    let curr_index = self
                        .builder
                        .build_load(self.value_type, index_ptr, "for_index_step")
                        .unwrap()
                        .into_struct_value();
                    let curr_num = self
                        .builder
                        .build_extract_value(curr_index, 1, "for_index_step_num")
                        .unwrap()
                        .into_float_value();
                    let next_num = self
                        .builder
                        .build_float_add(
                            curr_num,
                            self.context.f64_type().const_float(1.0),
                            "for_index_next",
                        )
                        .unwrap();
                    let mut next_index = self.value_type.get_undef();
                    next_index = self
                        .builder
                        .build_insert_value(
                            next_index,
                            self.context.f64_type().const_float(TYPE_NUM as f64),
                            0,
                            "for_next_t",
                        )
                        .unwrap()
                        .into_struct_value();
                    next_index = self
                        .builder
                        .build_insert_value(next_index, next_num, 1, "for_next_v")
                        .unwrap()
                        .into_struct_value();
                    next_index = self
                        .builder
                        .build_insert_value(next_index, self.ptr_type.const_null(), 2, "for_next_p")
                        .unwrap()
                        .into_struct_value();
                    self.builder.build_store(index_ptr, next_index).unwrap();
                    self.builder.build_unconditional_branch(cond_bb).unwrap();

                    self.builder.position_at_end(end_bb);
                    if let Some(prev) = previous_iter {
                        self.local_vars.insert(iterator, prev);
                    } else {
                        self.local_vars.remove(&iterator);
                    }
                }
            },
            StmtKind::FuncCall(expr) => {
                self.evaluate_expression(expr)?;
            }
            StmtKind::Expression(expr) => {
                self.evaluate_expression(expr)?;
            }
            StmtKind::ClassDeclaration(_) => {
                // TODO: Implement LLVM class generation
            }
            StmtKind::UnsafeBlock(body) => {
                // @unsafe is logically transparent but conceptually important here
                for s in body {
                    self.generate_statement(s)?;
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        break;
                    }
                }
            }
            StmtKind::Zone { name, body } => {
                if let Some(enter_fn) = self.functions.get("s_zone_enter") {
                    let zone_name = self
                        .builder
                        .build_global_string_ptr(&name, "zone_name")
                        .unwrap();
                    self.builder
                        .build_call(
                            *enter_fn,
                            &[zone_name.as_pointer_value().into()],
                            "zone_enter",
                        )
                        .unwrap();
                }
                self.active_zone_depth += 1;
                for s in body {
                    self.generate_statement(s.clone())?;
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        break;
                    }
                }
                self.active_zone_depth -= 1;
                if self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    self.emit_zone_leave();
                }
            }
            StmtKind::Scope { name: _, body } => {
                // Similar to zones
                for s in body {
                    self.generate_statement(s)?;
                    if self
                        .builder
                        .get_insert_block()
                        .unwrap()
                        .get_terminator()
                        .is_some()
                    {
                        break;
                    }
                }
            }
            StmtKind::Promote { target, .. } => {
                let (p, _) = self
                    .local_vars
                    .get(&target)
                    .or_else(|| self.variables.get(&target))
                    .ok_or_else(|| format!("Var {} not found for promotion.", target))?;
                let v = self
                    .builder
                    .build_load(self.value_type, *p, "to_promote")
                    .unwrap()
                    .into_struct_value();

                let v_ptr = self.create_entry_block_alloca(self.value_type, "v_ptr");
                self.builder.build_store(v_ptr, v).unwrap();

                let out_ptr = self.create_entry_block_alloca(self.value_type, "promoted_out");
                let f_promote = self.functions.get("s_promote").unwrap();
                self.builder
                    .build_call(*f_promote, &[out_ptr.into(), v_ptr.into()], "call_promote")
                    .unwrap();

                let promoted_v = self
                    .builder
                    .build_load(self.value_type, out_ptr, "promoted_v")
                    .unwrap()
                    .into_struct_value();
                self.builder.build_store(*p, promoted_v).unwrap();
            }
            StmtKind::Entangle { .. } => {
                // Future OM feature: static anchoring
            }
            _ => {}
        }
        Ok(())
    }

    fn box_value(&self, val: BasicValueEnum<'ctx>, ty: crate::types::Type) -> StructValue<'ctx> {
        let f64_type = self.f64_type;
        let mut s = self.value_type.get_undef();
        match ty {
            crate::types::Type::Int
            | crate::types::Type::I64
            | crate::types::Type::I32
            | crate::types::Type::I16
            | crate::types::Type::I8
            | crate::types::Type::U64
            | crate::types::Type::U32
            | crate::types::Type::U16
            | crate::types::Type::U8
            | crate::types::Type::Usize
            | crate::types::Type::Isize => {
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(TYPE_NUM as f64), 0, "t")
                    .unwrap()
                    .into_struct_value();
                let f_val = self
                    .builder
                    .build_signed_int_to_float(val.into_int_value(), f64_type, "to_f")
                    .unwrap();
                s = self
                    .builder
                    .build_insert_value(s, f_val, 1, "v")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, self.ptr_type.const_null(), 2, "p")
                    .unwrap()
                    .into_struct_value();
            }
            crate::types::Type::Float | crate::types::Type::F64 => {
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(TYPE_NUM as f64), 0, "t")
                    .unwrap()
                    .into_struct_value();
                let f = if val.into_float_value().get_type() == self.f64_type {
                    val.into_float_value()
                } else {
                    self.builder
                        .build_float_cast(val.into_float_value(), self.f64_type, "f_to_f64")
                        .unwrap()
                };
                s = self
                    .builder
                    .build_insert_value(s, f, 1, "v")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, self.ptr_type.const_null(), 2, "p")
                    .unwrap()
                    .into_struct_value();
            }
            crate::types::Type::F32 => {
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(TYPE_NUM as f64), 0, "t")
                    .unwrap()
                    .into_struct_value();
                let f_val = self
                    .builder
                    .build_float_cast(val.into_float_value(), f64_type, "f32_to_f64")
                    .unwrap();
                s = self
                    .builder
                    .build_insert_value(s, f_val, 1, "v")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, self.ptr_type.const_null(), 2, "p")
                    .unwrap()
                    .into_struct_value();
            }
            crate::types::Type::Bool => {
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(TYPE_BOOL as f64), 0, "t")
                    .unwrap()
                    .into_struct_value();
                let f_val = self
                    .builder
                    .build_unsigned_int_to_float(val.into_int_value(), f64_type, "to_f")
                    .unwrap();
                s = self
                    .builder
                    .build_insert_value(s, f_val, 1, "v")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, self.ptr_type.const_null(), 2, "p")
                    .unwrap()
                    .into_struct_value();
            }
            crate::types::Type::String => {
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(TYPE_STR as f64), 0, "t")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, f64_type.const_float(0.0), 1, "v")
                    .unwrap()
                    .into_struct_value();
                s = self
                    .builder
                    .build_insert_value(s, val.into_pointer_value(), 2, "p")
                    .unwrap()
                    .into_struct_value();
            }
            _ => {
                if val.is_struct_value() {
                    return val.into_struct_value();
                }
            }
        }
        s
    }

    fn unbox_value(
        &self,
        val: StructValue<'ctx>,
        target_ty: crate::types::Type,
    ) -> BasicValueEnum<'ctx> {
        match target_ty {
            crate::types::Type::Int
            | crate::types::Type::I64
            | crate::types::Type::I32
            | crate::types::Type::I16
            | crate::types::Type::I8
            | crate::types::Type::U64
            | crate::types::Type::U32
            | crate::types::Type::U16
            | crate::types::Type::U8
            | crate::types::Type::Usize
            | crate::types::Type::Isize => {
                let f_val = self
                    .builder
                    .build_extract_value(val, 1, "f")
                    .unwrap()
                    .into_float_value();
                self.builder
                    .build_float_to_signed_int(
                        f_val,
                        self.snask_type_to_llvm(&target_ty).into_int_type(),
                        "to_i",
                    )
                    .unwrap()
                    .into()
            }
            crate::types::Type::Float | crate::types::Type::F64 => {
                self.builder.build_extract_value(val, 1, "f").unwrap()
            }
            crate::types::Type::F32 => {
                let f_val = self
                    .builder
                    .build_extract_value(val, 1, "f")
                    .unwrap()
                    .into_float_value();
                self.builder
                    .build_float_cast(f_val, self.context.f32_type(), "to_f32")
                    .unwrap()
                    .into()
            }
            crate::types::Type::Bool => {
                let f_val = self
                    .builder
                    .build_extract_value(val, 1, "f")
                    .unwrap()
                    .into_float_value();
                let b = self
                    .builder
                    .build_float_compare(
                        inkwell::FloatPredicate::ONE,
                        f_val,
                        self.f64_type.const_float(0.0),
                        "to_b",
                    )
                    .unwrap();
                b.into()
            }
            crate::types::Type::String | crate::types::Type::Ptr | crate::types::Type::User(_) => {
                self.builder.build_extract_value(val, 2, "p").unwrap()
            }
            _ => val.into(),
        }
    }

    fn evaluate_expression(
        &self,
        expr: Expr,
    ) -> Result<(BasicValueEnum<'ctx>, crate::types::Type), String> {
        let nil_p = self.ptr_type.const_null();
        match expr.kind {
            ExprKind::Literal(lit) => match lit {
                LiteralValue::Number(n) => {
                    if n.fract() == 0.0 {
                        Ok((
                            self.i64_type.const_int(n as u64, true).into(),
                            crate::types::Type::Int,
                        ))
                    } else {
                        Ok((
                            self.f64_type.const_float(n).into(),
                            crate::types::Type::Float,
                        ))
                    }
                }
                LiteralValue::String(str_v) => {
                    let g = self.builder.build_global_string_ptr(&str_v, "s").unwrap();
                    Ok((g.as_pointer_value().into(), crate::types::Type::String))
                }
                LiteralValue::Boolean(b) => Ok((
                    self.bool_type
                        .const_int(if b { 1 } else { 0 }, false)
                        .into(),
                    crate::types::Type::Bool,
                )),
                LiteralValue::Nil => Ok((nil_p.into(), crate::types::Type::Void)),
                LiteralValue::Dict(pairs) => {
                    let fn_snif = self.functions.get("snif_new_object").unwrap();
                    let out_p = self.create_entry_block_alloca(self.value_type, "dict_out");
                    self.builder
                        .build_call(*fn_snif, &[out_p.into()], "dict_new")
                        .unwrap();
                    let obj_val = self
                        .builder
                        .build_load(self.value_type, out_p, "obj_val")
                        .unwrap()
                        .into_struct_value();

                    if !pairs.is_empty() {
                        let fn_set = self.functions.get("json_set").unwrap();
                        let obj_p = self.create_entry_block_alloca(self.value_type, "objp");
                        self.builder.build_store(obj_p, obj_val).unwrap();

                        for (k_expr, v_expr) in pairs.clone() {
                            let (k_val, k_ty) = self.evaluate_expression(k_expr)?;
                            let (v_val, v_ty) = self.evaluate_expression(v_expr)?;

                            let k_boxed = self.box_value(k_val, k_ty);
                            let v_boxed = self.box_value(v_val, v_ty);

                            let k_p = self.create_entry_block_alloca(self.value_type, "kp");
                            self.builder.build_store(k_p, k_boxed).unwrap();

                            let v_p = self.create_entry_block_alloca(self.value_type, "vp");
                            self.builder.build_store(v_p, v_boxed).unwrap();

                            let set_out_p =
                                self.create_entry_block_alloca(self.value_type, "set_out");
                            self.builder
                                .build_call(
                                    *fn_set,
                                    &[set_out_p.into(), obj_p.into(), k_p.into(), v_p.into()],
                                    "set_call",
                                )
                                .unwrap();
                        }
                    }
                    Ok((obj_val.into(), crate::types::Type::Dict))
                }
                LiteralValue::List(items) => {
                    let fn_snif = self.functions.get("snif_new_array").unwrap();
                    let out_p = self.create_entry_block_alloca(self.value_type, "arr_out");
                    self.builder
                        .build_call(*fn_snif, &[out_p.into()], "arr_new")
                        .unwrap();
                    let arr_val = self
                        .builder
                        .build_load(self.value_type, out_p, "arr_val")
                        .unwrap()
                        .into_struct_value();

                    if !items.is_empty() {
                        let fn_push = self.functions.get("snif_arr_push").unwrap();
                        let arr_p = self.create_entry_block_alloca(self.value_type, "arrp");
                        self.builder.build_store(arr_p, arr_val).unwrap();

                        for item_expr in items.clone() {
                            let (v_val, v_ty) = self.evaluate_expression(item_expr)?;
                            let v_boxed = self.box_value(v_val, v_ty);

                            let v_p = self.create_entry_block_alloca(self.value_type, "vp");
                            self.builder.build_store(v_p, v_boxed).unwrap();

                            let push_out_p =
                                self.create_entry_block_alloca(self.value_type, "push_out");
                            self.builder
                                .build_call(
                                    *fn_push,
                                    &[push_out_p.into(), arr_p.into(), v_p.into()],
                                    "push_call",
                                )
                                .unwrap();
                        }
                    }
                    Ok((arr_val.into(), crate::types::Type::List))
                }
                _ => Err(format!("Lit not supported: {:?}", lit)),
            },
            ExprKind::Variable(name) => {
                if let Some((p, ty)) = self
                    .local_vars
                    .get(&name)
                    .or_else(|| self.variables.get(&name))
                {
                    let llvm_ty = self.snask_type_to_llvm(ty);
                    return Ok((
                        self.builder.build_load(llvm_ty, *p, &name).unwrap(),
                        ty.clone(),
                    ));
                }
                Err(format!("Var {} not found.", name))
            }
            ExprKind::Binary { op, left, right } => {
                let (lhs, lty) = self.evaluate_expression(*left)?;
                let (rhs, rty) = self.evaluate_expression(*right)?;

                if matches!(op, BinaryOp::Add)
                    && (lty == crate::types::Type::String || rty == crate::types::Type::String)
                {
                    let res_p = self.create_entry_block_alloca(self.value_type, "rp");
                    let f = self.module.get_function("s_concat").unwrap();
                    let lp = self.create_entry_block_alloca(self.value_type, "lp");
                    let rp = self.create_entry_block_alloca(self.value_type, "rp");
                    let l_boxed = self.box_value(lhs, lty.clone());
                    let r_boxed = self.box_value(rhs, rty.clone());
                    self.builder.build_store(lp, l_boxed).unwrap();
                    self.builder.build_store(rp, r_boxed).unwrap();
                    self.builder
                        .build_call(f, &[res_p.into(), lp.into(), rp.into()], "c")
                        .unwrap();
                    let res_v = self
                        .builder
                        .build_load(self.value_type, res_p, "f")
                        .unwrap()
                        .into_struct_value();
                    return Ok((
                        self.unbox_value(res_v, crate::types::Type::String),
                        crate::types::Type::String,
                    ));
                }

                if lty.is_numeric() && rty.is_numeric() {
                    if lty.is_integer() && rty.is_integer() {
                        let result_ty = if matches!(
                            op,
                            BinaryOp::LessThan
                                | BinaryOp::LessThanOrEquals
                                | BinaryOp::GreaterThan
                                | BinaryOp::GreaterThanOrEquals
                                | BinaryOp::Equals
                                | BinaryOp::StrictEquals
                                | BinaryOp::NotEquals
                        ) {
                            crate::types::Type::Bool
                        } else if lty == rty {
                            lty.clone()
                        } else {
                            crate::types::Type::Int
                        };
                        let op_ty = if result_ty == crate::types::Type::Bool {
                            if lty == rty {
                                lty.clone()
                            } else {
                                crate::types::Type::Int
                            }
                        } else {
                            result_ty.clone()
                        };
                        let li = self
                            .cast_basic_value(lhs, lty.clone(), &op_ty)
                            .into_int_value();
                        let ri = self
                            .cast_basic_value(rhs, rty.clone(), &op_ty)
                            .into_int_value();
                        let res = match op {
                            BinaryOp::Add => {
                                self.builder.build_int_add(li, ri, "add").unwrap().into()
                            }
                            BinaryOp::Subtract => {
                                self.builder.build_int_sub(li, ri, "sub").unwrap().into()
                            }
                            BinaryOp::Multiply => {
                                self.builder.build_int_mul(li, ri, "mul").unwrap().into()
                            }
                            BinaryOp::Divide | BinaryOp::IntDivide => self
                                .builder
                                .build_int_signed_div(li, ri, "div")
                                .unwrap()
                                .into(),
                            BinaryOp::Modulo => self
                                .builder
                                .build_int_signed_rem(li, ri, "rem")
                                .unwrap()
                                .into(),
                            BinaryOp::BitAnd => {
                                self.builder.build_and(li, ri, "band").unwrap().into()
                            }
                            BinaryOp::BitOr => self.builder.build_or(li, ri, "bor").unwrap().into(),
                            BinaryOp::BitXor => {
                                self.builder.build_xor(li, ri, "bxor").unwrap().into()
                            }
                            BinaryOp::ShiftLeft => {
                                self.builder.build_left_shift(li, ri, "shl").unwrap().into()
                            }
                            BinaryOp::ShiftRight => self
                                .builder
                                .build_right_shift(li, ri, true, "shr")
                                .unwrap()
                                .into(),
                            BinaryOp::LessThan => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SLT, li, ri, "lt")
                                .unwrap()
                                .into(),
                            BinaryOp::LessThanOrEquals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SLE, li, ri, "le")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterThan => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SGT, li, ri, "gt")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterThanOrEquals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SGE, li, ri, "ge")
                                .unwrap()
                                .into(),
                            BinaryOp::Equals | BinaryOp::StrictEquals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::EQ, li, ri, "eq")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEquals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::NE, li, ri, "ne")
                                .unwrap()
                                .into(),
                            _ => self
                                .snask_type_to_llvm(&op_ty)
                                .into_int_type()
                                .const_zero()
                                .into(),
                        };
                        return Ok((res, result_ty));
                    }

                    if lty == crate::types::Type::Int && rty == crate::types::Type::Int {
                        let li = lhs.into_int_value();
                        let ri = rhs.into_int_value();
                        let res = match op {
                            BinaryOp::Add => {
                                self.builder.build_int_add(li, ri, "add").unwrap().into()
                            }
                            BinaryOp::Subtract => {
                                self.builder.build_int_sub(li, ri, "sub").unwrap().into()
                            }
                            BinaryOp::Multiply => {
                                self.builder.build_int_mul(li, ri, "mul").unwrap().into()
                            }
                            BinaryOp::Divide => self
                                .builder
                                .build_int_signed_div(li, ri, "div")
                                .unwrap()
                                .into(),
                            BinaryOp::LessThan => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SLT, li, ri, "lt")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterThan => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::SGT, li, ri, "gt")
                                .unwrap()
                                .into(),
                            BinaryOp::Equals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::EQ, li, ri, "eq")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEquals => self
                                .builder
                                .build_int_compare(inkwell::IntPredicate::NE, li, ri, "ne")
                                .unwrap()
                                .into(),
                            _ => self.i64_type.const_int(0, false).into(),
                        };
                        let res_ty = if matches!(
                            op,
                            BinaryOp::LessThan
                                | BinaryOp::GreaterThan
                                | BinaryOp::Equals
                                | BinaryOp::NotEquals
                        ) {
                            crate::types::Type::Bool
                        } else {
                            crate::types::Type::Int
                        };
                        return Ok((res, res_ty));
                    } else {
                        // Cast para float e opera
                        let lf = if lty == crate::types::Type::Int {
                            self.builder
                                .build_signed_int_to_float(
                                    lhs.into_int_value(),
                                    self.f64_type,
                                    "l2f",
                                )
                                .unwrap()
                        } else {
                            lhs.into_float_value()
                        };
                        let rf = if rty == crate::types::Type::Int {
                            self.builder
                                .build_signed_int_to_float(
                                    rhs.into_int_value(),
                                    self.f64_type,
                                    "r2f",
                                )
                                .unwrap()
                        } else {
                            rhs.into_float_value()
                        };
                        let res = match op {
                            BinaryOp::Add => {
                                self.builder.build_float_add(lf, rf, "fadd").unwrap().into()
                            }
                            BinaryOp::Subtract => {
                                self.builder.build_float_sub(lf, rf, "fsub").unwrap().into()
                            }
                            BinaryOp::Multiply => {
                                self.builder.build_float_mul(lf, rf, "fmul").unwrap().into()
                            }
                            BinaryOp::Divide => {
                                self.builder.build_float_div(lf, rf, "fdiv").unwrap().into()
                            }
                            BinaryOp::LessThan => self
                                .builder
                                .build_float_compare(inkwell::FloatPredicate::OLT, lf, rf, "flt")
                                .unwrap()
                                .into(),
                            BinaryOp::GreaterThan => self
                                .builder
                                .build_float_compare(inkwell::FloatPredicate::OGT, lf, rf, "fgt")
                                .unwrap()
                                .into(),
                            BinaryOp::Equals => self
                                .builder
                                .build_float_compare(inkwell::FloatPredicate::OEQ, lf, rf, "feq")
                                .unwrap()
                                .into(),
                            BinaryOp::NotEquals => self
                                .builder
                                .build_float_compare(inkwell::FloatPredicate::ONE, lf, rf, "fne")
                                .unwrap()
                                .into(),
                            _ => self.f64_type.const_float(0.0).into(),
                        };
                        let res_ty = if matches!(
                            op,
                            BinaryOp::LessThan
                                | BinaryOp::GreaterThan
                                | BinaryOp::Equals
                                | BinaryOp::NotEquals
                        ) {
                            crate::types::Type::Bool
                        } else {
                            crate::types::Type::Float
                        };
                        return Ok((res, res_ty));
                    }
                }

                Err(format!(
                    "Operation {:?} not supported for types {:?} and {:?}",
                    op, lty, rty
                ))
            }
            ExprKind::PropertyAccess { target, property } => {
                if let ExprKind::Variable(library) = &target.kind {
                    let surface = format!("{}.{}", library, property);
                    if let Some(value) = self.om_constant_for_surface(library, &surface) {
                        return Ok((
                            self.i64_type.const_int(value as u64, true).into(),
                            crate::types::Type::Int,
                        ));
                    }
                }

                let (obj, obj_ty) = self.evaluate_expression(*target)?;
                let get_f = self.functions.get("json_get").unwrap();

                let obj_boxed = self.box_value(obj, obj_ty);
                let key_boxed = self.box_value(
                    self.builder
                        .build_global_string_ptr(&property, "prop_lookup")
                        .unwrap()
                        .as_pointer_value()
                        .into(),
                    crate::types::Type::String,
                );

                let obj_p = self.create_entry_block_alloca(self.value_type, "objp");
                self.builder.build_store(obj_p, obj_boxed).unwrap();
                let idx_p = self.create_entry_block_alloca(self.value_type, "idxp");
                self.builder.build_store(idx_p, key_boxed).unwrap();

                let res_p = self.create_entry_block_alloca(self.value_type, "rp");
                self.builder
                    .build_call(*get_f, &[res_p.into(), obj_p.into(), idx_p.into()], "get")
                    .unwrap();

                let res_v = self
                    .builder
                    .build_load(self.value_type, res_p, "r")
                    .unwrap()
                    .into_struct_value();
                Ok((res_v.into(), crate::types::Type::Any))
            }
            ExprKind::IndexAccess { target, index } => {
                let (obj, obj_ty) = self.evaluate_expression(*target)?;
                let (idx, idx_ty) = self.evaluate_expression(*index)?;

                let get_f = self.functions.get("json_get").unwrap();

                let obj_boxed = self.box_value(obj, obj_ty);
                let idx_boxed = self.box_value(idx, idx_ty);

                let obj_p = self.create_entry_block_alloca(self.value_type, "objp");
                self.builder.build_store(obj_p, obj_boxed).unwrap();
                let idx_p = self.create_entry_block_alloca(self.value_type, "idxp");
                self.builder.build_store(idx_p, idx_boxed).unwrap();

                let out_p = self.create_entry_block_alloca(self.value_type, "outp");
                self.builder
                    .build_call(
                        *get_f,
                        &[out_p.into(), obj_p.into(), idx_p.into()],
                        "idx_get",
                    )
                    .unwrap();

                let res_v = self
                    .builder
                    .build_load(self.value_type, out_p, "r")
                    .unwrap()
                    .into_struct_value();
                Ok((res_v.into(), crate::types::Type::Any))
            }
            ExprKind::FunctionCall { callee, args } => {
                if let Some(path) = Self::expr_path(&callee) {
                    if path.len() == 2 {
                        let library = &path[0];
                        let surface = format!("{}.{}", library, path[1]);
                        let contract_fn = self.om_function_for_surface(library, &surface)?;
                        self.ensure_om_function_exposed(contract_fn)?;
                        if contract_fn.c_function == "SDL_PollEvent"
                            && contract_fn.surface == "sdl2.poll_event"
                        {
                            return self.emit_sdl_poll_event_type();
                        }

                        let wrapper_name = surface.replace('.', "_");
                        if let Some(f) = self.functions.get(&wrapper_name).cloned() {
                            let out_p = self.create_entry_block_alloca(self.value_type, "om_out");
                            let mut call_args: Vec<BasicMetadataValueEnum> = vec![out_p.into()];
                            for arg in args {
                                let (v, ty) = self.evaluate_expression(arg.clone())?;
                                let boxed = self.box_value(v, ty);
                                let arg_p =
                                    self.create_entry_block_alloca(self.value_type, "om_arg");
                                self.builder.build_store(arg_p, boxed).unwrap();
                                call_args.push(arg_p.into());
                            }

                            self.builder.build_call(f, &call_args, "om_call").unwrap();
                            let res_v = self
                                .builder
                                .build_load(self.value_type, out_p, "om_ret")
                                .unwrap()
                                .into_struct_value();
                            return Ok((res_v.into(), crate::types::Type::Any));
                        }

                        if contract_fn.c_param_types.len() != args.len() {
                            return Err(format!(
                                "OM function `{surface}` expects {} C arguments, got {}.",
                                contract_fn.c_param_types.len(),
                                args.len()
                            ));
                        }

                        let f = self.declare_c_function(contract_fn)?;
                        let mut call_args: Vec<BasicMetadataValueEnum> = Vec::new();
                        for (arg, c_type) in args.iter().zip(contract_fn.c_param_types.iter()) {
                            let (v, ty) = self.evaluate_expression(arg.clone())?;
                            call_args.push(self.convert_to_c_arg(v, ty, c_type)?);
                        }

                        let call = self.builder.build_call(f, &call_args, "om_c_call").unwrap();
                        let c_return = contract_fn.c_return_type.as_deref().unwrap_or("void");
                        if c_return.contains('*') {
                            if let Some(resource) =
                                self.om_resource_for_constructor(library, &contract_fn.c_function)
                            {
                                let c_ptr = call
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or_else(|| {
                                        format!(
                                            "OM constructor `{surface}` did not return a C pointer."
                                        )
                                    })?
                                    .into_pointer_value();
                                let resource_value = self.register_om_resource(c_ptr, resource)?;
                                return Ok((resource_value.into(), crate::types::Type::Any));
                            }
                        }
                        let (value, mut ty) =
                            self.c_return_to_snask(call.try_as_basic_value().left(), c_return)?;
                        if contract_fn.output == "str" && ty == crate::types::Type::Ptr {
                            ty = crate::types::Type::String;
                        }
                        return Ok((value, ty));
                    }
                }

                let mut l_args = Vec::new();
                let r_a = self.create_entry_block_alloca(self.value_type, "ra");
                l_args.push(r_a.into());

                if let ExprKind::Variable(name) = &callee.kind {
                    if let Some(result) = self.emit_systems_low_level_builtin(name, &args)? {
                        return Ok(result);
                    }

                    if matches!(
                        name.as_str(),
                        "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add"
                    ) && args.len() == 2
                    {
                        let (lhs, lty) = self.evaluate_expression(args[0].clone())?;
                        let (rhs, rty) = self.evaluate_expression(args[1].clone())?;
                        if lty.is_integer() && rty.is_integer() {
                            let result_ty = if lty == rty {
                                lty.clone()
                            } else {
                                crate::types::Type::Int
                            };
                            let li = self.cast_basic_value(lhs, lty, &result_ty).into_int_value();
                            let ri = self.cast_basic_value(rhs, rty, &result_ty).into_int_value();
                            let raw = match name.as_str() {
                                "wrapping_add" => {
                                    self.builder.build_int_add(li, ri, "wrap_add").unwrap()
                                }
                                "wrapping_sub" => {
                                    self.builder.build_int_sub(li, ri, "wrap_sub").unwrap()
                                }
                                "wrapping_mul" => {
                                    self.builder.build_int_mul(li, ri, "wrap_mul").unwrap()
                                }
                                "saturating_add" => {
                                    let sum =
                                        self.builder.build_int_add(li, ri, "sat_add").unwrap();
                                    let overflow = self
                                        .builder
                                        .build_int_compare(
                                            inkwell::IntPredicate::ULT,
                                            sum,
                                            li,
                                            "sat_overflow",
                                        )
                                        .unwrap();
                                    let max = sum.get_type().const_all_ones();
                                    self.builder
                                        .build_select(overflow, max, sum, "sat_select")
                                        .unwrap()
                                        .into_int_value()
                                }
                                _ => unreachable!(),
                            };
                            return Ok((raw.into(), result_ty));
                        }
                    }

                    let f = self
                        .module
                        .get_function(name)
                        .or_else(|| self.module.get_function(&format!("f_{}", name)))
                        .or_else(|| self.functions.get(name).cloned())
                        .ok_or_else(|| format!("Função {} não encontrada.", name))?;

                    for arg in args {
                        let (v, ty) = self.evaluate_expression(arg.clone())?;
                        let boxed = self.box_value(v, ty);
                        let arg_a = self.create_entry_block_alloca(self.value_type, "a");
                        self.builder.build_store(arg_a, boxed).unwrap();
                        l_args.push(arg_a.into());
                    }

                    self.builder.build_call(f, &l_args, "c").unwrap();
                    let res_v = self
                        .builder
                        .build_load(self.value_type, r_a, "r")
                        .unwrap()
                        .into_struct_value();
                    return Ok((res_v.into(), crate::types::Type::Any));
                }
                Err("Indirect not supported.".to_string())
            }
            ExprKind::Unary { op, expr } => {
                let (raw, ty) = self.evaluate_expression(*expr)?;
                match op {
                    crate::ast::UnaryOp::Negative => match ty {
                        crate::types::Type::Float => Ok((
                            self.builder
                                .build_float_neg(raw.into_float_value(), "neg")
                                .unwrap()
                                .into(),
                            crate::types::Type::Float,
                        )),
                        crate::types::Type::Int
                        | crate::types::Type::I64
                        | crate::types::Type::I32
                        | crate::types::Type::I16
                        | crate::types::Type::I8
                        | crate::types::Type::U64
                        | crate::types::Type::U32
                        | crate::types::Type::U16
                        | crate::types::Type::U8
                        | crate::types::Type::Usize
                        | crate::types::Type::Isize => Ok((
                            self.builder
                                .build_int_neg(raw.into_int_value(), "neg")
                                .unwrap()
                                .into(),
                            ty,
                        )),
                        _ => {
                            let boxed = self.box_value(raw, ty);
                            let n = self
                                .builder
                                .build_extract_value(boxed, 1, "n")
                                .unwrap()
                                .into_float_value();
                            Ok((
                                self.builder.build_float_neg(n, "neg").unwrap().into(),
                                crate::types::Type::Float,
                            ))
                        }
                    },
                    crate::ast::UnaryOp::BitNot => {
                        if ty.is_integer() {
                            Ok((
                                self.builder
                                    .build_not(raw.into_int_value(), "bitnot")
                                    .unwrap()
                                    .into(),
                                ty,
                            ))
                        } else {
                            Err(format!("Bitwise not not supported for {:?}", ty))
                        }
                    }
                    crate::ast::UnaryOp::Not => {
                        let is_true = match ty {
                            crate::types::Type::Bool => raw.into_int_value(),
                            _ => {
                                let boxed = self.box_value(raw, ty);
                                let n = self
                                    .builder
                                    .build_extract_value(boxed, 1, "n")
                                    .unwrap()
                                    .into_float_value();
                                self.builder
                                    .build_float_compare(
                                        inkwell::FloatPredicate::ONE,
                                        n,
                                        self.context.f64_type().const_float(0.0),
                                        "is_true",
                                    )
                                    .unwrap()
                            }
                        };
                        Ok((
                            self.builder.build_not(is_true, "not").unwrap().into(),
                            crate::types::Type::Bool,
                        ))
                    }
                }
            }
            ExprKind::New {
                class,
                args,
                strategy,
            } => {
                // Instanciação de classe (simplificada para o novo modelo)
                let out_p = self.create_entry_block_alloca(self.value_type, "alloc_out");
                let fn_alloc = self.functions.get("s_alloc_obj").unwrap();
                let properties = self.collect_class_properties(&class)?;

                let size_boxed = self.box_value(
                    self.f64_type.const_float(properties.len() as f64).into(),
                    crate::types::Type::Float,
                );
                let size_p = self.create_entry_block_alloca(self.value_type, "szp");
                self.builder.build_store(size_p, size_boxed).unwrap();

                let names_arg = self.build_class_names_arg(&class, &properties)?;
                self.builder
                    .build_call(
                        *fn_alloc,
                        &[out_p.into(), size_p.into(), names_arg.into()],
                        "alloc",
                    )
                    .unwrap();

                let res_v = self
                    .builder
                    .build_load(self.value_type, out_p, "obj")
                    .unwrap()
                    .into_struct_value();
                Ok((res_v.into(), crate::types::Type::User(class)))
            }
            _ => Err(format!("Expr not supported: {:?}", expr.kind)),
        }
    }
    pub fn emit_to_file(&self, path: &str) -> Result<(), String> {
        self.module
            .print_to_file(std::path::Path::new(path))
            .map_err(|e| e.to_string())
    }
}

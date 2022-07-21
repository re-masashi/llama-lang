/*

extern crate inkwell;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::Peekable;
use std::ops::DerefMut;
use std::str::Chars;

use self::inkwell::builder::Builder;
use self::inkwell::context::Context;
use self::inkwell::module::Module;
use self::inkwell::passes::PassManager;
use self::inkwell::types::BasicMetadataTypeEnum;
use self::inkwell::values::{BasicMetadataValueEnum, BasicValue, FloatValue, FunctionValue, PointerValue};
use self::inkwell::{FloatPredicate, OptimizationLevel};

use crate::parser::{ExprValue,Function};


pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
    pub function: &'a Function,

    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    /// Returns the `FunctionValue` representing the function being compiled.
    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    /// Creates a new stack allocation instruction in the entry block of the function.
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.i32_type(), name)
    }

    /// Compiles the specified `Expr` into an LLVM `FloatValue`.
    fn compile_expr(&mut self, expr: &Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match *expr {
            ExprValue::Integer(nb) => Ok(self.context.i32_type().const_int(nb)),

            ExprValue::Identifier(ref name) => match self.variables.get(name.clone()) {
                Some(var) => Ok(self.builder.build_load(*var, name.clone()).into_int_value()),
                None => Err("Could not find a matching variable."),
            },

            ExprValue ::BinOp {
                ref left,
                op,
                ref right,
            } => {
                    let lhs = self.compile_expr(left)?;
                    let rhs = self.compile_expr(right)?;

                    match *op {
                        TokenType::Plus => Ok(self.builder.build_int_add(lhs, rhs, "tmpadd")),
                        TokenType::Minus => Ok(self.builder.build_int_sub(lhs, rhs, "tmpsub")),
                        TokenType::Mul => Ok(self.builder.build_int_mul(lhs, rhs, "tmpmul")),
                        TokenType::Div => Ok(self.builder.build_int_div(lhs, rhs, "tmpdiv")),
                        TokenType::Less => Ok({
                            let cmp = self
                                .builder
                                .build_int_compare(FloatPredicate::ULT, lhs, rhs, "tmpcmp");
                        }),
                        TokenType::Greater => Ok({
                            let cmp = self
                                .builder
                                .build_int_compare(FloatPredicate::ULT, rhs, lhs, "tmpcmp");
                        }),

                        _ => unreachable!("Unexpected operator"),
                }
            },

            ExprValue::FnCall(ref fn_name, ref args ) => match self.get_function(fn_name.as_str()) {
                Some(fun) => {
                    let mut compiled_args = Vec::with_capacity(args.len());

                    for arg in args {
                        compiled_args.push(self.compile_expr(arg)?);
                    }

                    let argsv: Vec<BasicMetadataValueEnum> =
                        compiled_args.iter().by_ref().map(|&val| val.into()).collect();

                    match self
                        .builder
                        .build_call(fun, argsv.as_slice(), "tmp")
                        .try_as_basic_value()
                        .left()
                    {
                        Some(value) => Ok(value.into_int_value()),
                        None => Err("Invalid call produced."),
                    }
                },
                None => Err("Unknown function."),
            },

            ExprValue::IfElse {
                ref cond,
                ref if_,
                ref else_,
            } => {
                let parent = self.fn_value();
                let zero_const = self.context.i32_type().integer_value(0);

                // create condition by comparing without 0.0 and returning an int
                let cond = self.compile_expr(cond)?;
                let cond = self
                    .builder
                    .build_int_compare(FloatPredicate::ONE, cond, zero_const, "ifcond");

                // build branch
                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let cont_bb = self.context.append_basic_block(parent, "ifcont");

                self.builder.build_conditional_branch(cond, then_bb, else_bb);

                // build then block
                self.builder.position_at_end(then_bb);
                let then_val = self.compile_expr(if_)?;
                self.builder.build_unconditional_branch(cont_bb);

                let then_bb = self.builder.get_insert_block().unwrap();

                // build else block
                self.builder.position_at_end(else_bb);
                let else_val = self.compile_expr(else_)?;
                self.builder.build_unconditional_branch(cont_bb);

                let else_bb = self.builder.get_insert_block().unwrap();

                // emit merge block
                self.builder.position_at_end(cont_bb);

                let phi = self.builder.build_phi(self.context.f64_type(), "iftmp");

                phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

                Ok(phi.as_basic_value().into_int_value())
            },

        }
    }

    /// Compiles the specified `Function` in the given `Context` and using the specified `Builder`, `PassManager`, and `Module`.
    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        function: &Function,
    ) -> Result<FunctionValue<'ctx>, &'static str> {
        let mut compiler = Compiler {
            context,
            builder,
            fpm: pass_manager,
            module,
            function,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        compiler.compile_fn()
    }
}

// ======================================================================================
// PROGRAM ==============================================================================
// ======================================================================================

// macro used to print & flush without printing a new line
*/
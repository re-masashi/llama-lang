use std::collections::HashMap;
use inkwell::{
	context::Context,
	builder::Builder,
	module::Module,
	values::{FunctionValue,PointerValue, AnyValueEnum}
};
use crate::{
    Result,
    parser::ExprValue,
};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    // pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
    // pub function: &'a Function,

    variables: HashMap<String, (String /* Type */, PointerValue<'ctx>)>, 
    fn_value_opt: Option<FunctionValue<'ctx>>
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

     fn compile_expr(&mut self, expr: &ExprValue) -> Result<AnyValueEnum<'ctx>> {
        match *expr {
            ExprValue::Integer(n) => Ok(AnyValueEnum::IntValue(self.context.i32_type().const_int(n as u64, false))),

            ExprValue::Identifier(ref name) => {
                match self.variables.get(name) {
                    Some(type_,var) => {
                        let var_type = self.builder.build_load(*var, name.as_str());
                        let var_value = match type_.clone() {
                           "i32".to_string() => AnyValueEnum::IntValue(
                                    var_type.into_float_value()
                                ),
                           _ => unimplemented!(),
                        };
                        Ok(var_value)
                    },
                    None => Err("Could not find a matching variable.".to_string())
                }
            },

            ExprValue::VarDecl{ref name, ref type_} => {
                match self.variables.get(name) {
                    Some(_) => Err("Variable already declared."),
                    None => {
                        let value = self.builder.build_alloca(name.clone().as_str());
                        self.variables.insert(name,(type_, value));
                        return self.compile_expr(ExprValue::Identifier(ref name));
                    }
                }
            },

            ExprValue::Boolean(n) => Ok(AnyValueEnum::IntValue(self.context.bool_type().const_int(n as u64, false))),

    }
}
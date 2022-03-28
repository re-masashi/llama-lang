/*
use std::collections::HashMap;
use inkwell::{
	context::Context,
	builder::Builder,
	module::Module,
	values::{FunctionValue,PointerValue, AnyValueEnum},
    types::AnyTypeEnum::IntType,
};
use crate::{
    Result,
    parser::{ExprValue, Function}
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
                    Some(tuple_) => {
                        match tuple_ {
                            (type_,var) => {
                                let var_type = self.builder.build_load(*var, name.as_str());
                                let var_value = match type_.as_str() {
                                   "i32" => AnyValueEnum::IntValue(
                                            var_type.into_float_value()
                                        ),
                                   _ => unimplemented!(),
                                };
                                Ok(var_value)
                            },
                            _ => unreachable!(),
                        }
                    },
                    None => Err("Could not find a matching variable.".to_string())
                }
            },

            ExprValue::VarDecl{ref name, ref type_} => {
                match self.variables.get(name) {
                    Some(_) => Err("Variable already declared.".to_string()),
                    None => {
                        let value = self.builder.build_alloca(name.clone().as_str());
                        self.variables.insert(name.clone(),(type_.clone(), value));
                        return self.compile_expr(&ExprValue::Integer(0));
                    }
                }
            },

            ExprValue::Boolean(n) => Ok(AnyValueEnum::IntValue(self.context.bool_type().const_int(n as u64, false))),
            
            ExprValue::Assign{ref name, ref value} => {
                match self.variables.get(name) {
                    Some(type_, var) => {
                        let var_value = match type_.clone().as_str() {
                            "i32" => AnyValueEnum::IntValue(
                                var_type.into_float_value()
                            ),
                            _ => unimplemented!(),
                        }; 
                        let value = self.builder.build_store(*type_, var_value);
                    },
                    None => Err("No such variable.".to_string()),
                }
            },
        }
    }

    fn compile_fn(&mut self, fn_: &Function){
        match fn_ {
               Function{name, args, expressions, return_type} => {
                    let fn_type = 
                    match return_type.as_str(){
                        "None" => self.context.void_type().fn_type(
                            &[BasicTypeEnum::IntType(IntType);args.len()], false
                        ),
                        "i32" => self.context.void_type().fn_type(
                            &[BasicTypeEnum::IntType(IntType);args.len()], false
                        ),
                        _ => unimplemented!(),
                    };
                    let fn_val = self.module.add_function(name.clone().as_str(), fn_type, None);
                    let entry_point = self.context.append_basic_block(fn_val, "entry");
                    self.builder.position_at_end(entry_point);
                    
                    for expr in expressions {
                        self.compile_expr(expr);
                    }
                }
                

        }
    }
}
*/
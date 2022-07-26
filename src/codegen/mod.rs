use std::collections::HashMap;
use inkwell::{
    context::Context,
    builder::Builder,
    module::Module,
    values::{FunctionValue,PointerValue, AnyValueEnum, IntValue},
    types::{BasicMetadataTypeEnum,IntType}
};
use crate::{
    Result,
    unwrap_some,
    parser::{ExprValue, Function}
};


pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    // pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,
    // pub function: &'a Function,

    variables: HashMap<String, (String /* Type */, PointerValue<'ctx>)>, 
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {

    pub fn new(context: &'ctx Context, builder: &'a Builder<'ctx>, module: &'a Module<'ctx> ) ->Self{
        let variables:HashMap<String,(String,  PointerValue<'ctx>)> = HashMap::new();
        let compiler:Compiler<'a,'ctx> = Compiler {
            context, builder, module, variables
        };
        compiler
    }
    /// Gets a defined function given its name.
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
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
                                            var_type.into_int_value()
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
                        let value = self.builder.build_alloca(self.context.i32_type(),name.clone().as_str());
                        self.variables.insert(name.clone(),(type_.clone(), value));
                        return self.compile_expr(&ExprValue::Integer(0));
                    }
                }
            },

            ExprValue::Boolean(n) => Ok(AnyValueEnum::IntValue(self.context.bool_type().const_int(n as u64, false))),
            
            ExprValue::Assign{ref name, ref value} => {
                        let value_ = self.ret_int(&*value);
                        match self.variables.get(name) {
                    Some((type_, var)) => {

                        self.builder.build_store(
                            *var, self.context.i32_type().const_int(
                                match &value_.get_zero_extended_constant(){Some(i)=>*i,None=>unreachable!()},
                                false
                            )
                        );
                        Ok(AnyValueEnum::IntValue(value_))
                    },
                    None => Err("No such variable.".to_string()),
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn compile_fn(&mut self, fn_: &Function){
        match fn_ {
               Function{name, args, expressions, return_type} => {
                    let fn_type = 
                    match return_type.as_str(){
                        "None" => self.context.void_type().fn_type(
                            &[BasicMetadataTypeEnum::IntType(self.context.i32_type())], false
                        ),
                        "i32" => self.context.void_type().fn_type(
                            &[BasicMetadataTypeEnum::IntType(self.context.i32_type())], false
                        ),
                        _ => unimplemented!(),
                    };
                    let fn_val = self.module.add_function(name.clone().as_str(), fn_type, None);
                    let entry_point = self.context.append_basic_block(fn_val, "entry");
                    self.builder.position_at_end(entry_point);
                    for (i,arg) in fn_val.get_param_iter().enumerate(){
                        let arg_name = args[0][i].as_str();
                        let alloca = self.builder.build_alloca(self.context.i32_type(), arg_name);
                    }
                    for expr in expressions {
                        self.compile_expr(expr);
                    }

                }
        }
    }
    fn ret_int(&mut self, value:&Box<ExprValue>)->IntValue<'ctx>{
        match self.compile_expr(&*value){
            Ok(AnyValueEnum::IntValue(i)) =>i,
            _ => unreachable!(),
        }
    }
}

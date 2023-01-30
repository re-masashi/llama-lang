use crate::c_str;
use crate::codegen::Generator;
use crate::lexer::tokens::TokenType;
use crate::parser::ExprValue;
use crate::Result;
use llvm_sys::core;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMIntPredicate;
use log::{trace, info};

impl Generator {
    pub unsafe fn gen_expression(&self, expression: &ExprValue, current_fn: LLVMValueRef) -> Result<LLVMValueRef> {
        trace!("Generating expression");
        match expression {

            ExprValue::Integer(i) => {
                trace!("Integer literal: {}", i);
                Ok(core::LLVMConstInt(self.i32_type(), *i as u64, false as i32))
            }

            ExprValue::Str(s) => {
                trace!("Str literal: {}", s);
                Ok(core::LLVMConstString(
                    c_str!(s),
                    s.len() as u32,
                    false as i32,
                ))
            }

            ExprValue::Boolean(b)=>{
                trace!("Integer literal: {}", b);
                Ok(core::LLVMConstInt(self.bool_type(), *b as u64, false as i32))
            }

            ExprValue::Identifier (name) => {
                trace!("Generating Identifier reference: {}", name);
                if let Some(var) = self.local_vars.borrow().get(name) {
                    trace!("Local variable: {}", name);
                    Ok(core::LLVMBuildLoad2(
                        self.builder,
                        self.i32_type(),
                        *var,
                        c_str!(""),
                    ))
                } else {
                    Err(format!("Unresolved variable reference `{}`", name))
                }
            }

            ExprValue::FnCall (name, args) => {
                trace!("Generating function call expression: {}", name);
                let mut llvm_args: Vec<LLVMValueRef> = Vec::new();
                for arg in args {
                    llvm_args.push(self.gen_expression(arg, current_fn)?);
                }

                let function = core::LLVMGetNamedFunction(self.module, c_str!(name));
                if function.is_null() {
                    return Err(format!("Function `{}` doesn't exist", name));
                }
                Ok(core::LLVMBuildCall(
                    self.builder,
                    function,
                    llvm_args.as_mut_ptr(),
                    args.len() as u32,
                    c_str!(""),
                ))
            }

            ExprValue::BinOp (l_expression, op, r_expression) => {
                trace!("Generating binary expression");

                let r = self.gen_expression(r_expression, current_fn)?;
                let l = self.gen_expression(l_expression, current_fn)?;
                match **op {
                    TokenType::Plus => Ok(core::LLVMBuildAdd(self.builder, l, r, c_str!(""))),
                    TokenType::Minus => Ok(core::LLVMBuildSub(self.builder, l, r, c_str!(""))),
                    TokenType::Mul => Ok(core::LLVMBuildMul(self.builder, l, r, c_str!(""))),
                    TokenType::Div => Ok(core::LLVMBuildSDiv(self.builder, l, r, c_str!(""))),
                    TokenType::Equal 
                    | TokenType::NotEq 
                    | TokenType::Less 
                    | TokenType::Greater 
                    | TokenType::LessEq
                    | TokenType::GreaterEq => {
                        let cmp = {
                            core::LLVMBuildICmp(
                                self.builder,
                                match **op {
                                    TokenType::Equal => LLVMIntPredicate::LLVMIntEQ,
                                    TokenType::NotEq  => LLVMIntPredicate::LLVMIntNE,
                                    TokenType::Greater => LLVMIntPredicate::LLVMIntSLT,
                                    TokenType::Less => LLVMIntPredicate::LLVMIntSGT,
                                    TokenType::LessEq => LLVMIntPredicate::LLVMIntSLE,
                                    TokenType::GreaterEq => LLVMIntPredicate::LLVMIntSGE,
                                    _ => {
                                        return Err("Unhandled comparison binary operation".to_string())
                                    }
                                },
                                l,
                                r,
                                c_str!(""),
                            )
                        };
                        Ok(cmp)
                    }
                    _ => Err("Misidentified binary expression".to_string()),
                }
            }

            ExprValue::UnOp ( op, expression ) => {
                trace!("Generating unary expression");
                match **op {
                    TokenType::Minus => Ok(core::LLVMBuildNeg(
                        self.builder,
                        self.gen_expression(expression, current_fn)?,
                        c_str!(""),
                    )),
                    TokenType::Not => Ok(core::LLVMBuildNot(
                        self.builder,
                        self.gen_expression(expression, current_fn)?,
                        c_str!(""),
                    )),
                    _ => Err("Misidentified unary expression".to_string()),
                }
            }

            ExprValue::IfElse {cond, if_, else_ } => {
                trace!("Generating if else");
                let entry = core::LLVMGetLastBasicBlock(current_fn);
                let end = core::LLVMAppendBasicBlock(current_fn, c_str!("end"));
                
                let if_bb = core::LLVMAppendBasicBlock(current_fn, c_str!("then"));
                core::LLVMPositionBuilderAtEnd(self.builder, if_bb);
                for expr in if_{
                    self.gen_expression(&*expr, current_fn)?;
                }
                core::LLVMBuildBr(self.builder, end);

                let else_bb = core::LLVMAppendBasicBlock(current_fn, c_str!("else"));
                core::LLVMPositionBuilderAtEnd(self.builder, else_bb);
                for expr in else_{
                    self.gen_expression(&*expr, current_fn)?;
                }

                core::LLVMPositionBuilderAtEnd(self.builder, entry);
                let cond_llvm = self.gen_expression(cond, current_fn)?;

                core::LLVMBuildCondBr(self.builder, cond_llvm, if_bb, else_bb);

                Ok(cond_llvm)
            }

            ExprValue::Return (value) => {
                trace!("Generating return");
                let val_llvm = self.gen_expression(value, current_fn)?;
                core::LLVMBuildRet(self.builder, val_llvm);
                Ok(val_llvm)
            }

            ExprValue::VarDecl { name, type_ } => {
                trace!("Generating variable declaration {}", name);
                let mut local_vars_mut = self.local_vars.borrow_mut();

                if local_vars_mut.contains_key(name) {
                    return Err(format!("Variable `{}` already exists", name));
                }

                let var = core::LLVMBuildAlloca(
                    self.builder, 
                    match type_.as_str() {
                        "i32"=>self.i32_type(),
                        "i64"=>self.i64_type(),
                        "bool"=>self.bool_type(),
                        _ => todo!(),
                    },
                     c_str!(""));
                info!("Adding `{}` to local vars", name);
                local_vars_mut.insert(String::from(name), var);
                self.scope_var_names
                    .borrow_mut()
                    .last_mut()
                    .unwrap()
                    .push(String::from(name));

                drop(local_vars_mut);
                Ok(var)
            }

            ExprValue::Assign {name, value} =>{
                Ok(core::LLVMBuildStore(self.builder, 
                    self.gen_expression(value, current_fn)?, 
                    match self.local_vars.borrow().get(name) {
                        Some(v)=>*v,
                        None=>panic!("No such variable")
                    }
                ))
            }

            ExprValue::AugAssign { .. } => todo!()
        }
    }
}

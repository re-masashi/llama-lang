use crate::c_str;
use crate::codegen::Generator;
use crate::parser::Function;
use crate::Result;
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::{
    core,
    prelude::LLVMTypeRef
};
use log::{info, trace};

impl Generator {
    pub unsafe fn gen_function(&self, function: &Function, mut current_fn: LLVMValueRef) -> Result<()> {
        trace!("Generating function");

        let args = &function.args;

        let name = function.name.clone();

        let mut arg_types: Vec<LLVMTypeRef> = Vec::new();

        for arg in args.type_.iter() {
            arg_types.insert(arg_types.len(),
                match arg.as_str(){
                    "i32" => self.i32_type(),
                    "i64" => self.i64_type(),
                    "bool" => self.bool_type(),
                    _=>todo!(),
                }
            )
        }

        let return_type = match function.return_type.as_str(){
            "i32" => self.i32_type(),
            "i64" => self.i64_type(),
            "bool" => self.bool_type(),
            _=>todo!(),
        };

        // Create function
        let llvm_fn = core::LLVMAddFunction(
            self.module,
            c_str!(name),
            core::LLVMFunctionType(
                return_type,
                arg_types.as_mut_ptr(),
                args.name.len() as u32, // name.len() == type_.len()
                0,
            ),
        );
        current_fn = llvm_fn;


        {
            // Append empty block
            let entry =
                core::LLVMAppendBasicBlockInContext(self.context, llvm_fn, c_str!("entry"));

            core::LLVMPositionBuilderAtEnd(self.builder, entry);

            for (i, arg_name) in args.name.iter().enumerate() {
                // Set arg name in function prototype
                let arg = core::LLVMGetParam(llvm_fn, i as u32);
                core::LLVMSetValueName2(arg, c_str!(arg_name), arg_name.len());

                let mut local_vars_mut = self.local_vars.borrow_mut();

                let var = core::LLVMBuildAlloca(self.builder, self.i32_type(), c_str!(""));
                core::LLVMBuildStore(self.builder, arg, var);
            }

            for expr in &function.expressions{
                self.gen_expression(expr, current_fn)?;
            }
        }

        Ok(())
    }
}

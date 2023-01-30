use crate::c_str;
use crate::codegen::Generator;
use crate::parser::External;
use crate::Result;
use llvm_sys::{
    core,
    prelude::LLVMTypeRef
};
use log::{info, trace};

impl Generator {
    pub unsafe fn gen_extern(&self, function: &External) -> Result<()> {
        trace!("Generating extern");

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

        Ok(())
    }
}
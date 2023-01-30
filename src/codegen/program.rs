use crate::codegen::Generator;
use crate::parser::AstNode;
use crate::Result;
use std::ptr;
use log::trace;

impl Generator {
    pub unsafe fn gen_program(&mut self, program: Vec<AstNode>) -> Result<()> {
        trace!("Generating program");
        let current_fn = ptr::null_mut();
        for function in &program {
            self.local_vars.borrow_mut().clear();
            match function {
                AstNode::FunctionDef(f) => self.gen_function(f, current_fn)?,
                AstNode::Extern(e) => self.gen_extern(e)?,
            };
        }
        Ok(())
    }
}
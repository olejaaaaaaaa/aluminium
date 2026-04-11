#![allow(missing_docs)]

use std::any::Any;

use super::{PassContext, PassBuilder, Setup};

use std::any::{TypeId};

pub struct PresentPass<'frame> {
    pub(crate) name: String,
    pub(crate) reads: Vec<bool>,
    pub(crate) execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> PresentPass<'frame> {
    
    pub fn new<Name, Type, Setup, Execute>(name: Name, setup: Setup, execute: Execute) -> Self 
    where 
        Name: Into<String>, 
        Type: Any + Send, 
        Setup: FnOnce(&mut PassBuilder<'frame>) -> Type + Send + 'frame,
        Execute: FnOnce(&mut PassContext, &Type) + Send + 'frame
    {
        let mut builder = PassBuilder {
            reads: vec![],
            writes: vec![],
            render_target_desc: None,
        };
        
        let data = setup(&mut builder);
        
        Self {
            name: name.into(),
            reads: builder.reads,
            execute: Box::new(move |ctx| {
                execute(ctx, &data);
            }),
        }
    }
}

impl<'a> Into<super::Pass<'a>> for PresentPass<'a> {
    fn into(self) -> super::Pass<'a> {
        super::Pass::Present(self)
    }
}

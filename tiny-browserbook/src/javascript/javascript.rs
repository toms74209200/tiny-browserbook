use std::{cell::RefCell, rc::Rc, sync::Once};

use v8::{
    new_default_platform, Context, CreateParams, EscapableHandleScope, Global, HandleScope,
    Isolate, OwnedIsolate,
    V8::{initialize, initialize_platform},
};

pub struct JavascriptRuntimeState {
    context: Global<Context>,
}

#[derive(Debug)]
pub struct JavascriptRuntime {
    v8_isolate: OwnedIsolate,
}

impl JavascriptRuntime {
    pub fn new() -> Self {
        static V8_INIT: Once = Once::new();
        V8_INIT.call_once(move || {
            let platform = new_default_platform(0, false).make_shared();
            initialize_platform(platform);
            initialize();
        });

        let mut isolate = Isolate::new(CreateParams::default());
        let context = {
            let isolate_scope = &mut HandleScope::new(&mut isolate);
            let handle_scope = &mut EscapableHandleScope::new(isolate_scope);
            let context = Context::new(handle_scope, Default::default());
            let context_scope = handle_scope.escape(context);
            Global::new(handle_scope, context_scope)
        };

        isolate.set_slot(Rc::new(RefCell::new(JavascriptRuntimeState { context })));

        JavascriptRuntime {
            v8_isolate: isolate,
        }
    }
}

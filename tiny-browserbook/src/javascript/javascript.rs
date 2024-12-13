use std::{cell::RefCell, rc::Rc, sync::Once};

use v8::{
    new_default_platform, undefined, Context, CreateParams, EscapableHandleScope, Global,
    HandleScope, Isolate, OwnedIsolate, Script, ScriptOrigin, TryCatch,
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

    pub fn execute(&mut self, filename: &str, source: &str) -> Result<String, String> {
        let scope = &mut self.get_handle_scope();

        let source = v8::String::new(scope, source).unwrap();
        let source_map = undefined(scope);
        let name = v8::String::new(scope, filename).unwrap();
        let origin = ScriptOrigin::new(
            scope,
            name.into(),
            0,
            0,
            false,
            0,
            Some(source_map.into()),
            false,
            false,
            false,
            None,
        );

        let mut tc_scope = TryCatch::new(scope);
        let script = match Script::compile(&mut tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                assert!(tc_scope.has_caught());
                return Err(to_pretty_string(tc_scope));
            }
        };

        match script.run(&mut tc_scope) {
            Some(result) => Ok(result
                .to_string(&mut tc_scope)
                .unwrap()
                .to_rust_string_lossy(&mut tc_scope)),
            None => {
                assert!(tc_scope.has_caught());
                Err(to_pretty_string(tc_scope))
            }
        }
    }
}

impl JavascriptRuntime {
    pub fn state(isolate: &Isolate) -> Rc<RefCell<JavascriptRuntimeState>> {
        let s = isolate
            .get_slot::<Rc<RefCell<JavascriptRuntimeState>>>()
            .unwrap();
        s.clone()
    }

    pub fn get_state(&self) -> Rc<RefCell<JavascriptRuntimeState>> {
        Self::state(&self.v8_isolate)
    }

    pub fn get_handle_scope(&mut self) -> HandleScope {
        let context = self.get_context();
        HandleScope::with_context(&mut self.v8_isolate, context)
    }

    pub fn get_context(&mut self) -> Global<Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }
}

fn to_pretty_string(mut try_catch: TryCatch<HandleScope>) -> String {
    let exception_string = try_catch
        .exception()
        .unwrap()
        .to_string(&mut try_catch)
        .unwrap()
        .to_rust_string_lossy(&mut try_catch);

    let message = try_catch.message().unwrap();

    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );
    let line_number = message.get_line_number(&mut try_catch).unwrap_or_default();
    format!("{}:{}: {}", filename, line_number, exception_string)
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[fixture]
    fn runtime() -> JavascriptRuntime {
        JavascriptRuntime::new()
    }

    #[rstest]
    fn test_execute_add(mut runtime: JavascriptRuntime) {
        let result = runtime.execute("", "1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }

    #[rstest]
    fn test_execute_add_string(mut runtime: JavascriptRuntime) {
        let result = runtime.execute("", "'test' + \"func\" + `012${1+1+1}`");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "testfunc0123");
    }

    #[rstest]
    fn test_execute_undefined(mut runtime: JavascriptRuntime) {
        let result = runtime.execute("", "test");
        assert!(result.is_err());
    }

    #[rstest]
    fn test_execute_lambda(mut runtime: JavascriptRuntime) {
        {
            let result = runtime.execute("", "let inc = (i) => { return i + 1 }; inc(1)");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "2");
        }
        {
            let result = runtime.execute("", "inc(4)");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "5");
        }
    }
}

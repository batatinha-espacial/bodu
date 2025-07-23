use std::collections::HashMap;

use bodu_vm::{make_container, make_err, op::{set_base, to_string_base}, Container, Function, Gi, StateContainer, Value};
use cbodu::op::load_lib;

macro_rules! make_function_true {
    ($state:expr, $scope:expr, $prop:expr, $fcall:expr) => {{
        let f = make_fn_true!($state, $fcall);
        set_base($state.clone(), $scope.clone(), $prop.to_string(), f).await.unwrap();
    }};
}

macro_rules! make_fn_true {
    ($state:expr, $fcall:expr) => {
        make_container(Value::Function(Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
            caller_state: true,
        }))
    };
}

pub async fn custom_init(state: StateContainer) {
    let scope = state.lock().await.scope.clone();
    make_function_true!(state, scope, "load_lib", load_lib_);
}

pub async fn load_lib_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("load_lib requires 1 argument"));
    }
    let a = to_string_base(state.clone(), args[0].clone()).await?;
    let f = load_lib(state.clone(), a).await?;
    Ok(f)
}
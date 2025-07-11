use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock, Weak}};

use crate::vm::{make_container, op::{call, make_object, make_object_base, set_base, to_string, to_string_base}, push_defer, Container, Function, Gi, State, StateContainer, Value};

// TODO: add comments

mod array;
mod iter;

macro_rules! make_function {
    ($state:expr, $scope:expr, $prop:expr, $fcall:expr) => {{
        let f = make_container(Value::Function(Function {
            internals: HashMap::new(),
            call: $fcall,
            state: $state.clone(),
        }));
        set_base($state.clone(), $scope.clone(), $prop.to_string(), f).unwrap();
    }};
}

pub fn new_global_state() -> StateContainer {
    let s = Arc::new(Mutex::new(State {
        scope: make_object(),
        parent: None,
        global: None,
    }));
    let s2 = s.clone();
    s.lock().unwrap().global = Some(s2);
    s
}

pub fn init_global_state(state: StateContainer) {
    let scope = state.lock().unwrap().scope.clone();
    make_function!(state, scope, "async", async_);
    make_function!(state, scope, "await", await_);
    make_function!(state, scope, "eprint", eprint);
    make_function!(state, scope, "input", input);
    make_function!(state, scope, "print", print);
    make_function!(state, scope, "string", string);
    {
        let array_object = make_object();
        make_function!(state, array_object, "new", array::new);
        set_base(state.clone(), scope.clone(), "array".to_string(), array_object).unwrap();
    }
    {
        let iter_object = make_object();
        make_function!(state, iter_object, "collect", iter::collect);
        make_function!(state, iter_object, "all", iter::all);
        make_function!(state, iter_object, "any", iter::any);
        make_function!(state, iter_object, "chain", iter::chain);
        set_base(state.clone(), scope.clone(), "iter".to_string(), iter_object).unwrap();
    }
}

fn print(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i?);
    }
    let args = args2;
    let str = args.join("\t");
    println!("{}", str);
    Ok(make_container(Value::Null))
}

fn eprint(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i?);
    }
    let args = args2;
    let str = args.join("\t");
    eprintln!("{}", str);
    Ok(make_container(Value::Null))
}

fn input(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i?);
    }
    let args = args2;
    let str = args.join("\t");
    print!("{}", str);
    let mut i = String::new();
    std::io::stdin().read_line(&mut i).unwrap();
    Ok(make_container(Value::String(i)))
}

static CONTAINERS: OnceLock<Mutex<Vec<Weak<Mutex<Value>>>>> = OnceLock::new();

fn init_async() {
    match CONTAINERS.set(Mutex::new(Vec::new())) {
        Ok(_) => {
            push_defer(Arc::new(|| {
                let cs = CONTAINERS.get().unwrap().lock().unwrap();
                let cs = cs.as_slice();
                for p in cs {
                    let p = match p.upgrade() {
                        Some(p) => p,
                        None => continue,
                    };
                    let p = match p.lock().unwrap().clone() {
                        Value::Object(obj) => obj,
                        _ => return,
                    };
                    let p = match p.externals.get(&0) {
                        None => return,
                        Some(a) => a,
                    };
                    let mut p = p.lock().unwrap();
                    let p = p.downcast_mut::<HashMap<u64, tokio::task::JoinHandle<Result<Container, Container>>>>();
                    let p = match p {
                        Some(a) => a,
                        None => return,
                    };
                    let p = match p.remove(&0) {
                        Some(a) => a,
                        None => return,
                    };
                    let _ = tokio::runtime::Runtime::new().unwrap().block_on(p.into_future());
                }
            }));
        },
        Err(_) => {},
    }
}

fn async_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("async requires 1 argument".to_string())))
    }
    let f = args[0].clone();
    let mut internals = HashMap::new();
    internals.insert(0, f);
    Ok(make_container(Value::Function(Function {
        internals,
        call: |state, args, gi| {
            let f = gi(0).unwrap();
            let g = {
                let f = f.clone();
                let state = state.clone();
                let args = args.clone();
                async move || {
                    call(state, f, args)
                }
            };
            let t = tokio::task::spawn(g());
            let mut hm = HashMap::new();
            hm.insert(0u64, t);
            let mut obj = make_object_base();
            obj.externals.insert(0, Arc::new(Mutex::new(Box::new(hm))));
            init_async();
            let obj = make_container(Value::Object(obj));
            let mut cs = CONTAINERS.get().unwrap().lock().unwrap();
            cs.push(Arc::downgrade(&obj));
            Ok(obj)
        },
        state: state.clone(),
    })))
}

fn await_(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("await requires 1 argument".to_string())))
    }
    let p = args[0].clone();
    let p = match p.lock().unwrap().clone() {
        Value::Object(obj) => obj,
        _ => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = match p.externals.get(&0) {
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
        Some(a) => a,
    };
    let mut p = p.lock().unwrap();
    let p = p.downcast_mut::<HashMap<u64, tokio::task::JoinHandle<Result<Container, Container>>>>();
    let p = match p {
        Some(a) => a,
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = match p.remove(&0) {
        Some(a) => a,
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(p.into_future()).unwrap()
}

fn string(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("string requires 1 argument".to_string())))
    }
    let v = args[0].clone();
    to_string(state.clone(), v)
}
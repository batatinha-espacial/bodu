use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::vm::{make_container, make_err, op::{call, make_object, make_object_base, make_tuple, resolve_bind, set_base, to_number_base, to_string, to_string_base}, Container, Function, Gi, GlobalData, State, StateContainer, Value};

// TODO: add comments

mod array;
mod iter;
mod os;

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
        globaldata: None,
    }));
    let s2 = s.clone();
    s.lock().unwrap().global = Some(s2);
    let gd = Arc::new(Mutex::new(GlobalData {
        threads: Arc::new(Mutex::new(HashMap::new())),
        threadid: 0,
        threadsvec: Vec::new(),
    }));
    s.lock().unwrap().globaldata = Some(gd);
    s
}

pub fn init_global_state(state: StateContainer) {
    let scope = state.lock().unwrap().scope.clone();
    {
        let array_object = make_object();
        make_function!(state, array_object, "new", array::new);
        set_base(state.clone(), scope.clone(), "array".to_string(), array_object).unwrap();
    }
    make_function!(state, scope, "async", async_);
    make_function!(state, scope, "await", await_);
    make_function!(state, scope, "eprint", eprint);
    make_function!(state, scope, "input", input);
    {
        let iter_object = make_object();
        make_function!(state, iter_object, "all", iter::all);
        make_function!(state, iter_object, "any", iter::any);
        make_function!(state, iter_object, "chain", iter::chain);
        make_function!(state, iter_object, "collect", iter::collect);
        make_function!(state, iter_object, "cycle", iter::cycle);
        set_base(state.clone(), scope.clone(), "iter".to_string(), iter_object).unwrap();
    }
    {
        let os_object = make_object();
        make_function!(state, os_object, "name", os::name);
        set_base(state.clone(), scope.clone(), "os".to_string(), os_object).unwrap();
    }
    make_function!(state, scope, "print", print);
    make_function!(state, scope, "range", range);
    make_function!(state, scope, "string", string);
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
            let globaldata = &mut *state.lock().unwrap();
            let globaldata = &mut *globaldata.globaldata.as_mut().unwrap().lock().unwrap();
            let tid = globaldata.threadid;
            globaldata.threadid += 1;
            let t = tokio::task::spawn(g());
            let threads = &mut *globaldata.threads.lock().unwrap();
            threads.insert(tid, t);
            let mut obj = make_object_base();
            obj.externals.insert(0, Arc::new(Mutex::new(Box::new(tid))));
            let obj = make_container(Value::Object(obj));
            Ok(obj)
        },
        state: state.clone(),
    })))
}

fn await_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("await requires 1 argument".to_string())))
    }
    let p = resolve_bind(state.clone(), args[0].clone())?;
    let p = match p.lock().unwrap().clone() {
        Value::Object(obj) => obj,
        _ => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = match p.externals.get(&0) {
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
        Some(a) => a,
    };
    let mut p = p.lock().unwrap();
    let p = p.downcast_mut::<u64>();
    let p = match p {
        Some(a) => *a,
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = {
        let threads = &mut *state.lock().unwrap();
        let threads = &mut *threads.globaldata.as_mut().unwrap().lock().unwrap();
        threads.threadsvec = threads.threadsvec.clone().into_iter().filter(|i| *i != p).collect();
        let threads = &mut *threads.threads.lock().unwrap();
        threads.remove(&p).unwrap()
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

fn range(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("range requires from 1 to 3 arguments"))
    }
    let mut step: i64 = 1;
    let mut start: i64 = 0;
    let stop = if args.len() >= 2 {
        args[1].clone()
    } else {
        args[0].clone()
    };
    let stop = to_number_base(state.clone(), stop)?;
    if args.len() >= 2 {
        start = to_number_base(state.clone(), args[0].clone())?;
    }
    if args.len() >= 3 {
        step = to_number_base(state.clone(), args[2].clone())?;
    }
    let f = {
        let mut internals = HashMap::new();
        let mut obj = make_object_base();
        obj.internals.insert(0, make_container(Value::Number(start)));
        obj.internals.insert(1, make_container(Value::Number(stop)));
        obj.internals.insert(2, make_container(Value::Number(step)));
        internals.insert(0, make_container(Value::Object(obj)));
        Function {
            internals,
            call: |_, _, gi| {
                let obj = gi(0).unwrap().clone();
                let obj = &mut *obj.lock().unwrap();
                let obj = match obj {
                    Value::Object(obj) => obj,
                    _ => return Err(make_err("data corrupted")),
                };
                let start = match obj.internals.get(&0).unwrap().clone().lock().unwrap().clone() {
                    Value::Number(n) => n,
                    _ => return Err(make_err("data corrupted")),
                };
                let stop = match obj.internals.get(&1).unwrap().clone().lock().unwrap().clone() {
                    Value::Number(n) => n,
                    _ => return Err(make_err("data corrupted")),
                };
                let step = match obj.internals.get(&2).unwrap().clone().lock().unwrap().clone() {
                    Value::Number(n) => n,
                    _ => return Err(make_err("data corrupted")),
                };
                if step == 0 {
                    return Err(make_err("a step of 0 was passed into range"))
                }
                let cond = if step > 0 {
                    start < stop
                } else {
                    start > stop
                };
                if cond {
                    obj.internals.insert(0, make_container(Value::Number(start+step)));
                    Ok(make_tuple(vec![make_container(Value::Boolean(true)), make_container(Value::Number(start))]))
                } else {
                    Ok(make_tuple(vec![make_container(Value::Boolean(false)), make_container(Value::Null)]))
                }
            },
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(f)))
}
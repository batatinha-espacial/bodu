use std::{collections::HashMap, io::Write, sync::Arc};

use base64::Engine;

use tokio::sync::Mutex;

use crate::vm::{make_container, make_err, op::{call, make_object, make_object_base, make_tuple, resolve_bind, set_base, to_boolean, to_float, to_number, to_number_base, to_string, to_string_base}, Container, Function, Gi, GlobalData, State, StateContainer, Value};

// TODO: add comments

mod array;
mod iter;
mod os;
mod buffer;
mod json;
mod event;
mod math;

macro_rules! make_function {
    ($state:expr, $scope:expr, $prop:expr, $fcall:expr) => {{
        let f = make_fn!($state, $fcall);
        set_base($state.clone(), $scope.clone(), $prop.to_string(), f).await.unwrap();
    }};
}

macro_rules! make_fn {
    ($state:expr, $fcall:expr) => {{
        make_container(Value::Function(Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
        }))
    }};
}

pub async fn new_global_state(debug: bool) -> StateContainer {
    let s = Arc::new(Mutex::new(State {
        scope: make_object(),
        parent: None,
        global: None,
        globaldata: None,
        debug,
    }));
    let s2 = s.clone();
    s.lock().await.global = Some(s2);
    let gd = Arc::new(Mutex::new(GlobalData {
        threads: HashMap::new(),
        threadid: 0,
        exitcode: 0,
    }));
    s.lock().await.globaldata = Some(gd);
    s
}

pub async fn init_global_state(state: StateContainer) {
    let scope = state.lock().await.scope.clone();
    make_function!(state, scope, "atob", atob);
    {
        let array_object = make_object();
        make_function!(state, array_object, "is_array", array::is_array);
        make_function!(state, array_object, "new", array::new);
        set_base(state.clone(), scope.clone(), "array".to_string(), array_object).await.unwrap();
    }
    make_function!(state, scope, "async", async_);
    make_function!(state, scope, "await", await_);
    make_function!(state, scope, "awaitfn", awaitfn);
    make_function!(state, scope, "bin", bin);
    make_function!(state, scope, "boolean", boolean);
    make_function!(state, scope, "btoa", btoa);
    {
        let buffer_obj = make_object();
        make_function!(state, buffer_obj, "from_string_utf8", buffer::from_string_utf8);
        make_function!(state, buffer_obj, "from_string_utf16be", buffer::from_string_utf16be);
        make_function!(state, buffer_obj, "from_string_utf16le", buffer::from_string_utf16le);
        set_base(state.clone(), scope.clone(), "buffer".to_string(), buffer_obj).await.unwrap();
    }
    make_function!(state, scope, "chr", chr);
    make_function!(state, scope, "eprint", eprint);
    {
        let event_obj = make_object();
        make_function!(state, event_obj, "new", event::new);
        set_base(state.clone(), scope.clone(), "event".to_string(), event_obj).await.unwrap();
    }
    make_function!(state, scope, "float", float);
    make_function!(state, scope, "hex", hex);
    make_function!(state, scope, "hex_upper", hex_upper);
    make_function!(state, scope, "id", id);
    make_function!(state, scope, "input", input);
    {
        let iter_object = make_object();
        make_function!(state, iter_object, "all", iter::all);
        make_function!(state, iter_object, "any", iter::any);
        make_function!(state, iter_object, "chain", iter::chain);
        make_function!(state, iter_object, "collect", iter::collect);
        make_function!(state, iter_object, "cycle", iter::cycle);
        set_base(state.clone(), scope.clone(), "iter".to_string(), iter_object).await.unwrap();
    }
    {
        let json_obj = make_object();
        make_function!(state, json_obj, "decode", json::decode);
        make_function!(state, json_obj, "encode", json::encode);
        set_base(state.clone(), scope.clone(), "json".to_string(), json_obj).await.unwrap();
    }
    {
        let math_obj = make_object();
        make_function!(state, math_obj, "abs", math::abs);
        set_base(state.clone(), scope.clone(), "math".to_string(), math_obj).await.unwrap();
    }
    make_function!(state, scope, "number", number);
    make_function!(state, scope, "oct", oct);
    make_function!(state, scope, "ord", ord);
    {
        let os_object = make_object();
        make_function!(state, os_object, "name", os::name);
        set_base(state.clone(), scope.clone(), "os".to_string(), os_object).await.unwrap();
    }
    make_function!(state, scope, "print", print);
    make_function!(state, scope, "range", range);
    make_function!(state, scope, "sleep", sleep);
    make_function!(state, scope, "string", string);
}

async fn print(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i.await?);
    }
    let args = args2;
    let str = args.join("\t");
    println!("{}", str);
    Ok(make_container(Value::Null))
}

async fn eprint(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i.await?);
    }
    let args = args2;
    let str = args.join("\t");
    eprintln!("{}", str);
    Ok(make_container(Value::Null))
}

async fn input(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i.await?);
    }
    let args = args2;
    let str = args.join("\t");
    print!("{}", str);
    std::io::stdout().flush().unwrap();
    let mut i = String::new();
    std::io::stdin().read_line(&mut i).unwrap();
    Ok(make_container(Value::String(i)))
}

async fn async_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("async requires 1 argument".to_string())))
    }
    let f = args[0].clone();
    let mut internals = HashMap::new();
    internals.insert(0, f);
    Ok(make_container(Value::Function(Function {
        internals,
        call: |state, args, gi| {
            Box::pin(async move {
                let f = gi(0).unwrap();
                let globaldata = &mut *state.lock().await;
                let globaldata = &mut *globaldata.globaldata.as_mut().unwrap().lock().await;
                let tid = globaldata.threadid;
                globaldata.threadid += 1;
                globaldata.threads.insert(tid, tokio::spawn({
                    let f = f.clone();
                    let state = state.clone();
                    let args = args.clone();
                    async move {
                        call(state, f, args).await
                    }
                }));
                let mut obj = make_object_base();
                obj.externals.insert(0, Arc::new(Mutex::new(Box::new(tid))));
                let obj = make_container(Value::Object(obj));
                Ok(obj)
            })
        },
        state: state.clone(),
    })))
}

async fn await_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("await requires 1 argument".to_string())))
    }
    let p = resolve_bind(state.clone(), args[0].clone()).await?;
    let p = match p.lock().await.clone() {
        Value::Object(obj) => obj,
        _ => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = match p.externals.get(&0) {
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
        Some(a) => a,
    };
    let mut p = p.lock().await;
    let p = p.downcast_mut::<u64>();
    let p = match p {
        Some(a) => *a,
        None => return Err(make_container(Value::String("await requires its argument to be a promise".to_string()))),
    };
    let p = {
        let threads = &mut *state.lock().await;
        let threads = &mut *threads.globaldata.as_mut().unwrap().lock().await;
        threads.threads.remove(&p).unwrap()
    };
    p.into_future().await.unwrap()
}

async fn awaitfn(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("awaitfn requires 1 argument"));
    }
    let mut internals = HashMap::new();
    internals.insert(0, args[0].clone());
    let f = make_container(Value::Function(Function {
        internals,
        call: |state, args, gi| {
            Box::pin(async move {
                let r = call(state.clone(), gi(0).unwrap().clone(), args).await?;
                let aw = make_fn!(state, await_);
                call(state.clone(), aw, vec![r]).await
            })
        },
        state: state.clone(),
    }));
    Ok(f)
}

async fn string(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_container(Value::String("string requires 1 argument".to_string())))
    }
    let v = args[0].clone();
    to_string(state.clone(), v).await
}

async fn range(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
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
    let stop = to_number_base(state.clone(), stop).await?;
    if args.len() >= 2 {
        start = to_number_base(state.clone(), args[0].clone()).await?;
    }
    if args.len() >= 3 {
        step = to_number_base(state.clone(), args[2].clone()).await?;
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
                Box::pin(async move {
                    let obj = gi(0).unwrap().clone();
                    let obj = &mut *obj.lock().await;
                    let obj = match obj {
                        Value::Object(obj) => obj,
                        _ => return Err(make_err("data corrupted")),
                    };
                    let start = match obj.internals.get(&0).unwrap().clone().lock().await.clone() {
                        Value::Number(n) => n,
                        _ => return Err(make_err("data corrupted")),
                    };
                    let stop = match obj.internals.get(&1).unwrap().clone().lock().await.clone() {
                        Value::Number(n) => n,
                        _ => return Err(make_err("data corrupted")),
                    };
                    let step = match obj.internals.get(&2).unwrap().clone().lock().await.clone() {
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
                })
            },
            state: state.clone(),
        }
    };
    Ok(make_container(Value::Function(f)))
}

async fn btoa(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("btoa requires 1 argument"));
    }
    let data = {
        let o = args[0].clone();
        let o = match match o.lock().await.clone() {
            Value::Object(o) => Some(o),
            _ => None,
        } {
            Some(o) => o,
            _ => return Err(make_err("btoa requires 1 buffer")),
        };
        let o = match o.externals.get(&0) {
            Some(o) => o.clone(),
            _ => return Err(make_err("btoa requires 1 buffer")),
        };
        let mut o = o.lock().await;
        let o = match o.downcast_mut::<Vec<u8>>() {
            Some(o) => o.clone(),
            _ => return Err(make_err("btoa requires 1 buffer")),
        };
        o
    };

    let output = base64::engine::general_purpose::STANDARD.encode(data);

    Ok(make_container(Value::String(output)))
}

async fn atob(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("atob requires 1 argument"));
    }
    let data = to_string_base(state.clone(), args[0].clone()).await?;

    let output = base64::engine::general_purpose::STANDARD.decode(data).map_err(|_| make_err("error decoding base64 string"))?;

    buffer::new_from_vec(state.clone(), output).await
}

async fn chr(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("chr requires 1 argument"));
    }
    let a = to_number_base(state.clone(), args[0].clone()).await?;
    let a = char::from_u32(a as u32);
    match a {
        Some(a) => Ok(make_container(Value::String(a.to_string()))),
        None => Err(make_err("chr received an invalid codepoint")),
    }
}

async fn ord(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("ord requires 1 argument"));
    }
    let a = to_string_base(state.clone(), args[0].clone()).await?;
    if a.len() == 0 {
        Err(make_err("ord received an empty string"))
    } else {
        Ok(make_container(Value::Number((a.chars().next().unwrap() as u32) as i64)))
    }
}

async fn sleep(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("sleep requires 1 arguemnt"));
    }
    let n = to_number_base(state.clone(), args[0].clone()).await?;
    std::thread::sleep(std::time::Duration::from_millis(n as u64));
    Ok(make_container(Value::Null))
}

async fn id(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("id requires 1 arguemnt"));
    }
    let v = args[0].clone();
    let v = &*v as *const _ as usize;
    Ok(make_container(Value::String(format!("{:x}", v))))
}

async fn bin(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("bin requires 1 arguemnt"));
    }
    let n = to_number_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(format!("{:b}", n))))
}

async fn oct(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("oct requires 1 arguemnt"));
    }
    let n = to_number_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(format!("{:o}", n))))
}

async fn hex(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("hex requires 1 arguemnt"));
    }
    let n = to_number_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(format!("{:x}", n))))
}

async fn hex_upper(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("hex_upper requires 1 arguemnt"));
    }
    let n = to_number_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::String(format!("{:X}", n))))
}

async fn number(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("number requires 1 argument"));
    }
    to_number(state, args[0].clone()).await
}

async fn boolean(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("boolean requires 1 argument"));
    }
    to_boolean(state, args[0].clone()).await
}

async fn float(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("float requires 1 argument"));
    }
    to_float(state, args[0].clone()).await
}
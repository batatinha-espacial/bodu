use bodu_vm::op::{get_base, make_function, new_state, to_boolean_base};
pub use bodu_vm as vm;

use std::{collections::HashMap, io::Write, sync::Arc};

use base64::Engine;

use tokio::sync::Mutex;

use crate::vm::{make_container, make_err, op::{call, make_object, make_object_base, make_tuple, resolve_bind, set_base, to_boolean, to_float, to_number, to_number_base, to_string, to_string_base}, Container, Function, Gi, GlobalData, State, StateContainer, Value};

mod array;
mod buffer;
mod event;
mod iter;
mod json;
mod math;
mod object;
mod os;
mod readline;
mod regex;
mod string;

macro_rules! make_function {
    ($state:expr, $scope:expr, $prop:expr, $fcall:expr, $gdprop:expr) => {{
        let f = make_fn!($state, $fcall);
        {
            let gd = &mut *$state.lock().await;
            let gd = &mut *gd.globaldata.as_mut().unwrap().lock().await;
            gd.register.insert($gdprop.to_string(), f.clone());
        }
        set_base($state.clone(), $scope.clone(), $prop.to_string(), f).await.unwrap();
    }};
}

macro_rules! make_fn {
    ($state:expr, $fcall:expr) => {
        make_container(Value::Function(Function {
            internals: HashMap::new(),
            call: |state, args, gi| {
                Box::pin(async move {
                    $fcall(state, args, gi).await
                })
            },
            state: $state.clone(),
            caller_state: false,
        }))
    };
}

macro_rules! make_function_true {
    ($state:expr, $scope:expr, $prop:expr, $fcall:expr, $gdprop:expr) => {{
        let f = make_fn_true!($state, $fcall);
        {
            let gd = &mut *$state.lock().await;
            let gd = &mut *gd.globaldata.as_mut().unwrap().lock().await;
            gd.register.insert($gdprop.to_string(), f.clone());
        }
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
        threads: Vec::new(),
        threadawaited: HashMap::new(),
        threadresult: HashMap::new(),
        threadid: 0,
        exitcode: 0,
        regex: HashMap::new(),
        gdefers: Vec::new(),
        libid: 0,
        libs: HashMap::new(),
        register: HashMap::new(),
    }));
    s.lock().await.globaldata = Some(gd);
    s
}

pub async fn init_global_state(state: StateContainer) {
    let scope = state.lock().await.scope.clone();
    make_function!(state, scope, "atob", atob, "atob");
    {
        let array_object = make_object();
        make_function!(state, array_object, "is_array", array::is_array, "array.is_array");
        make_function!(state, array_object, "new", array::new, "array.new");
        set_base(state.clone(), scope.clone(), "array".to_string(), array_object).await.unwrap();
    }
    make_function_true!(state, scope, "async", async_, "async");
    make_function!(state, scope, "await", await_, "await");
    make_function_true!(state, scope, "awaitfn", awaitfn, "awaitfn");
    make_function!(state, scope, "bin", bin, "bin");
    make_function!(state, scope, "boolean", boolean, "boolean");
    make_function!(state, scope, "btoa", btoa, "btoa");
    {
        let buffer_obj = make_object();
        make_function!(state, buffer_obj, "from_string_utf8", buffer::from_string_utf8, "buffer.from_string_utf8");
        make_function!(state, buffer_obj, "from_string_utf16be", buffer::from_string_utf16be, "buffer.from_string_utf16be");
        make_function!(state, buffer_obj, "from_string_utf16le", buffer::from_string_utf16le, "from_string_utf16le");
        set_base(state.clone(), scope.clone(), "buffer".to_string(), buffer_obj).await.unwrap();
    }
    make_function!(state, scope, "chr", chr, "chr");
    make_function!(state, scope, "eprint", eprint, "eprint");
    {
        let event_obj = make_object();
        make_function!(state, event_obj, "new", event::new, "event.new");
        set_base(state.clone(), scope.clone(), "event".to_string(), event_obj).await.unwrap();
    }
    make_function!(state, scope, "exec", exec, "exec");
    make_function!(state, scope, "float", float, "float");
    make_function!(state, scope, "from_bin", from_bin, "from_bin");
    make_function!(state, scope, "from_hex", from_hex, "from_hex");
    make_function!(state, scope, "from_oct", from_oct, "from_oct");
    make_function!(state, scope, "hex", hex, "hex");
    make_function!(state, scope, "hex_upper", hex_upper, "hex_upper");
    make_function!(state, scope, "id", id, "id");
    make_function!(state, scope, "input", input, "input");
    {
        let iter_object = make_object();
        make_function_true!(state, iter_object, "all", iter::all, "iter.all");
        make_function_true!(state, iter_object, "any", iter::any, "iter.any");
        make_function_true!(state, iter_object, "chain", iter::chain, "iter.chain");
        make_function_true!(state, iter_object, "collect", iter::collect, "iter.collect");
        make_function_true!(state, iter_object, "count", iter::count, "iter.count");
        make_function_true!(state, iter_object, "cycle", iter::cycle, "iter.cycle");
        make_function_true!(state, iter_object, "enumerate", iter::enumerate, "iter.enumerate");
        make_function_true!(state, iter_object, "filter", iter::filter, "iter.filter");
        make_function_true!(state, iter_object, "map", iter::map, "iter.map");
        make_function_true!(state, iter_object, "reverse", iter::reverse, "iter.reverse");
        set_base(state.clone(), scope.clone(), "iter".to_string(), iter_object).await.unwrap();
    }
    {
        let json_obj = make_object();
        make_function!(state, json_obj, "decode", json::decode, "json.decode");
        make_function!(state, json_obj, "encode", json::encode, "json.encode");
        set_base(state.clone(), scope.clone(), "json".to_string(), json_obj).await.unwrap();
    }
    make_function!(state, scope, "load", load, "load");
    make_function_true!(state, scope, "load_here", load_here, "load_here");
    {
        let math_obj = make_object();
        make_function!(state, math_obj, "abs", math::abs, "math.abs");
        make_function!(state, math_obj, "acos", math::acos, "math.acos");
        make_function!(state, math_obj, "acosh", math::acosh, "math.acosh");
        make_function!(state, math_obj, "asin", math::asin, "math.asin");
        make_function!(state, math_obj, "asinh", math::asinh, "math.asinh");
        make_function!(state, math_obj, "atan", math::atan, "math.atan");
        make_function!(state, math_obj, "atan2", math::atan2, "math.atan2");
        make_function!(state, math_obj, "atanh", math::atanh, "math.atanh");
        make_function!(state, math_obj, "cbrt", math::cbrt, "math.cbrt");
        make_function!(state, math_obj, "ceil", math::ceil, "math.ceil");
        make_function!(state, math_obj, "copysign", math::copysign, "math.copysign");
        make_function!(state, math_obj, "cos", math::cos, "math.cos");
        make_function!(state, math_obj, "cosh", math::cosh, "math.cosh");
        set_base(state.clone(), math_obj.clone(), "epsilon".to_string(), make_container(Value::Float(f64::EPSILON))).await.unwrap();
        make_function!(state, math_obj, "exp", math::exp, "math.exp");
        make_function!(state, math_obj, "exp2", math::exp2, "math.exp2");
        make_function!(state, math_obj, "exp_m1", math::exp_m1, "math.exp_m1");
        make_function!(state, math_obj, "floor", math::floor, "math.floor");
        make_function!(state, math_obj, "fract", math::fract, "math.fract");
        make_function!(state, math_obj, "hypot", math::hypot, "math.hypot");
        set_base(state.clone(), math_obj.clone(), "inf".to_string(), make_container(Value::Float(f64::INFINITY))).await.unwrap();
        make_function!(state, math_obj, "is_finite", math::is_finite, "math.is_finite");
        make_function!(state, math_obj, "is_infinite", math::is_infinite, "math.is_infinite");
        make_function!(state, math_obj, "is_nan", math::is_nan, "math.is_nan");
        make_function!(state, math_obj, "is_normal", math::is_normal, "math.is_normal");
        make_function!(state, math_obj, "is_sign_negative", math::is_sign_negative, "math.is_sign_negative");
        make_function!(state, math_obj, "is_sign_positive", math::is_sign_positive, "math.is_sign_positive");
        make_function!(state, math_obj, "is_subnormal", math::is_subnormal, "math.is_subnormal");
        make_function!(state, math_obj, "ln", math::ln, "math.ln");
        make_function!(state, math_obj, "ln_1p", math::ln_1p, "math.ln_1p");
        make_function!(state, math_obj, "log", math::log, "math.log");
        make_function!(state, math_obj, "log2", math::log2, "math.log2");
        make_function!(state, math_obj, "log10", math::log10, "math.log10");
        set_base(state.clone(), math_obj.clone(), "max".to_string(), make_container(Value::Float(f64::MAX))).await.unwrap();
        set_base(state.clone(), math_obj.clone(), "min".to_string(), make_container(Value::Float(f64::MIN))).await.unwrap();
        set_base(state.clone(), math_obj.clone(), "min_positive".to_string(), make_container(Value::Float(f64::MIN_POSITIVE))).await.unwrap();
        set_base(state.clone(), math_obj.clone(), "nan".to_string(), make_container(Value::Float(f64::NAN))).await.unwrap();
        set_base(state.clone(), math_obj.clone(), "neg_inf".to_string(), make_container(Value::Float(f64::NEG_INFINITY))).await.unwrap();
        make_function!(state, math_obj, "next_down", math::next_down, "math.next_down");
        make_function!(state, math_obj, "next_up", math::next_up, "math.next_up");
        make_function!(state, math_obj, "pow", math::pow, "math.pow");
        make_function!(state, math_obj, "recip", math::recip, "math.recip");
        make_function!(state, math_obj, "round", math::round, "math.round");
        make_function!(state, math_obj, "round_ties_even", math::round_ties_even, "math.round_ties_even");
        make_function!(state, math_obj, "signum", math::signum, "math.signum");
        make_function!(state, math_obj, "sin", math::sin, "math.sin");
        make_function!(state, math_obj, "sinh", math::sinh, "math.sinh");
        make_function!(state, math_obj, "sqrt", math::sqrt, "math.sqrt");
        make_function!(state, math_obj, "tan", math::tan, "math.tan");
        make_function!(state, math_obj, "tanh", math::tanh, "math.tanh");
        make_function!(state, math_obj, "to_degrees", math::to_degrees, "math.to_degrees");
        make_function!(state, math_obj, "to_radians", math::to_radians, "math.to_radian");
        make_function!(state, math_obj, "trunc", math::trunc, "math.trunc");
        set_base(state.clone(), scope.clone(), "math".to_string(), math_obj).await.unwrap();
    }
    make_function!(state, scope, "number", number, "number");
    {
        let obj_obj = make_object();
        make_function!(state, obj_obj, "new", object::new, "object.new");
        set_base(state.clone(), scope.clone(), "object".to_string(), obj_obj).await.unwrap();
    }
    make_function!(state, scope, "oct", oct, "oct");
    make_function!(state, scope, "ord", ord, "ord");
    {
        let os_object = make_object();
        make_function!(state, os_object, "arch", os::arch, "os.arch");
        make_function!(state, os_object, "is_unix", os::is_unix, "os.is_unix");
        make_function!(state, os_object, "name", os::name, "os.name");
        set_base(state.clone(), scope.clone(), "os".to_string(), os_object).await.unwrap();
    }
    make_function!(state, scope, "print", print, "print");
    make_function!(state, scope, "push_gdefer", push_gdefer, "push_gdefer");
    make_function!(state, scope, "range", range, "range");
    {
        let readline_obj = make_object();
        make_function!(state, readline_obj, "new", readline::new, "readline.new");
        set_base(state.clone(), scope.clone(), "readline".to_string(), readline_obj).await.unwrap();
    }
    {
        let regex_obj = make_object();
        make_function!(state, regex_obj, "captures", regex::captures, "regex.captures");
        make_function!(state, regex_obj, "captures_many", regex::captures_many, "regex.captures_many");
        make_function!(state, regex_obj, "find", regex::find, "regex.find");
        make_function!(state, regex_obj, "find_many", regex::find_many, "regex.find_many");
        make_function!(state, regex_obj, "is_match", regex::is_match, "regex.is_match");
        make_function!(state, regex_obj, "replace", regex::replace, "regex.replace");
        make_function!(state, regex_obj, "replace_all", regex::replace_all, "regex.replace_all");
        make_function!(state, regex_obj, "replacen", regex::replacen, "regex.replacen");
        make_function!(state, regex_obj, "split", regex::split, "regex.split");
        make_function!(state, regex_obj, "splitn", regex::splitn, "regex.splitn");
        set_base(state.clone(), scope.clone(), "regex".to_string(), regex_obj).await.unwrap();
    }
    make_function!(state, scope, "sleep", sleep, "sleep");
    make_function!(state, scope, "stderr", stderr, "stderr");
    make_function!(state, scope, "stdout", stdout, "stdout");
    {
        let string_obj = {
            let mut obj = make_object_base();
            let metaobj = make_object();
            let f = make_fn!(state, string);
            {
                let gd = &mut *state.lock().await;
                let gd = &mut *gd.globaldata.as_mut().unwrap().lock().await;
                gd.register.insert("string".to_string(), f.clone());
            }
            set_base(state.clone(), metaobj.clone(), "call".to_string(), f).await.unwrap();
            obj.metaobj = metaobj;
            make_container(Value::Object(obj))
        };
        make_function!(state, string_obj, "chars", string::chars, "string.chars");
        make_function!(state, string_obj, "count_chars", string::count_chars, "string.count_chars");
        make_function!(state, string_obj, "len", string::len, "string.len");
        make_function!(state, string_obj, "ords", string::ords, "string.ords");
        make_function!(state, string_obj, "reverse", string::reverse, "string.reverse");
        make_function!(state, string_obj, "trim", string::trim, "string.trim");
        set_base(state.clone(), scope.clone(), "string".to_string(), string_obj).await.unwrap();
    }
    make_function!(state, scope, "type", type_, "type");
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

async fn stdout(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i.await?);
    }
    let args = args2;
    let str = args.join("\t");
    print!("{}", str);
    std::io::stdout().flush().unwrap();
    Ok(make_container(Value::Null))
}

async fn stderr(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args.iter().map(|a| to_string_base(state.clone(), a.clone())).collect::<Vec<_>>();
    let mut args2 = Vec::new();
    for i in args {
        args2.push(i.await?);
    }
    let args = args2;
    let str = args.join("\t");
    eprint!("{}", str);
    std::io::stderr().flush().unwrap();
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
                let (tx, rx) = std::sync::mpsc::channel();
                globaldata.threadresult.insert(tid, rx);
                globaldata.threads.push(tokio::spawn({
                    let f = f.clone();
                    let state = state.clone();
                    let args = args.clone();
                    async move {
                        let _ = tx.send(call(state, f, args).await);
                    }
                }));
                let mut obj = make_object_base();
                obj.externals.insert(0, Arc::new(Mutex::new(Box::new(tid))));
                let obj = make_container(Value::Object(obj));
                Ok(obj)
            })
        },
        state: state.clone(),
        caller_state: true,
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
        if threads.threadawaited.contains_key(&p) {
            return Err(make_err("promise passed to await was already awaited"));
        }
        threads.threadawaited.insert(p, ());
        threads.threadresult.remove(&p).unwrap()
    };
    p.recv().unwrap()
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
        caller_state: true,
    }));
    Ok(f)
}

async fn string(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    let args = args[1..].to_vec();
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
            caller_state: false,
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

async fn from_bin(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("from_bin requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let n = i64::from_str_radix(&s, 2).map_err(|_| make_err("from_bin couldn't parse the binary number"))?;
    Ok(make_container(Value::Number(n)))
}

async fn from_oct(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("from_oct requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let n = i64::from_str_radix(&s, 8).map_err(|_| make_err("from_oct couldn't parse the binary number"))?;
    Ok(make_container(Value::Number(n)))
}

async fn from_hex(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("from_hex requires 1 argument"));
    }
    let s = to_string_base(state.clone(), args[0].clone()).await?;
    let n = i64::from_str_radix(&s, 16).map_err(|_| make_err("from_hex couldn't parse the binary number"))?;
    Ok(make_container(Value::Number(n)))
}

async fn type_(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("type requires 1 argument"));
    }
    let v = args[0].clone().lock().await.clone();
    Ok(make_container(Value::String(match v {
        Value::Number(_) => "number".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::Null => "null".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Boolean(_) => "boolean".to_string(),
        Value::Object(_) => "object".to_string(),
        Value::Tuple(_) => "tuple".to_string(),
        Value::Function(_) => "function".to_string(),
        Value::Bind(_) => return Err(make_err("type failed to get type of value")),
    })))
}

#[derive(Clone, Copy)]
enum Debug {
    Inherit,
    Debug,
    Release,
}

async fn load(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("load requires 1 argument"));
    }
    let code = to_string_base(state.clone(), args[0].clone()).await?;
    let global = state.as_ref().lock().await.global.clone().unwrap().clone();
    let debug = match args.get(1) {
        None => Debug::Inherit,
        Some(v) => {
            let v = get_base(state.clone(), v.clone(), "dbg".to_string()).await?;
            let v = to_boolean_base(state.clone(), v).await?;
            if v {
                Debug::Debug
            } else {
                Debug::Release
            }
        },
    };
    let code = bodu_script::s1::s1(code).map_err(|s| make_err(&format!("parsing error inside load (S1): {}", s)))?;
    let code = bodu_script::s2::s2(code).map_err(|s| make_err(&format!("parsing error inside load (S2): {}", s)))?;
    let code = bodu_script::s3::s3(code).map_err(|s| make_err(&format!("parsing error inside load (S3): {}", s)))?;
    let code = bodu_script::s4::s4(code).map_err(|s| make_err(&format!("parsing error inside load (S4): {}", s)))?;
    let s = match debug {
        Debug::Inherit => None,
        Debug::Release => {
            let s = new_state(state.clone()).await;
            {
                s.lock().await.debug = false;
            }
            Some(s)
        },
        Debug::Debug => {
            let s = new_state(state.clone()).await;
            {
                s.lock().await.debug = true;
            }
            Some(s)
        },
    };
    let f = make_function(global.clone(), code, s).await?;
    Ok(f)
}

async fn load_here(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("load_here requires 1 argument"));
    }
    let code = to_string_base(state.clone(), args[0].clone()).await?;
    let code = bodu_script::s1::s1(code).map_err(|s| make_err(&format!("parsing error inside load_here (S1): {}", s)))?;
    let code = bodu_script::s2::s2(code).map_err(|s| make_err(&format!("parsing error inside load_here (S2): {}", s)))?;
    let code = bodu_script::s3::s3(code).map_err(|s| make_err(&format!("parsing error inside load_here (S3): {}", s)))?;
    let code = bodu_script::s4::s4(code).map_err(|s| make_err(&format!("parsing error inside load_here (S4): {}", s)))?;
    let f = make_function(state.clone(), code, Some(state.clone())).await?;
    Ok(f)
}

async fn exec(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("run requires 1 argument"));
    }
    let load_ = make_fn!(state, load);
    let f = call(state.clone(), load_, args.clone()).await?;
    call(state.clone(), f, vec![]).await
}

async fn push_gdefer(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("push_gdefer requires 1 argument"));
    }
    {
        let threads = &mut *state.lock().await;
        let threads = &mut *threads.globaldata.as_mut().unwrap().lock().await;
        threads.gdefers.push(args[0].clone());
    }
    Ok(make_container(Value::Null))
}
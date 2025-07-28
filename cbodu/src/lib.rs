use std::{collections::HashMap, ffi::{c_char, c_void, CStr, CString}, sync::Arc};

use bodu_vm::{make_container, op::{add, and, call, detuple, divide, eql, ge, get, gt, isnt_null, le, lt, make_object, make_object_base, make_tuple, multiply, negate, neql, not, or, orthat, remainder, set, subtract, to_boolean_base, to_float_base, to_number_base, to_string_base, xor}, Container, Function, ObjectProp, StateContainer, Value};
use tokio::sync::Mutex;

pub mod op;

pub type CBoduFn = unsafe extern "C" fn(*mut c_void);
pub struct CBoduState {
    pub i: u64,
    pub data: HashMap<u64, CBoduVal>,
    pub ret: Option<Container>,
    pub throw: Option<Container>,
    pub state: StateContainer,
    pub gi: Arc<dyn Fn(u64) -> Option<Container> + Sync + Send>,
    pub args: Vec<Container>,
    pub strs: HashMap<String, CString>,
}

#[derive(Clone, Debug)]
pub enum CBoduVal {
    Val(Container),
    Vec(Vec<Container>),
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_number(state: *mut CBoduState, n: i64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Number(n))));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_float(state: *mut CBoduState, n: f64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Float(n))));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_string(state: *mut CBoduState, s: *const c_char) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    let cstr = unsafe {
        CStr::from_ptr(s)
    };
    state.data.insert(i, CBoduVal::Val(make_container(Value::String(cstr.to_string_lossy().to_string()))));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_boolean(state: *mut CBoduState, b: u8) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    let b = if b % 2 == 0 {
        false
    } else {
        true
    };
    state.data.insert(i, CBoduVal::Val(make_container(Value::Boolean(b))));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_null(state: *mut CBoduState) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_iserr(state: *mut CBoduState) -> u8 {
    let state = unsafe {
        &mut *state
    };
    match state.throw {
        None => 0,
        Some(_) => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_geterr(state: *mut CBoduState) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    let err = match state.throw.clone() {
        None => make_container(Value::Null),
        Some(v) => v.clone(),
    };
    state.data.insert(i, CBoduVal::Val(err));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_seterr(state: *mut CBoduState, i: u64) {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => return,
    };
    state.throw = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_setret(state: *mut CBoduState, i: u64) {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => return,
    };
    state.ret = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_getinternal(state: *mut CBoduState, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let v = match (state.gi)(i) {
        None => make_container(Value::Null),
        Some(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(v));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_argslen(state: *mut CBoduState) -> u64 {
    let state = unsafe {
        &mut *state
    };
    state.args.len() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_getarg(state: *mut CBoduState, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.args.get(i as usize) {
        None => make_container(Value::Null),
        Some(v) => v.clone(),
    };
    let j = state.i;
    state.i += 1;
    state.data.insert(j, CBoduVal::Val(v));
    j
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_tostring(state: *mut CBoduState, i: u64) -> *const c_char {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(to_string_base(state.clone(), v).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    if let Err(e) = v {
        state.throw = Some(e.clone());
        let v = match state.strs.get(&"".to_string()) {
            None => {
                state.strs.insert("".to_string(), CString::from(c""));
                state.strs.get(&"".to_string()).unwrap()
            },
            Some(v) => v,
        };
        return v.as_ptr();
    }
    let v = v.unwrap();
    let v = match state.strs.get(&v.clone()) {
        None => {
            state.strs.insert(v.clone(), CString::new(v.clone()).unwrap());
            state.strs.get(&v.clone()).unwrap()
        },
        Some(v) => v,
    };
    v.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_tonumber(state: *mut CBoduState, i: u64) -> i64 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(to_number_base(state.clone(), v).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    match v {
        Err(e) => {
            state.throw = Some(e.clone());
            0
        },
        Ok(v) => v,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_tofloat(state: *mut CBoduState, i: u64) -> f64 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(to_float_base(state.clone(), v).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    match v {
        Err(e) => {
            state.throw = Some(e.clone());
            0.0
        },
        Ok(v) => v,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_toboolean(state: *mut CBoduState, i: u64) -> u8 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&i) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(to_boolean_base(state.clone(), v).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    match v {
        Err(e) => {
            state.throw = Some(e.clone());
            0
        },
        Ok(v) => if v {
            0
        } else {
            1
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_newobject(state: *mut CBoduState) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_object()));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_get(state: *mut CBoduState, obj: u64, key: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let key = match state.data.get(&key) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(get(state.clone(), obj, key).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    match v {
        Err(v) => {
            state.throw = Some(v.clone());
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
            i
        },
        Ok(v) => {
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(v));
            i
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_set(state: *mut CBoduState, obj: u64, key: u64, val: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let key = match state.data.get(&key) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let val = match state.data.get(&val) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let state = state.state.clone();
        tokio::spawn(async move {
            tx.send(set(state.clone(), obj, key, val).await).unwrap()
        });
    }
    let v = rx.recv().unwrap();
    match v {
        Err(v) => {
            state.throw = Some(v.clone());
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
            i
        },
        Ok(v) => {
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(v));
            i
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_obj_getmetaobj(state: *mut CBoduState, obj: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let obj = &mut *obj.lock().await;
            match obj {
                Value::Object(obj) => tx.send(Some(obj.metaobj.clone())).unwrap(),
                _ => tx.send(None).unwrap(),
            }
        });
        rx.recv().unwrap()
    };
    match r {
        None => {
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
            i
        },
        Some(v) => {
            let i = state.i;
            state.i += 1;
            state.data.insert(i, CBoduVal::Val(v));
            i
        },
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_obj_setmetaobj(state: *mut CBoduState, obj: u64, val: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let val = match state.data.get(&val) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let obj = &mut *obj.lock().await;
            match obj {
                Value::Object(obj) => {
                    obj.metaobj = val;
                },
                _ => {},
            }
            tx.send(()).unwrap()
        });
        rx.recv().unwrap();
    }
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_obj_makegetset(state: *mut CBoduState, obj: u64, key: u64, getter: u64, setter: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let key = match state.data.get(&key) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let getter = match state.data.get(&getter) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let setter = match state.data.get(&setter) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let (tx, rx) = std::sync::mpsc::channel();
        let state = state.state.clone();
        tokio::spawn(async move {
            let key = match to_string_base(state.clone(), key).await {
                Err(v) => {
                    tx.send(Err(v)).unwrap();
                    return;
                },
                Ok(v) => v,
            };
            let obj = &mut *obj.lock().await;
            match obj {
                Value::Object(obj) => {
                    obj.props.insert(key, ObjectProp::GetSet(getter, setter));
                },
                _ => {},
            }
            tx.send(Ok(())).unwrap()
        });
        rx.recv().unwrap()
    };
    if let Err(v) = r {
        state.throw = Some(v);
    }
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_obj_getinternal(state: *mut CBoduState, obj: u64, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let obj = &mut *obj.lock().await;
            match obj {
                Value::Object(obj) => {
                    match obj.internals.get(&i) {
                        None => tx.send(None).unwrap(),
                        Some(v) => tx.send(Some(v.clone())).unwrap(),
                    }
                },
                _ => tx.send(None).unwrap(),
            }
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Some(v) => v,
        None => make_container(Value::Null),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_obj_setinternal(state: *mut CBoduState, obj: u64, i: u64, val: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let obj = match state.data.get(&obj) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let val = match state.data.get(&val) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let obj = &mut *obj.lock().await;
            match obj {
                Value::Object(obj) => {
                    obj.internals.insert(i, val);
                },
                _ => {},
            }
            tx.send(()).unwrap()
        });
        rx.recv().unwrap();
    }
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_newfn(state: *mut CBoduState, f: CBoduFn) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let f = {
        let mut internals = HashMap::new();
        let mut obj = make_object_base();
        obj.externals.insert(0, Arc::new(Mutex::new(Box::new(f))));
        internals.insert(0, make_container(Value::Object(obj)));
        Function {
            internals,
            state: state.state.clone(),
            caller_state: false,
            call: |state, args, gi| {
                Box::pin(async move {
                    let o = gi(0).unwrap();
                    let o = (match o.lock().await.clone() {
                        Value::Object(o) => Some(o),
                        _ => None,
                    }).unwrap();
                    let o = o.externals.get(&0).unwrap().clone();
                    let mut o = o.lock().await;
                    let f = o.downcast_mut::<CBoduFn>().unwrap();
                    let mut s = CBoduState {
                        i: 0,
                        data: HashMap::new(),
                        ret: None,
                        throw: None,
                        gi: gi.clone(),
                        state: state.clone(),
                        args: args.clone(),
                        strs: HashMap::new(),
                    };
                    unsafe {
                        (*f)(&mut s as *mut CBoduState as *mut c_void);
                    }
                    match s.throw {
                        Some(v) => Err(v.clone()),
                        None => match s.ret {
                            Some(v) => Ok(v.clone()),
                            None => Ok(make_container(Value::Null)),
                        },
                    }
                })
            },
        }
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Function(f))));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_setinternal(state: *mut CBoduState, f: u64, i: u64, val: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let f = match state.data.get(&f) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let val = match state.data.get(&val) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let f = &mut *f.lock().await;
            match f {
                Value::Function(f) => {
                    f.internals.insert(i, val);
                },
                _ => {},
            }
            tx.send(()).unwrap()
        });
        rx.recv().unwrap();
    }
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(make_container(Value::Null)));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_add(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(add(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_subtract(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(subtract(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_multiply(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(multiply(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_divide(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(divide(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_remainder(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(remainder(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_negate(state: *mut CBoduState, op: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op = match state.data.get(&op) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(negate(state.clone(), op).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_eql(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(eql(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_neql(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(neql(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_not(state: *mut CBoduState, op: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op = match state.data.get(&op) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(not(state.clone(), op).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_gt(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(gt(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_ge(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(ge(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_lt(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(lt(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_le(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(le(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_and(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(and(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_or(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(or(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_xor(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(xor(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_vec_new(state: *mut CBoduState) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Vec(Vec::new()));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_vec_push(state: *mut CBoduState, i: u64, v: u64) {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&v) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    match state.data.get_mut(&i) {
        Some(CBoduVal::Vec(vec_)) => {
            vec_.push(v);
        },
        _ => {},
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_vec_pop(state: *mut CBoduState, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let r = match state.data.get_mut(&i) {
        Some(CBoduVal::Vec(vec_)) => {
            match vec_.pop() {
                Some(v) => v.clone(),
                _ => make_container(Value::Null)
            }
        },
        _ => make_container(Value::Null),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_vec_len(state: *mut CBoduState, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    match state.data.get(&i) {
        Some(CBoduVal::Vec(vec_)) => {
            vec_.len() as u64
        },
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_vec_get(state: *mut CBoduState, v: u64, i: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let r = match state.data.get(&v) {
        Some(CBoduVal::Vec(vec_)) => {
            match vec_.get(i as usize) {
                Some(v) => v.clone(),
                _ => make_container(Value::Null)
            }
        },
        _ => make_container(Value::Null),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_call(state: *mut CBoduState, f: u64, args: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let f = match state.data.get(&f) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let args = match state.data.get(&args) {
        Some(CBoduVal::Vec(v)) => v.clone(),
        _ => Vec::new(),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(call(state.clone(), f, args).await)
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_debug(state: *mut CBoduState) -> u8 {
    let state = unsafe {
        &mut *state
    };
    let state = state.state.clone();
    let debug = {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(state.lock().await.debug).unwrap();
        });
        rx.recv().unwrap()
    };
    if debug {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_orthat(state: *mut CBoduState, op1: u64, op2: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op1 = match state.data.get(&op1) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let op2 = match state.data.get(&op2) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(orthat(state.clone(), op1, op2).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_isntnull(state: *mut CBoduState, op: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let op = match state.data.get(&op) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(isnt_null(state.clone(), op).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Err(e) => {
            state.throw = Some(e);
            make_container(Value::Null)
        },
        Ok(v) => v.clone(),
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_maketuple(state: *mut CBoduState, v: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&v) {
        Some(CBoduVal::Vec(v)) => v.clone(),
        _ => Vec::new(),
    };
    let r = make_tuple(v);
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_detuple(state: *mut CBoduState, v: u64) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let v = match state.data.get(&v) {
        Some(CBoduVal::Val(v)) => v.clone(),
        _ => make_container(Value::Null),
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            tx.send(detuple(state.clone(), v).await).unwrap();
        });
        rx.recv().unwrap()
    };
    let r = match r {
        Ok(r) => r,
        Err(e) => {
            state.throw = Some(e);
            Vec::new()
        },
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Vec(r));
    i
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_clearerr(state: *mut CBoduState) {
    let state = unsafe {
        &mut *state
    };
    state.throw = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn cbodu_getregister(state: *mut CBoduState, s: *const c_char) -> u64 {
    let state = unsafe {
        &mut *state
    };
    let s = unsafe {
        CStr::from_ptr(s).to_string_lossy().to_string()
    };
    let r = {
        let state = state.state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::spawn(async move {
            let gd = &mut *state.lock().await;
            let gd = &mut *gd.globaldata.as_mut().unwrap().lock().await;
            tx.send(match gd.register.get(&s) {
                Some(v) => v.clone(),
                None => make_container(Value::Null),
            }).unwrap();
        });
        rx.recv().unwrap()
    };
    let i = state.i;
    state.i += 1;
    state.data.insert(i, CBoduVal::Val(r));
    i
}
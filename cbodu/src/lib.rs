use std::{collections::HashMap, ffi::{c_char, c_void, CStr, CString}, sync::Arc};

use bodu_vm::{make_container, op::{get, make_object, set, to_boolean_base, to_float_base, to_number_base, to_string_base}, Container, ObjectProp, StateContainer, Value};

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
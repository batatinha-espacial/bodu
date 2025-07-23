use std::{collections::HashMap, ffi::c_void, sync::Arc};

use bodu_vm::{make_container, make_err, op::make_object_base, Container, Function, StateContainer, Value};
use libloading::{Library, Symbol};
use tokio::sync::Mutex;

use crate::{CBoduFn, CBoduState};

pub async fn load_lib(state: StateContainer, name: String) -> Result<Container, Container> {
    let lib = unsafe {
        Library::new(name)
    };
    let lib = lib.map_err(|_| make_err("loaded invalid library"))?;
    let f = {
        let mut internals = HashMap::new();
        let mut obj = make_object_base();
        obj.externals.insert(0, Arc::new(Mutex::new(Box::new(lib))));
        internals.insert(u64::MAX, make_container(Value::Object(obj)));
        Function {
            internals,
            call: |state, args, gi| {
                Box::pin(async move {
                    let o = gi(u64::MAX).unwrap();
                    let o = (match o.lock().await.clone() {
                        Value::Object(o) => Some(o),
                        _ => None,
                    }).unwrap();
                    let o = o.externals.get(&0).unwrap().clone();
                    let mut o = o.lock().await;
                    let o = o.downcast_mut::<Library>().unwrap();
                    let f: Symbol<'_, CBoduFn> = unsafe {
                        match o.get(b"cbodu_main").map_err(|_| make_err("loaded invalid library")) {
                            Ok(f) => f,
                            Err(v) => return Err(v),
                        }
                    };
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
            state: state.clone(),
            caller_state: true,
        }
    };
    Ok(make_container(Value::Function(f)))
}
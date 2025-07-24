use std::{collections::HashMap, ffi::c_void, sync::Arc};

use bodu_vm::{make_container, make_err, op::to_number_base, Container, Function, StateContainer, Value};
use libloading::{Library, Symbol};

use crate::{CBoduFn, CBoduState};

pub async fn load_lib(state: StateContainer, name: String) -> Result<Container, Container> {
    let lib = unsafe {
        Library::new(name)
    };
    let lib = lib.map_err(|_| make_err("loaded invalid library"))?;
    let lid = {
        let globaldata = &mut *state.lock().await;
        let globaldata = &mut *globaldata.globaldata.as_mut().unwrap().lock().await;
        let id = globaldata.libid;
        globaldata.libid += 1;
        globaldata.libs.insert(id, Arc::new(lib));
        id
    };
    let f = {
        let mut internals = HashMap::new();
        internals.insert(u64::MAX, make_container(Value::Number(lid as i64)));
        Function {
            internals,
            call: |state, args, gi| {
                Box::pin(async move {
                    let o = gi(u64::MAX).unwrap();
                    let o = to_number_base(state.clone(), o).await?;
                    let o = {
                        let globaldata = &*state.lock().await;
                        let globaldata = &*globaldata.globaldata.as_ref().unwrap().lock().await;
                        let o = globaldata.libs.get(&(o as u64)).unwrap();
                        o.clone()
                    };
                    let f: Symbol<'_, CBoduFn> = unsafe {
                        o.get(b"cbodu_main").map_err(|_| make_err("loaded invalid library"))?
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
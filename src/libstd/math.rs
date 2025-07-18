use crate::vm::{make_container, make_err, op::to_float_base, Container, Gi, StateContainer, Value};

pub async fn abs(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.abs requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.abs())))
}
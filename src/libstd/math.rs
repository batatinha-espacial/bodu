use crate::vm::{make_container, make_err, op::to_float_base, Container, Gi, StateContainer, Value};

pub async fn abs(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.abs requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.abs())))
}

pub async fn acos(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.acos requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.acos())))
}

pub async fn acosh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.acosh requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.acosh())))
}

pub async fn asin(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.asin requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.asin())))
}

pub async fn asinh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.asinh requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.asinh())))
}

pub async fn atan(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.atan requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.atan())))
}

pub async fn atan2(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("math.atan2 requires 2 arguments"));
    }
    let a = to_float_base(state.clone(), args[0].clone()).await?;
    let b = to_float_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Float(a.atan2(b))))
}

pub async fn atanh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.atanh requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.atanh())))
}

pub async fn cbrt(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.cbrt requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.cbrt())))
}

pub async fn ceil(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.ceil requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.ceil())))
}

pub async fn copysign(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("math.copysign requires 2 arguments"));
    }
    let a = to_float_base(state.clone(), args[0].clone()).await?;
    let b = to_float_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Float(a.copysign(b))))
}

pub async fn cos(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.cos requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.cos())))
}

pub async fn cosh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.cosh requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.cosh())))
}

pub async fn exp(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.exp requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.exp())))
}

pub async fn exp2(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.exp2 requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.exp2())))
}

pub async fn exp_m1(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.exp_m1 requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.exp_m1())))
}

pub async fn floor(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.floor requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.floor())))
}

pub async fn fract(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.fract requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.fract())))
}

pub async fn hypot(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("math.hypot requires 2 arguments"));
    }
    let a = to_float_base(state.clone(), args[0].clone()).await?;
    let b = to_float_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Float(a.hypot(b))))
}

pub async fn is_finite(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_finite requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_finite())))
}

pub async fn is_infinite(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_infinite requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_infinite())))
}

pub async fn is_nan(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_nan requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_nan())))
}

pub async fn is_normal(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_normal requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_normal())))
}

pub async fn is_sign_negative(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_sign_negative requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_sign_negative())))
}

pub async fn is_sign_positive(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_sign_positive requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_sign_positive())))
}

pub async fn is_subnormal(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.is_subnormal requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Boolean(f.is_subnormal())))
}

pub async fn ln(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.ln requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.ln())))
}

pub async fn ln_1p(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.ln_1p requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.ln_1p())))
}

pub async fn log(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("math.log requires 2 arguments"));
    }
    let a = to_float_base(state.clone(), args[0].clone()).await?;
    let b = to_float_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Float(a.log(b))))
}

pub async fn log2(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.log2 requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.log2())))
}

pub async fn log10(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.log10 requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.log10())))
}

pub async fn next_down(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.next_down requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.next_down())))
}

pub async fn next_up(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.next_up requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.next_up())))
}

pub async fn pow(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("math.pow requires 2 arguments"));
    }
    let a = to_float_base(state.clone(), args[0].clone()).await?;
    let b = to_float_base(state.clone(), args[1].clone()).await?;
    Ok(make_container(Value::Float(a.powf(b))))
}

pub async fn recip(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.recip requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.recip())))
}

pub async fn round(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.round requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.round())))
}

pub async fn round_ties_even(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.round_ties_even requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.round_ties_even())))
}

pub async fn signum(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.signum requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.signum())))
}

pub async fn sin(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.sin requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.sin())))
}

pub async fn sinh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.sinh requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.sinh())))
}

pub async fn sqrt(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.sqrt requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.sqrt())))
}

pub async fn tan(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.tan requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.tan())))
}

pub async fn tanh(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.sqrt requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.tanh())))
}

pub async fn to_degrees(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.to_degrees requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.to_degrees())))
}

pub async fn to_radians(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.to_radians requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.to_radians())))
}

pub async fn trunc(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("math.trunc requires 1 argument"));
    }
    let f = to_float_base(state.clone(), args[0].clone()).await?;
    Ok(make_container(Value::Float(f.trunc())))
}
use crate::vm::{make_err, op::{add, and, call, divide, eql, ge, get, gt, le, lt, make_tuple, multiply, negate, neql, not, or, orthat, remainder, subtract, to_boolean_base, xor}, Container, Gi, StateContainer};

pub async fn plus(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[+] requires 2 arguments"));
    }
    add(state, args[0].clone(), args[1].clone()).await
}

pub async fn minus(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("[-] requires either 1 or 2 arguemnts"));
    }
    if args.len() == 1 {
        negate(state, args[0].clone()).await
    } else {
        subtract(state, args[0].clone(), args[1].clone()).await
    }
}

pub async fn times(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[*] requires 2 arguments"));
    }
    multiply(state, args[0].clone(), args[1].clone()).await
}

pub async fn divide_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[/] requires 2 arguments"));
    }
    divide(state, args[0].clone(), args[1].clone()).await
}

pub async fn modulus(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[%] requires 2 arguments"));
    }
    remainder(state, args[0].clone(), args[1].clone()).await
}

pub async fn orthat_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[??] requires 2 arguments"));
    }
    orthat(state, args[0].clone(), args[1].clone()).await
}

pub async fn ternary(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 3 {
        return Err(make_err("[?:] requires 3 arguments"));
    }
    if to_boolean_base(state.clone(), args[0].clone()).await? {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

pub async fn equalto(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[==] requires 2 arguments"));
    }
    eql(state, args[0].clone(), args[1].clone()).await
}

pub async fn not_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("[!] requires 1 argument"));
    }
    not(state, args[0].clone()).await
}

pub async fn notequalto(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[!=] requires 2 arguments"));
    }
    neql(state, args[0].clone(), args[1].clone()).await
}

pub async fn less(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[<] requires 2 arguments"));
    }
    lt(state, args[0].clone(), args[1].clone()).await
}

pub async fn lessorequal(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[<=] requires 2 arguments"));
    }
    le(state, args[0].clone(), args[1].clone()).await
}

pub async fn greater(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[>] requires 2 arguments"));
    }
    gt(state, args[0].clone(), args[1].clone()).await
}

pub async fn greaterorequal(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[>=] requires 2 arguments"));
    }
    ge(state, args[0].clone(), args[1].clone()).await
}

pub async fn and_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[&] requires 2 arguments"));
    }
    and(state, args[0].clone(), args[1].clone()).await
}

pub async fn or_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[|] requires 2 arguments"));
    }
    or(state, args[0].clone(), args[1].clone()).await
}

pub async fn xor_(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[^] requires 2 arguments"));
    }
    xor(state, args[0].clone(), args[1].clone()).await
}

pub async fn property(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[.] requires 2 arguments"));
    }
    get(state, args[0].clone(), args[1].clone()).await
}

pub async fn tuple(_: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() == 0 {
        return Err(make_err("[,] requires at least 1 argument"));
    }
    Ok(make_tuple(args.clone()))
}

pub async fn pipe(state: StateContainer, args: Vec<Container>, _: Gi) -> Result<Container, Container> {
    if args.len() < 2 {
        return Err(make_err("[|>] requires 2 arguments"));
    }
    call(state, args[1].clone(), vec![args[0].clone()]).await
}
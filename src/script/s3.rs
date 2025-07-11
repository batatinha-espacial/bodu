#[derive(Clone, PartialEq)]
pub enum S3T {
    Identifier(String),
    Let(String, Option<Box<S3T>>), // let: name, expr
    If(Vec<(ConditionType, Vec<S3T>)>, Option<Box<S3T>>), // if, unless, else if, else unless, else: ...if/unless, else
    Boolean(bool), // true, false
    Block(Vec<S3T>), // { ...expr/stat }
    Out(Box<S3T>), // out: expr
    Label(String), // #label
    Number(i64),
    Float(f64),
    String(String),
    TryCatchFinally(Vec<S3T>, Vec<S3T>, Option<Vec<S3T>>), // try, catch, finally: try { ... } catch { ... } finally { ... }
    Return(Box<S3T>), // return: expr
    Throw(Box<S3T>), // throw: expr
    Loop(Vec<S3T>), // loop: loop { ... }
    WhileUntil(LoopType, Box<S3T>, Vec<S3T>), // while, until: while/until, condition, { ... }
    Defer(Vec<S3T>), // defer: { ... }
    Bind(String, Box<S3T>), // bind: name, expr
    Function(Option<String>, Vec<String>, Vec<S3T>), // fn: name, ...args, { ... }
    FnShorthand(Box<S3T>), // $: expr
    FnShortArg, // $
    Assign(Box<S3T>, Box<S3T>), // expr = expr
    Plus(Box<S3T>, Box<S3T>), // expr + expr
    Minus(Box<S3T>, Box<S3T>), // expr - expr
    Negate(Box<S3T>), // -expr
    Times(Box<S3T>, Box<S3T>), // expr * expr
    Divide(Box<S3T>, Box<S3T>), // expr / expr
    Modulus(Box<S3T>, Box<S3T>), // expr % expr
    Ternary(Box<S3T>, Box<S3T>, Box<S3T>), // expr ? expr : expr
    EqualTo(Box<S3T>, Box<S3T>), // expr == expr
    Not(Box<S3T>), // !expr
    NotEqualTo(Box<S3T>, Box<S3T>), // expr != expr
    Less(Box<S3T>, Box<S3T>), // expr < expr
    LessOrEqual(Box<S3T>, Box<S3T>), // expr <= expr
    Greater(Box<S3T>, Box<S3T>), // expr > expr
    GreaterOrEqual(Box<S3T>, Box<S3T>), // expr >= expr
    And(Box<S3T>, Box<S3T>), // expr & expr
    Or(Box<S3T>, Box<S3T>), // expr | expr
    Xor(Box<S3T>, Box<S3T>), // expr ^ expr
    Property(Box<S3T>, Box<S3T>), // expr[prop], expr.prop
    Tuple(Vec<S3T>), // expr1, expr2, expr3, ...
    Detuple(Vec<String>, Box<S3T>), // (var1, var2, var3, ...) = expr
    FnCall(Box<S3T>, Vec<S3T>), // expr(arg1, arg2, arg3, ...)
    Decorator(Box<S3T>, Box<S3T>), // @expr
    Pipe(Box<S3T>, Box<S3T>), // expr |> f
    Goto(String), // goto #name
    OrThat(Box<S3T>, Box<S3T>), // expr ?? expr
    OperatorFn(Operator), // wrap the operator in brackets
}

#[derive(Clone, Copy, PartialEq)]
pub enum ConditionType {
    If,
    Unless,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LoopType {
    While,
    Until,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Operator {
    Plus, // +
    Minus, // -
    Times, // *
    Divide, // /
    Modulus, // %
    OrThat, // ??
    Ternary, // ?:
    EqualTo, // ==
    Not, // !
    NotEqualTo, // !=
    Less, // <
    LessOrEqual, // <=
    Greater, // >
    GreaterOrEqual, // >=
    And, // &
    Or, // |
    Xor, // ^
    Property, // . []
    Tuple, // ,
    Decorator, // @
    Pipe, // |>
}
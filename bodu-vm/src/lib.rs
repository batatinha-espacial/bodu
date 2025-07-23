use std::{any::Any, collections::HashMap, pin::Pin, sync::Arc};

use tokio::{sync::Mutex, task::JoinHandle};

pub mod op; // standard operations that bodu code can do
pub mod opfn;

// bodu values. those are often wrapped in Arc<Mutex<T>>s.
#[derive(Clone, Debug)]
pub enum Value {
    Number(i64), // integers
    Float(f64), // floats
    Null, // null
    String(String), // strings
    Boolean(bool), // booleans
    Object(Object), // objects
    Tuple(Vec<Container>), // tuples
    Bind(Container), // binds are like getter/setter properties, but they are variables and only have a getter.
    Function(Function), // functions
}

// objects
#[derive(Clone, Debug)]
pub struct Object {
    pub props: HashMap<String, ObjectProp>, // properties
    pub internals: HashMap<u64, Container>, // internal values used by the object
    pub metaobj: Container, // meta objects are objects that define how the object behaves (like for operator overloading)
    pub externals: HashMap<u64, SharedAny>, // internal values that aren't bodu language values
}

// object properties
#[derive(Clone, Debug)]
pub enum ObjectProp {
    Value(Container), // value properties
    GetSet(Container, Container), // getter/setter properties
}

// gets the function's internal values
pub type Gi = Arc<dyn Fn(u64) -> Option<Container> + Sync + Send>;

// functions
#[derive(Clone, Debug)]
pub struct Function {
    pub internals: HashMap<u64, Container>, // internal values used by the function
    pub call: fn(StateContainer, Vec<Container>, Gi) -> Pin<Box<dyn Future<Output = Result<Container, Container>> + Send>>, // the actual function
    pub state: StateContainer, // the state it runs on
    pub caller_state: bool, // if true, the state of the caller is passed. otherwise, the state field is passed.
}

// Arc<Mutex<Value>>, used almost everywhere in the codebase
pub type Container = Arc<Mutex<Value>>;

// creates a new container
pub fn make_container(v: Value) -> Container {
    Container::new(Mutex::new(v))
}

// the state: it contains the scope of a function and it is an execution context (kinda)
#[derive(Debug, Clone)]
pub struct State {
    pub scope: Container, // the scope
    pub parent: Option<StateContainer>, // parent state
    pub global: Option<StateContainer>, // global state
    pub globaldata: Option<Arc<Mutex<GlobalData>>>, // global data
    pub debug: bool, // whether debug mode is active
}

#[derive(Debug)]
pub struct GlobalData {
    pub threads: Vec<JoinHandle<()>>,
    pub threadawaited: HashMap<u64, ()>,
    pub threadresult: HashMap<u64, std::sync::mpsc::Receiver<Result<Container, Container>>>,
    pub threadid: u64,
    pub exitcode: u8,
    pub regex: HashMap<String, Result<regex::Regex, ()>>,
    pub gdefers: Vec<Container>,
}

// Container but for States
pub type StateContainer = Arc<Mutex<State>>;

// instructions for the VM
#[derive(Clone, Debug)]
pub enum Instruction {
    Add(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Return(VarIndex), // op
    Throw(VarIndex), // op
    Call(VarIndex, VarIndex, Vec<VarIndex>), // result, f, ...args
    Get(VarIndex, VarIndex, VarIndex), // result, obj, prop
    Multiply(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Negate(VarIndex, VarIndex), // result, op
    Subtract(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Has(VarIndex, VarIndex, VarIndex), // result, obj, prop
    Set(VarIndex, VarIndex, VarIndex, VarIndex), // result, obj, prop, value
    Decl(VarIndex), // op
    Label(Label),
    Goto(Label),
    Eql(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Neql(VarIndex, VarIndex, VarIndex), // result, op1, op2
    GotoIf(Label, VarIndex),
    Block(Vec<Instruction>),
    MakeTuple(VarIndex, Vec<VarIndex>), // result, ...ops
    DeTuple(Vec<VarIndex>, VarIndex), // ...results, op
    Divide(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Remainder(VarIndex, VarIndex, VarIndex), // result, op1, op2
    MakeBind(VarIndex, VarIndex), // result, f
    Catch(VarIndex, VarIndex, Vec<Instruction>), // err?, err, block
    Assign(VarIndex, VarIndex), // result, op
    Defer(Vec<Instruction>),
    Boolean(VarIndex, bool), // result, op
    Number(VarIndex, i64), // result, op
    Float(VarIndex, f64), // result, op
    String(VarIndex, String), // result, op
    MakeFunction(VarIndex, Vec<Instruction>), // result, body
    Not(VarIndex, VarIndex), // result, op
    Gt(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Ge(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Lt(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Le(VarIndex, VarIndex, VarIndex), // result, op1, op2
    And(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Or(VarIndex, VarIndex, VarIndex), // result, op1, op2
    Xor(VarIndex, VarIndex, VarIndex), // result, op1, op2
    GetPipeShorthand(VarIndex), // result
    SetPipeShorthand(VarIndex), // op
    OrThat(VarIndex, VarIndex, VarIndex), // result, op1, op2
    OperatorFn(VarIndex, Operator), // result, opfn
    Debug(VarIndex), // result
    Release(VarIndex), // result
    Maybe(VarIndex), // result
    ToNumber(VarIndex, VarIndex), // result, op
    Iterate(VarIndex, VarIndex, VarIndex), // r1, r2, it
}

#[derive(Clone, Debug)]
pub enum VarIndex {
    Arg(usize),
    Ident(String),
    Temp(u64),
}

#[derive(Clone, Debug)]
pub enum Label {
    Named(String),
    Unnamed(u64),
}

// type for object externals
pub type SharedAny = Arc<Mutex<Box<dyn Any + Send + Sync>>>;

// makes an error, could be a macro
pub fn make_err(v: &str) -> Container {
    make_container(Value::String(v.to_string()))
}

// used for operator functions
#[derive(Clone, Copy, PartialEq, Debug)]
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
    Property, // .
    Tuple, // ,
    Pipe, // |>
}
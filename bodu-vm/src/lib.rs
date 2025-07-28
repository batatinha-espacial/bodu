use std::{any::Any, collections::HashMap, pin::Pin, sync::Arc};

use libloading::Library;
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
    pub metalocked: bool, // true if you can't change the metaobj, false otherwise
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
    pub libid: u64,
    pub libs: HashMap<u64, Arc<Library>>,
    pub register: HashMap<String, Container>,
}

// Container but for States
pub type StateContainer = Arc<Mutex<State>>;

// instructions for the VM
#[derive(Clone, Debug)]
pub enum Instruction {
    Add(VarIndex, VarIndex, VarIndex), // 1: result, op1, op2
    Return(VarIndex), // 2: op
    Throw(VarIndex), // 3: op
    Call(VarIndex, VarIndex, Vec<VarIndex>), // 4: result, f, ...args
    Get(VarIndex, VarIndex, VarIndex), // 5: result, obj, prop
    Multiply(VarIndex, VarIndex, VarIndex), // 6: result, op1, op2
    Negate(VarIndex, VarIndex), // 7: result, op
    Subtract(VarIndex, VarIndex, VarIndex), // 8: result, op1, op2
    Has(VarIndex, VarIndex, VarIndex), // 9: result, obj, prop
    Set(VarIndex, VarIndex, VarIndex, VarIndex), // A: result, obj, prop, value
    Decl(VarIndex), // B: op
    Label(Label), // C
    Goto(Label), // D
    Eql(VarIndex, VarIndex, VarIndex), // E: result, op1, op2
    Neql(VarIndex, VarIndex, VarIndex), // F: result, op1, op2
    GotoIf(Label, VarIndex), // 10
    Block(Vec<Instruction>), // 11
    MakeTuple(VarIndex, Vec<VarIndex>), // 12: result, ...ops
    DeTuple(Vec<VarIndex>, VarIndex), // 13: ...results, op
    Divide(VarIndex, VarIndex, VarIndex), // 14: result, op1, op2
    Remainder(VarIndex, VarIndex, VarIndex), // 15: result, op1, op2
    MakeBind(VarIndex, VarIndex), // 16: result, f
    Catch(VarIndex, VarIndex, Vec<Instruction>), // 17: err?, err, block
    Assign(VarIndex, VarIndex), // 18: result, op
    Defer(Vec<Instruction>), // 19
    Boolean(VarIndex, bool), // 1A: result, op
    Number(VarIndex, i64), // 1B: result, op
    Float(VarIndex, f64), // 1C: result, op
    String(VarIndex, String), // 1D: result, op
    MakeFunction(VarIndex, Vec<Instruction>), // 1E: result, body
    Not(VarIndex, VarIndex), // 1F: result, op
    Gt(VarIndex, VarIndex, VarIndex), // 20: result, op1, op2
    Ge(VarIndex, VarIndex, VarIndex), // 21: result, op1, op2
    Lt(VarIndex, VarIndex, VarIndex), // 22: result, op1, op2
    Le(VarIndex, VarIndex, VarIndex), // 23: result, op1, op2
    And(VarIndex, VarIndex, VarIndex), // 24: result, op1, op2
    Or(VarIndex, VarIndex, VarIndex), // 25: result, op1, op2
    Xor(VarIndex, VarIndex, VarIndex), // 26: result, op1, op2
    GetPipeShorthand(VarIndex), // 27: result
    SetPipeShorthand(VarIndex), // 28: op
    OrThat(VarIndex, VarIndex, VarIndex), // 29: result, op1, op2
    OperatorFn(VarIndex, Operator), // 2A: result, opfn
    Debug(VarIndex), // 2B: result
    Release(VarIndex), // 2C: result
    Maybe(VarIndex), // 2D: result
    ToNumber(VarIndex, VarIndex), // 2E: result, op
    Iterate(VarIndex, VarIndex, VarIndex), // 2F: r1, r2, it
    Probably(VarIndex), // 30: result
    Possibly(VarIndex), // 31: result
    IsntNull(VarIndex, VarIndex), // 32: result, op
}

#[derive(Clone, Debug)]
pub enum VarIndex {
    Arg(usize), // 0
    Ident(String), // 1
    Temp(u64), // 2
}

#[derive(Clone, Debug)]
pub enum Label {
    Named(String), // 0
    Unnamed(u64), // 1
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
    Plus, // 0: +
    Minus, // 1: -
    Times, // 2: *
    Divide, // 3: /
    Modulus, // 4: %
    OrThat, // 5: ??
    Ternary, // 6: ?:
    EqualTo, // 7: ==
    Not, // 8: !
    NotEqualTo, // 9: !=
    Less, // A: <
    LessOrEqual, // B: <=
    Greater, // C: >
    GreaterOrEqual, // D: >=
    And, // E: &
    Or, // F: |
    Xor, // 10: ^
    Property, // 11: .
    Tuple, // 12: ,
    Pipe, // 13: |>
    IsntNull, // 14: ?
}
use bodu::{libstd::{init_global_state, new_global_state}, script::{s1::s1, s2::s2, s3::s3, s4::s4}, vm::op::{call, make_function}};

static HELP_MSG: &str = r#"Commands:
run <file> - Run a Bodu file
repl       - Start the Bodu REPL
help       - Show this help message
version    - Show the version of Bodu installed
--version  - Alias of `version`"#;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("{}", HELP_MSG);
        return;
    }

    let action = &args[1];

    match action.as_str() {
        "run" => {
            if args.len() < 3 {
                println!("{}", HELP_MSG);
                return;
            }
            interpret(args[2].clone());
        },
        "repl" => todo!("implement repl"),
        "help" => println!("{}", HELP_MSG),
        "version" | "--version" => println!("Bodu 0.1.0"),
        _ => println!("Invalid action. Run `bodu help` for available commands"),
    }
}

fn interpret(file: String) {
    let contents = std::fs::read_to_string(file).unwrap();
    let contents = s1(contents).unwrap();
    let contents = s2(contents).unwrap();
    let contents = s3(contents).unwrap();
    let instrs = s4(contents).unwrap();
    let state = new_global_state();
    init_global_state(state.clone());
    let f = make_function(state.clone(), instrs).unwrap();
    call(state.clone(), f, vec![]).unwrap();
}
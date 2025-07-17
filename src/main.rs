use bodu::{libstd::{init_global_state, new_global_state}, script::{s1::s1, s2::s2, s3::s3, s4::s4}, vm::op::{call, make_function, new_state, to_string_base}};
use clap::{Arg, Command};
use rustyline::DefaultEditor;

fn main() {
    let mut cmd = Command::new("bodu")
        .subcommand(
            Command::new("run")
                .arg(
                    Arg::new("file")
                        .required(true)
                ).about("run a bodu file")
        ).subcommand(
            Command::new("repl")
            .about("start the bodu repl")
        ).subcommand(
            Command::new("version")
                .visible_alias("--version")
                .about("print version and exit")
        ).subcommand_required(true);
    let matches = cmd.clone().get_matches();
    if let Some(_) = matches.subcommand_matches("version") {
        println!("Bodu 0.1.0");
    } else if let Some(_) = matches.subcommand_matches("repl") {
        repl();
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let file = matches.get_one::<String>("file").unwrap();
        interpret(file.clone());
    } else {
        cmd.print_help().unwrap();
    }
}

fn interpret(file: String) {
    let contents = std::fs::read_to_string(file).unwrap();
    let contents = s1(contents).unwrap();
    let contents = s2(contents).unwrap();
    let contents = s3(contents).unwrap();
    let instrs = s4(contents).unwrap();
    let state = new_global_state(false);
    init_global_state(state.clone());
    let f = make_function(state.clone(), instrs, None).unwrap();
    call(state.clone(), f, vec![]).unwrap();
}

fn repl() {
    println!("Welcome to the Bodu REPL!");
    let state = new_global_state(false);
    init_global_state(state.clone());
    let s = new_state(state.clone());
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let line = rl.readline(">> ");
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                let line = match s1(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S1): {}", s);
                        continue;
                    },
                };
                let line = match s2(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S2): {}", s);
                        continue;
                    },
                };
                let line = match s3(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S3): {}", s);
                        continue;
                    },
                };
                let line = match s4(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S4): {}", s);
                        continue;
                    },
                };
                let f = match make_function(state.clone(), line, Some(s.clone())) {
                    Ok(f) => f,
                    Err(e) => {
                        let e = to_string_base(state.clone(), e).unwrap();
                        println!("Error while compiling into function: {}", e);
                        continue;
                    },
                };
                match call(state.clone(), f, vec![]) {
                    Ok(_) => {},
                    Err(e) => {
                        let e = match to_string_base(state.clone(), e) {
                            Ok(e) => e,
                            Err(_) => {
                                println!("Error while converting error to string.");
                                continue;
                            },
                        };
                        println!("Runtime error: {}", e);
                        continue;
                    },
                }
            },
            _ => break,
        }
    }
}
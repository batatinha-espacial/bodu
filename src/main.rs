use bodu::{libstd::{init_global_state, new_global_state}, script::{s1::s1, s2::s2, s3::s3, s4::s4}, vm::op::{call, make_function, new_state, to_string_base}};
use clap::{Arg, Command};
use rustyline::DefaultEditor;

#[tokio::main]
async fn main() {
    let local = tokio::task::LocalSet::new();
    local.enter();
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
        repl().await;
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let file = matches.get_one::<String>("file").unwrap();
        interpret(file.clone()).await;
    } else {
        cmd.print_help().unwrap();
    }
}

static D: bool = false; // change this if you need to debug the parser

async fn interpret(file: String) {
    let contents = std::fs::read_to_string(file).unwrap();
    let contents = s1(contents).unwrap();
    if D {
        println!("S1: {:#?}", contents);
    }
    let contents = s2(contents).unwrap();
    if D {
        println!("S2: {:#?}", contents);
    }
    let contents = s3(contents).unwrap();
    if D {
        println!("S3: {:#?}", contents);
    }
    let instrs = s4(contents).unwrap();
    if D {
        println!("S4: {:#?}", instrs);
    }
    let state = new_global_state(false).await;
    init_global_state(state.clone()).await;
    let f = make_function(state.clone(), instrs, None).await.unwrap();
    call(state.clone(), f, vec![]).await.unwrap();
}

async fn repl() {
    println!("Welcome to the Bodu REPL!");
    let state = new_global_state(false).await;
    init_global_state(state.clone()).await;
    let s = new_state(state.clone()).await;
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
                if D {
                    println!("S1: {:#?}", line);
                }
                let line = match s2(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S2): {}", s);
                        continue;
                    },
                };
                if D {
                    println!("S2: {:#?}", line);
                }
                let line = match s3(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S3): {}", s);
                        continue;
                    },
                };
                if D {
                    println!("S3: {:#?}", line);
                }
                let line = match s4(line) {
                    Ok(s) => s,
                    Err(s) => {
                        println!("Error while parsing (S4): {}", s);
                        continue;
                    },
                };
                if D {
                    println!("S4: {:#?}", line);
                }
                let f = match make_function(state.clone(), line, Some(s.clone())).await {
                    Ok(f) => f,
                    Err(e) => {
                        let e = to_string_base(state.clone(), e).await.unwrap();
                        println!("Error while compiling into function: {}", e);
                        continue;
                    },
                };
                match call(state.clone(), f, vec![]).await {
                    Ok(_) => {},
                    Err(e) => {
                        let e = match to_string_base(state.clone(), e).await {
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
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
        "run" => todo!("implement running Bodu code"),
        "repl" => todo!("implement repl"),
        "help" => println!("{}", HELP_MSG),
        "version" | "--version" => println!("Bodu 0.1.0"),
        _ => println!("Invalid action. Run `bodu help` for available commands"),
    }
}

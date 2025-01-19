mod io;
mod runtime;
mod utils;

use std::env;
use io::logger;
use crate::runtime::package;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        logger::error("No command provided. Usage: snpm <command> [options]");
        return;
    }

    let debug_mode = args.contains(&"--debug".to_string());
    let force_mode = args.contains(&"--force".to_string());

    match args[1].as_str() {
        "install" => package::install(debug_mode, force_mode),
        "task" => {
            if args.len() < 3 {
                logger::error("No task name provided. Usage: snpm task <task-name>");
                return;
            }
            runtime::task::run(&args[2]);
        }
        _ => logger::error("Unknown command. Supported commands: install, task"),
    }
}


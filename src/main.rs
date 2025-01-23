mod io;
mod runtime;
mod utils;

use std::env;
use crate::runtime::{task, package};
use crate::io::logger;
use crate::utils::info;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        logger::error("No command provided");
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "install" => package::install(false, false),
        "task" => {
            if args.len() < 3 {
                logger::error("No task name provided");
                return;
            }
            task::run(&args[2]);
        },
        "x" | "exec" => {
            if args.len() < 3 {
                logger::error("No package name provided for execution");
                return;
            }
            let package_args = args[2..].join(" ");
            task::run_npx(&package_args);
        },
        "create" => {
            if args.len() < 3 {
                logger::error("No template name provided for create");
                return;
            }
            let create_args = args[2..].join(" ");
            task::run_create(&create_args);
        },
        "help" | "h" | "?" | "version" | "v" => {
            info::version();
        },
        _ => {
            info::version();
            std::process::exit(1);
        }
    }
}


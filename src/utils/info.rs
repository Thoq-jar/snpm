use crate::io::logger::colorize;
use crate::utils::utils::ASCII_ART;

pub fn version() {
    println!("{}", colorize("red", ASCII_ART));
    println!("{} {}", colorize("magenta", "SuperNPM"), colorize("magenta", env!("CARGO_PKG_VERSION")));

    println!("{}", colorize("white", "Usage: snpm <command> [options]"));
    println!("{}", colorize("white", "Commands:"));
    println!("{}", colorize("white", "  install       Install dependencies"));
    println!("{}", colorize("white", "  task          Run a task"));
    println!("{}", colorize("white", "  x | exec      Execute a package"));
    println!("{}", colorize("white", "  create        Create a new project"));
}

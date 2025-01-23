pub fn colorize(color: &str, message: &str) -> String {
    match color {
        "red" => format!("\x1b[31m{}\x1b[0m", message),
        "orange" => format!("\x1b[33m{}\x1b[0m", message),
        "yellow" => format!("\x1b[33m{}\x1b[0m", message),
        "green" => format!("\x1b[32m{}\x1b[0m", message),
        "blue" => format!("\x1b[34m{}\x1b[0m", message),
        "cyan" => format!("\x1b[36m{}\x1b[0m", message),
        "magenta" => format!("\x1b[35m{}\x1b[0m", message),
        "light_blue" => format!("\x1b[94m{}\x1b[0m", message),
        "light_green" => format!("\x1b[92m{}\x1b[0m", message),
        "light_cyan" => format!("\x1b[96m{}\x1b[0m", message),
        "light_red" => format!("\x1b[91m{}\x1b[0m", message),
        "light_magenta" => format!("\x1b[95m{}\x1b[0m", message),
        "light_yellow" => format!("\x1b[93m{}\x1b[0m", message),
        "gray" | "grey" => format!("\x1b[90m{}\x1b[0m", message),
        "white" => format!("\x1b[97m{}\x1b[0m", message),
        "pink" => format!("\x1b[38;5;207m{}\x1b[0m", message),
        "nextjs_pink" => format!("\x1b[38;5;183m{}\x1b[0m", message),
        _ => message.to_string(),
    }
}

fn format_message(message: String, log_type: &str) -> String {
    match log_type.to_lowercase().as_str() {
        "info" => format!("{} {} {}", colorize("white", "snpm"), colorize("light_blue", "info"), message),
        "warn" => format!("{} {} {}", colorize("white", "snpm"), colorize("yellow", "warn"), message),
        "error" => format!("{} {} {}", colorize("white", "snpm"), colorize("red", "err"), message),
        _ => format!("{} {}", "Unknown color".to_string(), message),
    }
}

pub fn info(message: &str) {
    println!("{}", format_message(message.to_string(), "info"));
}

pub fn warn(message: &str) {
    println!("{}", format_message(message.to_string(), "warn"));
}

pub fn error(message: &str) {
    println!("{}", format_message(message.to_string(), "error"));
}

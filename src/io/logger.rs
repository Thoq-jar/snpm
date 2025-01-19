fn colorize(color: &str, message: &str) -> String {
    match color {
        "red" => format!("\x1b[31m{}\x1b[0m", message),
        "magenta" => format!("\x1b[35m{}\x1b[0m", message),
        "blue" => format!("\x1b[34m{}\x1b[0m", message),
        "yellow" => format!("\x1b[33m{}\x1b[0m", message),
        "gray" | "grey" => format!("\x1b[90m{}\x1b[0m", message),
        _ => message.to_string(),
    }
}

fn format_message(message: String, log_type: &str) -> String {
    match log_type.to_lowercase().as_str() {
        "info" => format!("{}{}{}{}", colorize("gray", "[ "),
                          colorize("blue", "INFO"),
                          colorize("gray", " ] "),
                          message
        ),
        "warn" => format!("{}{}{}{}", colorize("gray", "[ "),
                          colorize("yellow", "WARN"),
                          colorize("gray", " ] "),
                          message
        ),
        "error" => format!("{}{}{}{}", colorize("gray", "[ "),
                           colorize("red", "ERROR"),
                           colorize("gray", " ] "),
                           message
        ),
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

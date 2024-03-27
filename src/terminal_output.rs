use colored::Colorize;

pub fn print_info(msg: &str) {
    println!("[{}]    {msg}", "INFO".blue())
}

pub fn print_success(msg: &str) {
    println!("[{}] {msg}", "SUCCESS".green())
}
pub fn print_error(msg: &str) {
    println!("[{}]   {msg}", "ERROR".red())
}

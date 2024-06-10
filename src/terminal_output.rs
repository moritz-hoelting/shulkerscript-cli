use std::fmt::Display;

use colored::Colorize;

pub fn print_info<D>(msg: D)
where
    D: Display,
{
    println!("[{}]    {msg}", "INFO".blue())
}

pub fn print_success<D>(msg: D)
where
    D: Display,
{
    println!("[{}] {msg}", "SUCCESS".green())
}

pub fn print_warning<D>(msg: D)
where
    D: Display,
{
    println!("[{}] {msg}", "WARNING".yellow())
}

pub fn print_error<D>(msg: D)
where
    D: Display,
{
    println!("[{}]   {msg}", "ERROR".red())
}

use chrono::Local;
use nanoid::nanoid;

pub fn generate_string() -> String {
    let s = String::from("1234567890qwertyuioplkjhgfdsazxcvbnm");
    let alphabet: Vec<char> = s.chars().collect();
    nanoid!(10, &alphabet)
}

pub fn generate_string_size(total_char: usize) -> String {
    let today = Local::now();
    let formatted_date = today.format("%y%m%d").to_string();
    let s = String::from("1234567890qwertyuioplkjhgfdsazxcvbnm");
    let alphabet: Vec<char> = s.chars().collect();
    let prefix = nanoid!(total_char, &alphabet);
    format!("{}{}", formatted_date, prefix)
}

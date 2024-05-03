use chrono::prelude::*;


pub fn log_this(message: &str) {
    let now = Utc::now();
    println!("[{}]", now.format("%Y-%m-%d %H:%M:%S"));
    println!("{}", message);
    println!("");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        super::log_this("This is a test");
    }
}
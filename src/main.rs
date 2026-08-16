mod accessibility;
mod keyboard;

use std::env;

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let _program = args.next();

    match args.next().as_deref() {
        Some("inspect") => {
            if let Err(error) = accessibility::inspect().await {
                eprintln!("inspect: {error}");
                std::process::exit(1);
            }
        }
        Some("keyboard-test") => {
            if let Err(error) = keyboard::test().await {
                eprintln!("keyboard-test: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage: vimux <inspect|keyboard-test>");
            std::process::exit(2);
        }
    }
}

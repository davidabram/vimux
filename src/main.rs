mod accessibility;

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
        _ => {
            eprintln!("usage: vimux inspect");
            std::process::exit(2);
        }
    }
}

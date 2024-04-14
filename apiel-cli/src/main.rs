#![allow(clippy::unnecessary_wraps)]

use std::io::{self, BufRead, Write};

use apiel::parse;

fn main() {
    tracing_subscriber::fmt().init();

    let stdin = io::stdin();

    println!(
        r#"
     /$$$$$$            /$$           /$$
    /$$__  $$          |__/          | $$
   | $$  \ $$  /$$$$$$  /$$  /$$$$$$ | $$
   | $$$$$$$$ /$$__  $$| $$ /$$__  $$| $$
   | $$__  $$| $$  \ $$| $$| $$$$$$$$| $$
   | $$  | $$| $$  | $$| $$| $$_____/| $$
   | $$  | $$| $$$$$$$/| $$|  $$$$$$$| $$
   |__/  |__/| $$____/ |__/ \_______/|__/
             | $$                        
             | $$                        
             |__/                        
"#
    );

    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(line)) => {
                if line.trim().is_empty() {
                    continue;
                }
                match parse::parse_and_evaluate(&line) {
                    Ok(result) => println!("Result: {:?}", result),
                    Err(err) => tracing::error!("{}", err),
                }
            }
            _ => break,
        }
    }
}

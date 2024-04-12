#![allow(clippy::unnecessary_wraps)]

use std::io::{self, BufRead, Write};

use apiel_mvp::parse;

fn main() {
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
    #[cfg(feature = "debug")]
    {
        println!("Debug mode is on.");
    }

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
                    Err(err) => eprintln!("{}", err),
                }
            }
            _ => break,
        }
    }
}

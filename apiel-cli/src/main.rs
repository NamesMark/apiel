use std::io::{self, BufRead, Write};

use apiel::Env;
use apiel::parse::{eval_to_val, format_val};

fn main() {
    tracing_subscriber::fmt().init();

    let stdin = io::stdin();
    let mut env = Env::new();

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
                match eval_to_val(&line, &mut env) {
                    Ok(val) => println!("{}", format_val(&val)),
                    Err(err) => eprintln!("ERROR: {err}"),
                }
            }
            _ => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use apiel::Env;
    use apiel::apl;
    use apiel::parse::{eval_to_val, format_val};
    use std::process::Command;

    #[test]
    fn macro_works() {
        assert_eq!(apl!("1 + 1").unwrap(), vec![2.0]);
        assert_eq!(apl!("⍳ 3").unwrap(), vec![1.0, 2.0, 3.0]);
        assert_eq!(apl!("+/ ⍳ 10").unwrap(), vec![55.0]);
    }

    #[test]
    fn format_output() {
        let mut env = Env::new();
        let val = eval_to_val("1 2 3", &mut env).unwrap();
        assert_eq!(format_val(&val), "1 2 3");

        let val = eval_to_val("'hello'", &mut env).unwrap();
        assert_eq!(format_val(&val), "hello");
    }

    #[test]
    fn cli_works() {
        let bin = assert_cmd::cargo::cargo_bin("apiel-cli");
        let output = Command::new(bin)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(b"1 2 3 + 4 5 6\n")?;
                child.wait_with_output()
            })
            .expect("failed to run apiel-cli");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("5 7 9"),
            "expected '5 7 9' in output, got: {stdout}"
        );
    }
}

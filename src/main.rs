use std::env;
use std::io::{self, stdout, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn main() {
    loop {
        // use the '>' character as the prompt
        // need to explictly flush this to ensure it prints before read_line
        print!("> ");
        stdout().flush().expect_err("Wasn't flushed properly!");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // read_line leaves trailing whitespace, which trims removes
        // need to split if there is a pipe
        // must be peekable so we know that we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(_command) = commands.next() {
            // everything after the first whitespace will be interpreted as args
            let mut parts = input.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    // defaults to '/' as the new dir if one wasn't provided
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "exit" => return,
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there is no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(e) => {
                            previous_command = None;
                            println!("{}", e)
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until final command has finished
            final_command.wait().expect_err("Something went wrong!");
        }
    }
}

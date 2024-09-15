use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{thread};

pub enum EventsFromCommand {
    OutputLine(String),
    ErrorLine(String),
    OtherError(String),
    ExitStatus(ExitStatus),
}

fn start_command(cmd: String) -> Receiver<EventsFromCommand> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let is_windows = cfg!(target_os = "windows");
        let mut command_without_args = if is_windows { Command::new("cmd") } else { Command::new("sh") }; // loosely based on the example from docs: https://doc.rust-lang.org/std/process/struct.Command.html
        let command_with_args = if is_windows { command_without_args.args(["/C", &cmd]) } else { command_without_args.args(["-c", &cmd]) };
        let child_wrapped =
            command_with_args
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        if let Err(err) = child_wrapped {
            let command_result_line = EventsFromCommand::OtherError(err.to_string());
            tx.send(command_result_line).unwrap();
            return;
        }
        let mut child = child_wrapped.unwrap();
        let stdout = child
            .stdout
            .take()
            .expect("Stout should never panic after the line .stdout(Stdio::piped())");
        let stderr = child
            .stderr
            .take()
            .expect("Stout should never panic after the line .stderr(Stdio::piped())");

        // Below creates another thread: reading error stream and sending it to the common channel.
        let tx1 = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                if let Ok(line_str) = line {
                    let command_result_line = EventsFromCommand::ErrorLine(line_str);
                    tx1.send(command_result_line).unwrap();
                }
            }
        });

        for line in BufReader::new(stdout).lines() {
            if let Ok(line_str) = line {
                let command_result_line = EventsFromCommand::OutputLine(line_str);
                tx.send(command_result_line).unwrap();
            }
        }

        if let Ok(exit_status) = child.wait() {
            tx.send(EventsFromCommand::ExitStatus(exit_status)).unwrap();
        }
    });
    rx
}

fn main() {
    let rx = start_command("dir".to_string());

    for result_line in rx {
        if let EventsFromCommand::OutputLine(line) = result_line {
            println!("{line}");
        } else if let EventsFromCommand::ErrorLine(line) = result_line {
            println!("{line}");
        } else if let EventsFromCommand::OtherError(line) = result_line {
            println!("{line}");
        } else if let EventsFromCommand::ExitStatus(exitStatus) = result_line {
            println!("Process exited with exit status: {exitStatus}");
        }
    }

    println!("Main app finished.");
}

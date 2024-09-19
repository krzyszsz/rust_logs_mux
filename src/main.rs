use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread};

struct CommandContext {
    cmd: String,
    tx: Sender<ExtendedEvent>,
    source_id: String
}

pub enum EventsFromCommand {
    OutputLine(String),
    ErrorLine(String),
    OtherError(String),
    ExitStatus(ExitStatus),
}

struct ExtendedEvent {
    payload: EventsFromCommand,
    source_id: String
}

fn enrich_event_with_context_data(payload: EventsFromCommand, context: &CommandContext) -> ExtendedEvent {
    let source_id = context.source_id.clone();
    ExtendedEvent { payload, source_id }
}

fn start_command_parametrized(command_context: CommandContext) {
    thread::spawn(move || {
        let is_windows = cfg!(target_os = "windows");
        let mut command_without_args = if is_windows { Command::new("cmd") } else { Command::new("sh") }; // loosely based on the example from docs: https://doc.rust-lang.org/std/process/struct.Command.html
        let command_with_args = if is_windows { command_without_args.args(["/C", &command_context.cmd]) } else { command_without_args.args(["-c", &command_context.cmd]) };
        let child_wrapped =
            command_with_args
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        if let Err(err) = child_wrapped {
            let command_result_line = EventsFromCommand::OtherError(err.to_string());
            command_context.tx.send(enrich_event_with_context_data(command_result_line, &command_context)).unwrap();
            return;
        }
        let mut child = child_wrapped.unwrap();
        let stdout = child
            .stdout
            .take()
            .expect("Stdout should never panic after the line .stdout(Stdio::piped())");
        let stderr = child
            .stderr
            .take()
            .expect("Stderr should never panic after the line .stderr(Stdio::piped())");

        // Below creates another thread: reading error stream and sending it to the common channel.
        let context_copy = CommandContext {
            tx: command_context.tx.clone(),
            cmd: command_context.cmd.clone(),
            source_id: command_context.source_id.clone()
        };
        let tx1 = command_context.tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                if let Ok(line_str) = line {
                    let command_result_line = enrich_event_with_context_data(EventsFromCommand::ErrorLine(line_str), &context_copy);
                    tx1.send(command_result_line).unwrap();
                }
            }
        });

        for line in BufReader::new(stdout).lines() {
            if let Ok(line_str) = line {
                let command_result_line = enrich_event_with_context_data(EventsFromCommand::OutputLine(line_str), &command_context);
                command_context.tx.send(command_result_line).unwrap();
            }
        }

        if let Ok(exit_status) = child.wait() {
            command_context.tx.send(enrich_event_with_context_data(EventsFromCommand::ExitStatus(exit_status), &command_context)).unwrap();
        }
    });
}

fn start_command(cmd: String) -> Receiver<ExtendedEvent> {
    let (tx, rx) = mpsc::channel();
    let ctx = CommandContext {
        cmd,
        tx,
        source_id: String::from("default-id")
    };
    start_command_parametrized(ctx);
    rx
}

fn start_many_commands() -> Receiver<ExtendedEvent> {
    let (tx, rx) = mpsc::channel();
    let ctx1 = CommandContext {
        cmd: String::from("ping -t www.google.com"),
        tx: tx.clone(),
        source_id: String::from("one")
    };
    start_command_parametrized(ctx1);

    let ctx2 = CommandContext {
        cmd: String::from("ping -t www.bbc.co.uk"),
        tx: tx.clone(),
        source_id: String::from("two")
    };
    start_command_parametrized(ctx2);

    rx
}

fn display_command_result(rx: Receiver<ExtendedEvent>) {
    for event_from_command in rx {
        let event_payload = event_from_command.payload;
        let source_id = event_from_command.source_id;
        match event_payload {
            EventsFromCommand::OutputLine(line) => println!("{source_id}: {line}"),
            EventsFromCommand::ErrorLine(line) => println!("{source_id}: {line}"),
            EventsFromCommand::OtherError(line) => println!("{source_id}: {line}"),
            EventsFromCommand::ExitStatus(exit_status) => println!("{source_id}: Process exited with exit status: {exit_status}")
        }
    }
}

fn main() {
    let mut rx = start_command(String::from("dir"));
    display_command_result(rx);

    rx = start_many_commands();
    display_command_result(rx);

    println!("Main app finished.");
}

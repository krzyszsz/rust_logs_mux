Not much to see here. I'm learning Rust with a toy project that changes the default process spawning, that is normally blocking, to be event driven. Most likely useless for anyone on higher level of Rust. :-)
Next step will be to build a clone of linux Mux that suits my needs better (with no ambitions to handle anything more than log files tail, so it's not going to replace standard linux mux any time soon!).

```Rust
    let rx = start_command("dir".to_string());

    for result_line in rx {
        match result_line {
            EventsFromCommand::OutputLine(line) => println!("{line}"),
            EventsFromCommand::ErrorLine(line) => println!("{line}"),
            EventsFromCommand::OtherError(line) => println!("{line}"),
            EventsFromCommand::ExitStatus(exitStatus) => println!("Process exited with exit status: {exitStatus}")
        }
    }
```

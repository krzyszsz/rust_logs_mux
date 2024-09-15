Not much to see here. I'm learning Rust with a toy project that changes the default process spawning, that is normally blocking, to be event driven. Most likely useless for anyone on higher level of Rust. :-)
Next step with be to build a clone of linux Mux that suits my needs better.

```Rust
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
```

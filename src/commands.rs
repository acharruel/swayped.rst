use swayipc::{Connection, Fallible};

pub enum SwaypedCommand {
    WorkspacePrev,
    WorkspaceNext,
    WorkspaceBackAndForth,
}

impl SwaypedCommand {
    pub fn process_command(self) {
        let cmd: String = match self {
            SwaypedCommand::WorkspacePrev => String::from("workspace prev"),
            SwaypedCommand::WorkspaceNext => String::from("workspace next"),
            SwaypedCommand::WorkspaceBackAndForth => String::from("workspace back_and_forth"),
        };
        sway_send_command(cmd);
    }
}

fn sway_send_command(cmd: String) -> Fallible<()> {
    let mut connection = Connection::new()?;

    for outcome in connection.run_command(cmd)? {
        if let Err(error) = outcome {
            println!("failure '{}'", error);
        } else {
            println!("success");
        }
    }
    Ok(())
}

use std::cmp;
use swayipc::{Connection, Fallible};

pub enum SwaypedCommand {
    WorkspacePrev,
    WorkspaceNext,
    WorkspaceBackAndForth,
    WorkspaceNew,
}

#[allow(unused_must_use)]
impl SwaypedCommand {
    pub fn process_command(self) {
        match self {
            SwaypedCommand::WorkspacePrev => {
                sway_send_command(String::from("workspace prev"));
            }
            SwaypedCommand::WorkspaceNext => {
                sway_send_command(String::from("workspace next"));
            }
            SwaypedCommand::WorkspaceBackAndForth => {
                sway_send_command(String::from("workspace back_and_forth"));
            }
            SwaypedCommand::WorkspaceNew => {
                sway_new_workspaces();
            }
        };
    }
}

fn sway_send_command(cmd: String) -> Fallible<()> {
    let mut connection = Connection::new()?;

    for res in connection.run_command(cmd)? {
        if let Err(error) = res {
            println!("Failed to run command: '{}'", error);
        }
    }

    Ok(())
}

#[allow(unused_must_use)]
fn sway_new_workspaces() -> Fallible<()> {
    let mut connection = Connection::new()?;
    let mut max = 0;

    for w in connection.get_workspaces()? {
        max = cmp::max(max, w.num);
    }

    sway_send_command(format!("workspace {}", max + 1));

    Ok(())
}

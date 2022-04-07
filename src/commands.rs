use log::error;

use swayipc::{Connection, Fallible};

#[allow(clippy::enum_variant_names)]
pub enum SwaypedCommand {
    WorkspacePrev,
    WorkspaceNext,
    WorkspaceBackAndForth,
    WorkspaceNew,
}

impl SwaypedCommand {
    pub fn process_command(self) {
        let res: Fallible<()> = match self {
            SwaypedCommand::WorkspacePrev => sway_send_command(String::from("workspace prev")),
            SwaypedCommand::WorkspaceNext => sway_send_command(String::from("workspace next")),
            SwaypedCommand::WorkspaceBackAndForth => {
                sway_send_command(String::from("workspace back_and_forth"))
            }
            SwaypedCommand::WorkspaceNew => sway_new_workspaces(),
        };
        match res {
            Ok(_) => (),
            Err(e) => error!("Failed to send sway command: '{}'", e),
        }
    }
}

fn sway_send_command(cmd: String) -> Fallible<()> {
    let mut connection = Connection::new()?;

    for res in connection.run_command(cmd)? {
        if let Err(error) = res {
            error!("Failed to run command: '{}'", error);
        }
    }

    Ok(())
}

fn sway_new_workspaces() -> Fallible<()> {
    let mut connection = Connection::new()?;
    let mut max = 1;
    let mut workspaces: Vec<i32> = Vec::new();

    for w in connection.get_workspaces()? {
        workspaces.push(w.num);
    }

    workspaces.sort_unstable();

    for w in workspaces {
        if w == max {
            max += 1;
        } else {
            break;
        }
    }

    sway_send_command(format!("workspace {}", max))?;

    Ok(())
}

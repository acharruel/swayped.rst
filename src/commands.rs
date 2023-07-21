use anyhow::{Result, Context, bail};

use swayipc::Connection;

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum SwaypedCommand {
    WorkspacePrev,
    WorkspaceNext,
    WorkspaceBackAndForth,
    WorkspaceNew,
}

impl SwaypedCommand {
    pub fn process_command(self) -> Result<()> {
        use SwaypedCommand::*;

        match self {
            WorkspacePrev => sway_send_command("workspace prev"),
            WorkspaceNext => sway_send_command("workspace next"),
            WorkspaceBackAndForth => sway_send_command("workspace back_and_forth"),
            WorkspaceNew => sway_new_workspaces(),
        }.with_context(|| format!("Failed to send sway command {self:?}"))
    }
}

fn sway_send_command(cmd: impl Into<String>) -> Result<()> {
    let mut connection = Connection::new()?;

    for res in connection.run_command(cmd.into())? {
        if let Err(error) = res {
            bail!("Failed to run command: '{}'", error);
        }
    }

    Ok(())
}

fn sway_new_workspaces() -> Result<()> {
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

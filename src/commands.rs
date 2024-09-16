use anyhow::{bail, Result};

use swayipc::Connection;
use tracing::debug;

#[derive(Debug)]
pub enum InputCommand {
    SwipeUp(i32),
    SwipeDown(i32),
    SwipeLeft(i32),
    SwipeRight(i32),
    ScrollLeft,
    ScrollRight,
}

impl InputCommand {
    pub fn process_command(self) -> Result<()> {

        match self {
            InputCommand::SwipeUp(3) => builtin::sway_new_workspaces()?,
            InputCommand::SwipeDown(3) => sway_send_command("workspace back_and_forth")?,
            InputCommand::SwipeLeft(3) => sway_send_command("workspace prev")?,
            InputCommand::SwipeRight(3) => sway_send_command("workspace next")?,
            InputCommand::ScrollLeft => sway_send_command("workspace prev")?,
            InputCommand::ScrollRight => sway_send_command("workspace next")?,
            _ => (),
        }
        Ok(())
    }
}

fn sway_send_command(cmd: &str) -> Result<()> {
    let mut connection = Connection::new()?;

    debug!(?cmd, "Sending command to sway");

    for res in connection.run_command(cmd)? {
        if let Err(error) = res {
            bail!("Failed to run command: '{}'", error);
        }
    }

    Ok(())
}

mod builtin {
    use super::*;

pub fn sway_new_workspaces() -> Result<()> {
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

    sway_send_command(&format!("workspace {}", max))?;

    Ok(())
}
}

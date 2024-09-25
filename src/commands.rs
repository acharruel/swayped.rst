use std::collections::HashMap;

use anyhow::{bail, Result};

use swayipc::Connection;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::config::TomlConfig;

#[derive(Debug)]
pub struct CommandDesc {
    dry_run: bool,
    tx: mpsc::Sender<InputCommand>,
    mappings: HashMap<InputCommand, OutputCommand>,
}

impl CommandDesc {
    pub fn new(dry_run: bool, config: TomlConfig, tx: mpsc::Sender<InputCommand>) -> Self {
        let mut mappings = HashMap::new();

        config
            .mappings
            .iter()
            .for_each(|x| match (x.gesture.as_str(), x.finger_count) {
                ("swipe_left", Some(n)) => {
                    mappings.insert(
                        InputCommand::SwipeLeft(n),
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                ("swipe_right", Some(n)) => {
                    mappings.insert(
                        InputCommand::SwipeRight(n),
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                ("swipe_up", Some(n)) => {
                    mappings.insert(
                        InputCommand::SwipeUp(n),
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                ("swipe_down", Some(n)) => {
                    mappings.insert(
                        InputCommand::SwipeDown(n),
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                ("scrollwheel_left", None) => {
                    mappings.insert(
                        InputCommand::ScrollLeft,
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                ("scrollwheel_right", None) => {
                    mappings.insert(
                        InputCommand::ScrollRight,
                        OutputCommand {
                            cmd: x.cmd.clone(),
                            cmd_type: x.cmd_type.clone(),
                        },
                    );
                }
                _ => warn!("Unsupported mapping: {:?}", x),
            });

        Self {
            dry_run,
            tx,
            mappings,
        }
    }

    pub async fn send(&self, cmd: InputCommand) -> Result<()> {
        self.tx.send(cmd).await?;
        Ok(())
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum InputCommand {
    SwipeUp(i32),
    SwipeDown(i32),
    SwipeLeft(i32),
    SwipeRight(i32),
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug)]
struct OutputCommand {
    cmd: String,
    cmd_type: String,
}

impl InputCommand {
    pub fn process_command(self, cmd_desc: &CommandDesc) -> Result<()> {
        let cmd = &cmd_desc.mappings.get(&self);

        let Some(cmd) = cmd else {
            bail!("Command not in configuration: {:?}", self);
        };

        if cmd_desc.dry_run {
            info!(?cmd, "Dry run, command: ");
            return Ok(());
        }

        match cmd.cmd_type.as_str() {
            "sway" => sway::process_command(&cmd.cmd)?,
            "builtin" => builtin::process_command(&cmd.cmd)?,
            cmd_type => warn!(?cmd_type, "Command type not supported"),
        }

        Ok(())
    }
}

mod sway {
    use super::*;

    pub fn process_command(cmd: &str) -> Result<()> {
        let mut connection = Connection::new()?;

        debug!(?cmd, "Sending command to sway");

        for res in connection.run_command(cmd)? {
            if let Err(error) = res {
                bail!("Failed to run command: '{}'", error);
            }
        }

        Ok(())
    }
}

mod builtin {
    use super::*;

    pub fn process_command(cmd: &str) -> Result<()> {
        match cmd {
            "workspace_new" => sway_new_workspaces()?,
            _ => warn!("Builtin command not supported"),
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

        sway::process_command(&format!("workspace {}", max))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Mapping;

    #[test]
    fn test_command_desc_new() {
        let (tx, _) = mpsc::channel(1);
        let config = TomlConfig { mappings: vec![] };

        let cmd_desc = CommandDesc::new(false, config, tx);

        assert_eq!(cmd_desc.dry_run, false);
        assert_eq!(cmd_desc.mappings.len(), 0);
    }

    #[tokio::test]
    async fn test_command_desc_recv() {
        let (tx, mut rx) = mpsc::channel(1);
        let config = TomlConfig { mappings: vec![] };

        let cmd_desc = CommandDesc::new(true, config, tx);

        let cmd = InputCommand::SwipeUp(3);
        let res = cmd_desc.send(cmd).await;
        assert_eq!(res.is_ok(), true);

        let cmd = rx.recv().await.unwrap();
        assert_eq!(cmd, InputCommand::SwipeUp(3));
    }

    #[tokio::test]
    async fn test_command_desc_process_fail() {
        let (tx, _) = mpsc::channel(1);
        let config = TomlConfig {
            mappings: vec![Mapping {
                gesture: "swipe_up".to_string(),
                finger_count: Some(3),
                cmd: "workspace_new".to_string(),
                cmd_type: "sway".to_string(),
            }],
        };

        let cmd_desc = CommandDesc::new(false, config, tx);

        let cmd = InputCommand::SwipeDown(3);
        let res = cmd.process_command(&cmd_desc);
        assert_eq!(res.is_err(), true);
    }
}

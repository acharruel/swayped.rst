mod commands;
mod config;
mod gesture;
mod pointer;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use gesture::SwaypedGesture;
use input::event::Event::Gesture;
use input::event::Event::Pointer;
use input::event::GestureEvent::{Hold, Swipe};
use input::event::PointerEvent::ScrollWheel;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDWR, O_WRONLY};
use tracing::warn;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{FromRawFd, IntoRawFd, RawFd},
};
use std::path::Path;
use std::path::PathBuf;
use tokio::io::unix::AsyncFd;
use tokio::io::Ready;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tracing::error;
use tracing::info;
use tracing::trace;

use crate::commands::{CommandDesc, InputCommand};
use crate::config::TomlConfig;
use crate::pointer::pointer_handle_scroll_event;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read(flags & O_RDWR != 0)
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

pub struct AsyncLibinput(AsyncFd<Libinput>);

impl AsyncLibinput {
    pub async fn read(&mut self, events: &mut Vec<Event>) -> io::Result<usize> {
        let mut guard = self.0.readable_mut().await?;
        match guard.try_io(|inner| {
            inner.get_mut().dispatch()?;
            while let Some(event) = inner.get_mut().next() {
                events.push(event);
            }
            Ok(())
        }) {
            Ok(_) => {
                guard.clear_ready_matching(Ready::READABLE);
                Ok(events.len())
            }
            Err(_would_block) => Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Libinput IO would block",
            )),
        }
    }
}

async fn process_event(
    event: &Event,
    gesture: &mut Box<SwaypedGesture<'_>>,
    cmd_desc: &CommandDesc,
) {
    trace!(?event, "Processing event:");
    let res = match event {
        Gesture(Hold(_)) => gesture.reset(),
        Gesture(Swipe(event)) => gesture.handle_event(event).await,
        Pointer(ScrollWheel(event)) => pointer_handle_scroll_event(event, cmd_desc).await,
        _ => Ok(()),
    };

    if let Err(err) = res {
        error!(?err, "Error");
    }
}

pub async fn run(dry_run: bool, config_file: Option<String>) -> Result<()> {
    let mut sigterm = signal(SignalKind::terminate()).context("Failed to create SIGTERM signal")?;

    let mut sigint = signal(SignalKind::interrupt()).context("Failed to create SIGINT signal")?;

    let mut input = Libinput::new_with_udev(Interface);
    let Ok(_) = input.udev_assign_seat("seat0") else {
        bail!("Failed to assign seat");
    };

    let mut input = AsyncLibinput(AsyncFd::new(input).context("Failed to create async libinput")?);

    let config_file = match config_file {
        Some(file) => PathBuf::from(file),
        None => TomlConfig::config_dir().join("config.toml"),
    };

    info!(?config_file, "Starting swayped");

    let config = TomlConfig::new(config_file)?;

    let (tx, mut rx) = mpsc::channel::<InputCommand>(8);
    let command_desc = CommandDesc::new(dry_run, config, tx);

    let mut gesture = Box::new(SwaypedGesture::new(&command_desc));
    let mut events = Vec::new();

    loop {
        events.clear();
        select! {
            Ok(_) = input.read(&mut events) => {
                for event in &events {
                    process_event(event, &mut gesture, &command_desc).await;
                }
            },

            Some(cmd) = rx.recv() => {
                cmd.process_command(&command_desc).unwrap_or_else(|err| {
                    warn!(?err, "Failed to process command");
                });
            },

            _ = sigterm.recv() => {
                print!("\r");
                warn!("Received SIGTERM signal");
                break;
            },

            _ = sigint.recv() => {
                print!("\r");
                warn!("Received SIGINT signal");
                break;
            },
        }
    }

    info!("Terminating program");
    Ok(())
}

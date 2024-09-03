mod commands;
mod gesture;
mod pointer;
mod swipe;

use anyhow::Result;
use input::event::Event::Gesture;
use input::event::Event::Pointer;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDWR, O_WRONLY};
use tracing::error;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{FromRawFd, IntoRawFd, RawFd},
};
use std::path::Path;
use tokio::io::unix::AsyncFd;
use tokio::io::Ready;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};

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

pub struct AsyncLibinput {
    inner: AsyncFd<Libinput>,
}

impl AsyncLibinput {
    pub fn new(input: Libinput) -> Self {
        Self {
            inner: AsyncFd::new(input).unwrap_or_else(|err| {
                panic!("Failed to create async libinput: {}", err);
            }),
        }
    }

    pub async fn read(&mut self, events: &mut Vec<Event>) -> io::Result<usize> {
        let mut guard = self.inner.readable_mut().await?;
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

use gesture::*;
use pointer::*;
fn process_event(event: &Event, gesture: &mut Option<SwaypedGesture>) {
    let res = match event {
        Gesture(gesture_event) => gesture_handle_event(gesture_event, gesture),
        Pointer(pointer_event) => pointer_handle_event(pointer_event),
        _ => Ok(()),
    };

    if let Err(err) = res {
        error!(?err, "Error");
    }
}

pub async fn run() -> io::Result<()> {
    let mut sigterm = signal(SignalKind::terminate()).unwrap_or_else(|err| {
        panic!("Failed to create SIGTERM signal: {}", err);
    });

    let mut sigint = signal(SignalKind::interrupt()).unwrap_or_else(|err| {
        panic!("Failed to create SIGINT signal: {}", err);
    });

    let mut input = Libinput::new_with_udev(Interface);
    let Ok(_) = input.udev_assign_seat("seat0") else {
        panic!("Failed to assign seat");
    };

    let mut input = AsyncLibinput {
        inner: AsyncFd::new(input).unwrap_or_else(|err| {
            panic!("Failed to create async libinput: {}", err);
        }),
    };

    let mut gesture: Option<SwaypedGesture> = None;
    let mut events = Vec::new();
    loop {
        events.clear();
        select! {
            Ok(_) = input.read(&mut events) => {
                for event in &events {
                    process_event(event, &mut gesture);
                }
            },

            _ = sigterm.recv() => {
                println!("\r");
                break;
            },

            _ = sigint.recv() => {
                println!("\r");
                break;
            },
        }
    }

    println!("Terminating program");
    Ok(())
}

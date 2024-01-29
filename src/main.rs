mod commands;
mod gesture;
mod pointer;
mod swipe;

use std::fs::{File, OpenOptions};
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{FromRawFd, IntoRawFd, RawFd},
};
use std::path::Path;

use input::event::Event::Gesture;
use input::event::Event::Pointer;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDWR, O_WRONLY};

use anyhow::{bail, Result};
use log::LevelFilter;
use std::io;
use syslog::{BasicLogger, Facility, Formatter3164};
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

fn syslog_config() -> Result<()> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "swayped".into(),
        pid: 0,
    };

    let Ok(logger) = syslog::unix(formatter) else {
        bail!("Failed to connect to syslog");
    };

    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Debug))?;

    Ok(())
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

    pub async fn read(&mut self) -> io::Result<Event> {
        let mut guard = self.inner.readable_mut().await?;
        match guard.try_io(|inner| {
            inner.get_mut().dispatch()?;
            match inner.get_mut().next() {
                Some(event) => Ok(event),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "No event available",
                )),
            }
        }) {
            Ok(result) => {
                guard.clear_ready_matching(Ready::READABLE);
                result
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
fn process_event(event: Event, gesture: &mut Option<SwaypedGesture>) {

    let res = match event {
        Gesture(gesture_event) => gesture_handle_event(gesture_event, gesture),
        Pointer(pointer_event) => pointer_handle_event(pointer_event),
        _ => Ok(()),
    };

    if let Err(err) = res {
        log::error!("Error: {}", err);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    syslog_config().unwrap_or_else(|err| {
        panic!("Failed to initialize syslog: {}", err);
    });

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
    loop {
        select! {
            event = input.read() => {
                match event {
                    Ok(event) => process_event(event, &mut gesture),
                    Err(_) => continue,
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
}

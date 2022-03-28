mod gesture;

use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::prelude::AsRawFd;
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{FromRawFd, IntoRawFd, RawFd},
};
use std::path::Path;

use input::event::Event::Gesture;
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use nix::poll::{poll, PollFd, PollFlags};

use gesture::*;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
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

fn main() -> io::Result<()> {
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    let pollfd = PollFd::new(input.as_raw_fd(), PollFlags::POLLIN);

    let mut gesture: Option<SwaypedGesture> = None;

    while poll(&mut [pollfd], -1).is_ok() {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                Gesture(gesture_event) => {
                    gesture_handle_event(gesture_event, &mut gesture);
                }
                _ => (),
            }
        }
    }

    Ok(())
}

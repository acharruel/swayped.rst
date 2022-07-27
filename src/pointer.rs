use input::event::pointer::Axis::*;
use input::event::{pointer::PointerScrollWheelEvent, PointerEvent};
use log::debug;

use crate::commands::SwaypedCommand;

enum SwaypedScrollDir {
    Left,
    Right,
}

impl SwaypedScrollDir {
    fn process_command(self) {
        let cmd: SwaypedCommand = match self {
            SwaypedScrollDir::Left => SwaypedCommand::WorkspacePrev,
            SwaypedScrollDir::Right => SwaypedCommand::WorkspaceNext,
        };
        cmd.process_command()
    }

    fn display(&self) -> String {
        match self {
            SwaypedScrollDir::Left => "LEFT".to_string(),
            SwaypedScrollDir::Right => "RIGHT".to_string(),
        }
    }
}

fn pointer_handle_scroll_event(event: PointerScrollWheelEvent) {
    let horiz = event.scroll_value_v120(Horizontal);
    let scroll: Option<SwaypedScrollDir> = if horiz > 0.0 {
        Some(SwaypedScrollDir::Right)
    } else if horiz < 0.0 {
        Some(SwaypedScrollDir::Left)
    } else {
        None
    };

    if let Some(sc) = scroll {
        debug!("scroll_detected! {:?}", sc.display());
        sc.process_command();
    }
}

pub fn pointer_handle_event(event: PointerEvent) {
    // only interested in scroll wheel event
    match event {
        PointerEvent::ScrollWheel(e) => pointer_handle_scroll_event(e),
        _ => (),
    }
}

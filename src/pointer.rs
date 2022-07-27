use input::event::pointer::Axis::*;
use input::event::pointer::ButtonState::*;
use input::event::pointer::PointerButtonEvent;
use input::event::{pointer::PointerScrollWheelEvent, PointerEvent};
use log::debug;

use crate::commands::SwaypedCommand;

const MOUSE5: u32 = 275;
const MOUSE6: u32 = 276;

enum SwaypedPointerAction {
    ScrollLeft,
    ScrollRight,
    Button5,
    Button6,
}

impl SwaypedPointerAction {
    fn process_command(self) {
        let cmd: SwaypedCommand = match self {
            SwaypedPointerAction::ScrollLeft => SwaypedCommand::WorkspacePrev,
            SwaypedPointerAction::ScrollRight => SwaypedCommand::WorkspaceNext,
            SwaypedPointerAction::Button5 => SwaypedCommand::WorkspaceBackAndForth,
            SwaypedPointerAction::Button6 => SwaypedCommand::WorkspaceNew,
        };
        cmd.process_command()
    }

    fn display(&self) -> String {
        match self {
            SwaypedPointerAction::ScrollLeft => "SCROLL LEFT".to_string(),
            SwaypedPointerAction::ScrollRight => "SCROLL RIGHT".to_string(),
            SwaypedPointerAction::Button5 => "MOUSE5".to_string(),
            SwaypedPointerAction::Button6 => "MOUSE6".to_string(),
        }
    }
}

fn pointer_handle_scroll_event(event: PointerScrollWheelEvent) {
    let horiz = event.scroll_value_v120(Horizontal);
    let scroll: Option<SwaypedPointerAction> = if horiz > 0.0 {
        Some(SwaypedPointerAction::ScrollRight)
    } else if horiz < 0.0 {
        Some(SwaypedPointerAction::ScrollLeft)
    } else {
        None
    };

    if let Some(sc) = scroll {
        debug!("scroll_detected! {:?}", sc.display());
        sc.process_command();
    }
}

fn pointer_handle_button(event: PointerButtonEvent) {
    if event.button_state() != Released {
        return;
    }

    let button: Option<SwaypedPointerAction> = if event.button() == MOUSE5 {
        Some(SwaypedPointerAction::Button5)
    } else if event.button() == MOUSE6 {
        Some(SwaypedPointerAction::Button6)
    } else {
        None
    };

    if let Some(b) = button {
        debug!("button press! {:?}", b.display());
        b.process_command();
    }
}

pub fn pointer_handle_event(event: PointerEvent) {
    // only interested in scroll wheel event
    match event {
        PointerEvent::ScrollWheel(e) => pointer_handle_scroll_event(e),
        PointerEvent::Button(e) => pointer_handle_button(e),
        _ => (),
    }
}

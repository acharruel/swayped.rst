use anyhow::Result;
use input::event::pointer::Axis::*;
use input::event::pointer::ButtonState::*;
use input::event::pointer::PointerButtonEvent;
use input::event::{pointer::PointerScrollWheelEvent, PointerEvent};
use log::debug;

use crate::commands::SwaypedCommand;

const MOUSE5: u32 = 275;
const MOUSE6: u32 = 276;

#[derive(Debug)]
enum SwaypedPointerAction {
    ScrollLeft,
    ScrollRight,
    Button5,
    Button6,
}

impl SwaypedPointerAction {
    fn process_command(self) -> Result<()> {
        use SwaypedCommand::*;
        use SwaypedPointerAction::*;

        let cmd: SwaypedCommand = match self {
            ScrollLeft => WorkspacePrev,
            ScrollRight => WorkspaceNext,
            Button5 => WorkspaceBackAndForth,
            Button6 => WorkspaceNew,
        };

        cmd.process_command()
    }

    fn display(&self) -> String {
        use SwaypedPointerAction::*;

        match self {
            ScrollLeft => "SCROLL LEFT".to_string(),
            ScrollRight => "SCROLL RIGHT".to_string(),
            Button5 => "MOUSE5".to_string(),
            Button6 => "MOUSE6".to_string(),
        }
    }
}

fn pointer_handle_scroll_event(event: &PointerScrollWheelEvent) -> Result<()> {
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
        sc.process_command()?;
    }

    Ok(())
}

fn pointer_handle_button(event: &PointerButtonEvent) -> Result<()> {
    if event.button_state() != Released {
        return Ok(());
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
        b.process_command()?;
    }

    Ok(())
}

pub fn pointer_handle_event(event: &PointerEvent) -> Result<()> {
    use PointerEvent::*;

    // only interested in scroll wheel event
    match event {
        ScrollWheel(e) => pointer_handle_scroll_event(e),
        Button(e) => pointer_handle_button(e),
        _ => Ok(()),
    }
}

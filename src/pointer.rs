use anyhow::Result;
use input::event::pointer::Axis::*;
use input::event::pointer::PointerScrollWheelEvent;
use tracing::debug;

use crate::commands::CommandDesc;
use crate::commands::InputCommand;

pub async fn pointer_handle_scroll_event(
    event: &PointerScrollWheelEvent,
    cmd_desc: &CommandDesc,
) -> Result<()> {
    let horiz = event.scroll_value_v120(Horizontal);

    if horiz > 0.0 {
        debug!("scroll right");
        cmd_desc.send(InputCommand::ScrollRight).await?;
    } else if horiz < 0.0 {
        debug!("scroll left");
        cmd_desc.send(InputCommand::ScrollLeft).await?;
    };

    Ok(())
}

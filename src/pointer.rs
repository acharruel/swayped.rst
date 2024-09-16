use anyhow::Result;
use input::event::pointer::Axis::*;
use input::event::pointer::PointerScrollWheelEvent;
use tracing::debug;
use tokio::sync::mpsc;

use crate::commands::InputCommand;

pub async fn pointer_handle_scroll_event(event: &PointerScrollWheelEvent, tx: &mpsc::Sender<InputCommand>) -> Result<()> {
    let horiz = event.scroll_value_v120(Horizontal);
    let cmd = if horiz > 0.0 {
        debug!("scroll right");
        InputCommand::ScrollRight
    } else {
        debug!("scroll left");
        InputCommand::ScrollLeft
    };

    tx.send(cmd).await?;

    Ok(())
}

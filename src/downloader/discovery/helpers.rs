use indicatif::ProgressBar;

use crate::{
    api::download_manager::ServerEvent, downloader::Context, ui::create_standalone_spinner,
};

pub fn init_progress_spinner(ctx: &Context, msg: &str) -> ProgressBar {
    let pb = create_standalone_spinner(msg);

    if let Some(manager) = &ctx.dm {
        manager.broadcast_event(ServerEvent::Message {
            message: msg.to_string(),
            level: "info".to_string(),
        });
    }

    pb
}

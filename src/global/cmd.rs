use std::{
    io::Cursor,
    sync::{Arc, mpsc::Sender},
    thread,
    time::Duration,
};

use image::ImageReader;
use lofty::picture::Picture;
use log::debug;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

use crate::{
    app::App,
    global::{
        logic::{confirmation::Response, image::crop_to_square, session_state::SessionState},
        message::Message,
    },
    player::{
        self,
        logic::{format_conversion::FormatConversion, track::Track},
    },
    playlist::cmd::playlist_save_confirm_then_resume,
    user_input::logic::InputMode,
};

/// The main loop of the program that occurs every frame.
pub async fn tick(app: &mut App) -> Option<Message> {
    if app.session_state == SessionState::RunningFFmpeg {
        return None;
    }

    if let Some(current_track) = app.player.current.as_mut() {
        let status = current_track.conversion_status;

        // Create Protocol if the track has cover art, if not already.
        if !current_track.started_decoding
            && let Some(picture) = current_track.picture.as_mut()
        {
            current_track.started_decoding = true;
            create_protocol(
                picture,
                current_track.id,
                app.picker.clone(),
                &app.senders.msg,
                &app.senders.info,
            );
        }

        // Convert file format to FLAC if current format is not supported, if not already.
        if current_track.conversion_status == FormatConversion::Idle {
            current_track.conversion_status = FormatConversion::Running;
            Track::convert_format(
                &current_track.real_path,
                &current_track.temp_path,
                &app.senders.async_msg,
                &app.senders.info,
            )
            .await;
        }

        // Reload track when track ends if looped
        if app.player.sink.empty()
            && let Some(dur) = current_track.duration
            && dur.saturating_sub(app.player.sink.get_pos()) < Duration::from_secs(3)
        {
            if app.player.looping {
                if let Err(e) = app.player.reload().await {
                    log::error!("Error looping track: {}", e);
                }
            }
        }

        // Load the next track after current track ends.
        if app.player.sink.empty()
            && !app.queue.is_empty()
            && !app.player.looping
            && (status == FormatConversion::Done || status == FormatConversion::Unnecessary)
        {
            debug!("Load the next track after current track ends.");
            player::cmd::play_next_track(app).await;
        } else if app.player.current.is_some()
            && app.player.sink.empty()
            && app.queue.is_empty()
            && (status == FormatConversion::Done || status == FormatConversion::Unnecessary)
        {
            // When the last track in the queue ends
            app.player.pause().await.inspect_err(|e| log::error!("Error pausing track: {}", e)).ok();
        }

        // Load first track if no current track and there is something in the queue.
    } else if app.player.current.is_none() && !app.queue.is_empty() {
        debug!("Load first track (player.current is None)");
        player::cmd::play_next_track(app).await;
    }

    None
}

pub fn ask_for_confirmation(prompt: String, msg: Message, app: &mut App) -> Option<Message> {
    app.user_confirmation.msg = Some(msg);
    app.user_confirmation.prompt = prompt;
    app.input_mode = InputMode::Confirmation;

    None
}

pub fn confirmed(answer: Response, app: &mut App) -> Option<Message> {
    let message = app.user_confirmation.msg.take();
    app.input_mode = InputMode::default();
    app.user_confirmation.prompt.clear();
    app.user_confirmation.response = Some(answer);

    message
}

pub fn update_info_msg(info: String, app: &mut App) -> Option<Message> {
    app.info_msg = info;
    debug!("Updated info message to {}", app.info_msg);
    debug!("Info message length: {}", app.info_msg.len());

    None
}

pub fn quit(app: &mut App) -> Option<Message> {
    if let Some(to_resume) = playlist_save_confirm_then_resume(Message::Quit, &mut app.playlist_ctl)
    {
        return Some(to_resume);
    }

    app.session_state = SessionState::Exit;

    None
}

pub fn cycle_tabs(app: &mut App) -> Option<Message> {
    if let Some(to_resume) =
        playlist_save_confirm_then_resume(Message::CycleTabs, &mut app.playlist_ctl)
    {
        return Some(to_resume);
    }

    app.focused_view_area.cycle_right();

    None
}

pub fn acknowledge_info(app: &mut App) -> Option<Message> {
    app.input_mode = InputMode::default();

    None
}

pub fn display_info_msg(info: String, app: &mut App) -> Option<Message> {
    if let Err(e) = app.senders.info.send(info.clone()) {
        log::error!("Error sending info to display: {e}");
    }

    let cloned_tx = app.senders.info.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        debug!("Clearing info message.");
        if let Err(e) = cloned_tx.send(String::new()) {
            log::error!("Error clearing info message: {e}");
        }
    });

    None
}

pub fn set_track_protocol(protocol: StatefulProtocol, id: u32, app: &mut App) -> Option<Message> {
    if let Some(current_track) = app.player.current.as_mut()
        && id == current_track.id
    {
        log::debug!("Setting protocol...");
        current_track.protocol = Some(protocol);
        if let Err(e) = app.senders.info.send(String::new()) {
            log::error!("{e}");
        };
    }

    None
}

pub fn create_protocol(
    picture: &Picture,
    id: u32,
    picker: Arc<Picker>,
    msg_tx: &Sender<Message>,
    info_tx: &Sender<String>,
) {
    if let Err(e) = info_tx.send("Loading Cover Art...".to_string()) {
        log::error!("{e}");
    };

    log::debug!("Creating protocol...");
    let picture_data = picture.data().to_vec();
    let msg_tx = msg_tx.clone();

    thread::spawn(move || {
        if let Some(dyn_img) = ImageReader::new(Cursor::new(&picture_data))
            .with_guessed_format()
            .ok()
            .and_then(|r| r.decode().ok())
        {
            log::debug!("Created Dynamic Image.");
            let protocol = picker.new_resize_protocol(crop_to_square(dyn_img));
            log::debug!("Cropped to square.");
            if let Err(e) = msg_tx.send(Message::ProtocolCreated(protocol, id)) {
                log::error!("Error sending Protocol back to main thread: {e}");
            }
        }
    });
}

pub fn toggle_show_help(app: &mut App) -> Option<Message> {
    app.show_help = !app.show_help;

    None
}

use std::{cell::RefCell, collections::VecDeque, path::PathBuf, rc::Rc};

use crate::{
    app::App,
    global::{
        logic::{
            confirmation::Response,
            files::{choose_dirs, choose_multiple_audio_files, filter_dir_for_audio_files},
        },
        message::Message,
    },
    playlist::{
        logic::{
            mini_metadata::MiniMetadata, playlist_controller::PlaylistController,
            playlist_tab_focus::PlaylistTabFocus,
        },
        message::PlaylistMessage,
    },
    queue::logic::mini_track::MiniTrack,
    tui::Direction,
    user_input::{logic::InputTarget, message::UserInputMessage},
};

pub fn playlist_save_confirm_then_resume(
    to_resume: Message,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    if let Some(selected_playlist) = playlist_ctl.get_selected_playlist()
        && selected_playlist.is_dirty()
    {
        return Some(Message::Playlist(PlaylistMessage::AskToSave(Box::new(
            Some(to_resume),
        ))));
    }

    None
}

pub fn create_playlist(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if let Some(to_resume) =
        playlist_save_confirm_then_resume(Message::Playlist(PlaylistMessage::Create), playlist_ctl)
    {
        return Some(to_resume);
    }

    let index = playlist_ctl.create_playlist();

    Some(Message::UserInput(UserInputMessage::EnterEditMode(
        "Playlist".to_string(),
        InputTarget::PlaylistName(index),
    )))
}

pub fn name_playlist(index: usize, app: &mut App) -> Option<Message> {
    let current_playlist_names: Vec<String> = app
        .playlist_ctl
        .get_all_playlist_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    if let Some(name) = app.user_input.input_history.pop()
        && let Some(playlist) = app.playlist_ctl.playlist_coll.get_playlist(index)
        && !current_playlist_names.contains(&name)
    {
        if let Err(e) = playlist.rename(&name) {
            log::error!("Error renaming playlist {e}");
        }
        return Some(Message::UserInput(UserInputMessage::Exit));
    }

    Some(Message::UserInput(UserInputMessage::EnterEditMode(
        "Playlist - Name Taken".to_string(),
        InputTarget::PlaylistName(index),
    )))
}

pub fn add_tracks(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if let Some(selected) = playlist_ctl.selected_playlist
        && let Some(path_vec) = choose_multiple_audio_files()
    {
        let new_tracks: Vec<PathBuf> = path_vec.into_iter().filter(|p| p.is_file()).collect();
        new_tracks.iter().for_each(|p| {
            playlist_ctl
                .playlist_coll
                .add_tracks_to_playlist(p, selected)
        });
    }

    None
}

pub fn remove_selected_track(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if matches!(playlist_ctl.tab_focus, PlaylistTabFocus::Tracks)
        && let Some(playlist) = playlist_ctl.get_selected_playlist()
    {
        playlist.remove_selected();
        if playlist.is_empty() {
            playlist_ctl.tab_focus = PlaylistTabFocus::Playlists;
        }
    }

    None
}

pub fn add_dir(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if let Some(selected_playlist_idx) = playlist_ctl.selected_playlist
        && let Some(dirs) = choose_dirs()
    {
        dirs.iter()
            .for_each(|dir| match filter_dir_for_audio_files(dir) {
                Ok(vec_path) => {
                    let new_tracks: Vec<PathBuf> =
                        vec_path.into_iter().filter(|p| p.is_file()).collect();
                    new_tracks.into_iter().for_each(|t| {
                        playlist_ctl
                            .playlist_coll
                            .add_tracks_to_playlist(t.as_path(), selected_playlist_idx)
                    });
                }
                Err(e) => log::error!("{}", e),
            });
    }

    None
}

pub fn enqueue(app: &mut App) -> Option<Message> {
    match app.playlist_ctl.tab_focus {
        PlaylistTabFocus::Playlists => {
            if let Some(selected) = app.playlist_ctl.get_selected_playlist() {
                selected
                    .mini_tracks
                    .iter()
                    .for_each(|m| app.queue.enqueue_mini_track_ref(m.clone()));

                log::info!(
                    "Enqueued playlist \"{}\".",
                    selected.get_name().unwrap_or_default()
                );
            }
        }
        PlaylistTabFocus::Tracks => {
            if let Some(playlist) = app.playlist_ctl.get_selected_playlist()
                && let Some(index) = playlist.selected_track
                && let Some(mini_track) = playlist.mini_tracks.get(index)
            {
                app.queue.enqueue_mini_track_ref(mini_track.clone());
                log::info!("Enqueued \"{}\".", mini_track.borrow().path.display());
            }
        }
    }

    None
}

pub fn prepend_queue(app: &mut App) -> Option<Message> {
    match app.playlist_ctl.tab_focus {
        PlaylistTabFocus::Playlists => {
            if let Some(selected) = app.playlist_ctl.get_selected_playlist() {
                let tracks: VecDeque<Rc<RefCell<MiniTrack>>> =
                    selected.mini_tracks.iter().cloned().collect();
                app.queue.prepend_mini_track_refs(tracks);

                log::info!(
                    "Enqueued playlist \"{}\".",
                    selected.get_name().unwrap_or_default()
                );
            }
        }
        PlaylistTabFocus::Tracks => {
            if let Some(playlist) = app.playlist_ctl.get_selected_playlist()
                && let Some(index) = playlist.selected_track
                && let Some(mini_track) = playlist.mini_tracks.get(index)
            {
                app.queue
                    .prepend_mini_track_refs(VecDeque::from(vec![mini_track.clone()]));
                log::info!("Enqueued \"{}\".", mini_track.borrow().path.display());
            }
        }
    }

    None
}

pub fn navigate_playlists(
    direction: Direction,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    if !matches!(playlist_ctl.tab_focus, PlaylistTabFocus::Playlists) {
        return None;
    }

    match direction {
        Direction::Up => {
            if let Some(to_resume) = playlist_save_confirm_then_resume(
                Message::Playlist(PlaylistMessage::Navigate(direction)),
                playlist_ctl,
            ) {
                return Some(to_resume);
            }

            playlist_ctl.prev_playlist();
        }
        Direction::Down => {
            if let Some(to_resume) = playlist_save_confirm_then_resume(
                Message::Playlist(PlaylistMessage::Navigate(direction)),
                playlist_ctl,
            ) {
                return Some(to_resume);
            }
            playlist_ctl.next_playlist();
        }
        _ => {}
    }

    None
}

pub fn navigate_tracks(direction: Direction, playlist_ctl: &mut PlaylistController) {
    if !matches!(playlist_ctl.tab_focus, PlaylistTabFocus::Tracks) {
        return;
    }

    let arrange_mode = playlist_ctl.arrange_mode;

    if let Some(selected_playlist) = playlist_ctl.get_selected_playlist() {
        match direction {
            Direction::Up => {
                selected_playlist.select_prev_track(arrange_mode);
            }
            Direction::Down => {
                selected_playlist.select_next_track(arrange_mode);
            }
            _ => {}
        }
    }
}

pub fn move_cursor(direction: Direction, playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if playlist_ctl.playlist_coll.is_empty() {
        return None;
    }

    match direction {
        Direction::Left => {
            playlist_ctl.tab_focus = PlaylistTabFocus::Playlists;
        }
        Direction::Right => {
            if let Some(selected_playlist) = playlist_ctl.get_selected_playlist()
                && !selected_playlist.is_empty()
            {
                playlist_ctl.tab_focus = PlaylistTabFocus::Tracks;
            }
        }
        _ => match playlist_ctl.tab_focus {
            PlaylistTabFocus::Playlists => return navigate_playlists(direction, playlist_ctl),
            PlaylistTabFocus::Tracks => navigate_tracks(direction, playlist_ctl),
        },
    }

    None
}

pub fn delete_playlist(
    confirmation: &mut Option<Response>,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    if playlist_ctl.get_selected_playlist().is_some() {
        if let Some(confirm) = confirmation.take() {
            match confirm {
                Response::Yes => {
                    if let Err(e) = playlist_ctl.delete_selected_playlist() {
                        log::error!("Error deleting selected playlist: {e}")
                    }
                }
                Response::No => {}
            }
        } else {
            return Some(Message::AskConfirmation(
                "Delete?".to_string(),
                Box::new(Message::Playlist(PlaylistMessage::Delete)),
            ));
        }
    }

    None
}

/// Rename selected playlist unless index is specified.
pub fn rename_playlist(
    ui_message: &str,
    index: Option<usize>,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    index
        .or(playlist_ctl.selected_playlist)
        .filter(|&idx| playlist_ctl.playlist_coll.get_playlist(idx).is_some())
        .map(|index| {
            Message::UserInput(UserInputMessage::EnterEditMode(
                ui_message.to_string(),
                InputTarget::PlaylistName(index),
            ))
        })
}

pub fn save_selected_playlist(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    if let Err(e) = playlist_ctl.save_selected_to_file() {
        log::error!("Error saving selected playlist to file: {e}");
    };

    Some(Message::DisplayInfoMsg("Saved successfully".to_string()))
}

pub fn toggle_arrange(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    playlist_ctl.arrange_mode = !playlist_ctl.arrange_mode;

    None
}

pub fn ask_to_save(
    confirmation: &mut Option<Response>,
    then_call: Option<Message>,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    let current_playlist_index = playlist_ctl.selected_playlist;
    let metadata_loader_tx = playlist_ctl.playlist_coll.metadata_loader_tx.clone();
    if let Some(current_playlist) = playlist_ctl.get_selected_playlist()
        && current_playlist.is_dirty()
        && let Some(current_playlist_index) = current_playlist_index
    {
        if let Some(confirm) = confirmation.take() {
            match confirm {
                Response::Yes => {
                    if let Err(e) = playlist_ctl.save_selected_to_file() {
                        log::error!("Error saving playlist: {e}");
                    }

                    return then_call;
                }
                Response::No => {
                    if let Err(e) = current_playlist.reload_from_file() {
                        log::error!("Error reloading playlist: {e}");
                    }

                    if let Err(e) = metadata_loader_tx
                        .send((current_playlist_index, current_playlist.get_path_vec()))
                    {
                        log::error!("Error sending playlist to metadata loader: {e}");
                    }
                    return then_call;
                }
            }
        } else {
            return Some(Message::AskConfirmation(
                "Save? (discard if not)".to_string(),
                Box::new(Message::Playlist(PlaylistMessage::AskToSave(Box::new(
                    then_call,
                )))),
            ));
        }
    }

    None
}

pub fn scroll_to_end(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    match playlist_ctl.tab_focus {
        PlaylistTabFocus::Playlists => {
            if !playlist_ctl.playlist_coll.is_empty() {
                playlist_ctl.selected_playlist = Some(playlist_ctl.playlist_coll.len() - 1);
            }
        }
        PlaylistTabFocus::Tracks => {
            if let Some(selected_playlist) = playlist_ctl.get_selected_playlist()
                && !selected_playlist.is_empty()
            {
                selected_playlist.selected_track = Some(selected_playlist.len() - 1);
            }
        }
    }

    None
}

pub fn scroll_to_start(playlist_ctl: &mut PlaylistController) -> Option<Message> {
    match playlist_ctl.tab_focus {
        PlaylistTabFocus::Playlists => {
            if !playlist_ctl.playlist_coll.is_empty() {
                playlist_ctl.selected_playlist = Some(0);
            }
        }
        PlaylistTabFocus::Tracks => {
            if let Some(selected_playlist) = playlist_ctl.get_selected_playlist()
                && !selected_playlist.is_empty()
            {
                selected_playlist.selected_track = Some(0);
            }
        }
    }

    None
}

pub fn append_metadata(
    index: usize,
    mini_metadata: MiniMetadata,
    playlist_ctl: &mut PlaylistController,
) -> Option<Message> {
    if let Some(playlist) = playlist_ctl.playlist_coll.get_playlist(index) {
        for track in playlist.mini_tracks.iter_mut() {
            if track.borrow().metadata.is_none() {
                log::debug!("Appending metadata for {}.", track.borrow().path.display());
                track.borrow_mut().metadata = Some(mini_metadata);
                return None;
            }
        }
    }

    None
}

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::{
    app::App,
    global::{
        logic::files::{
            audio_paths_from_dir, choose_dirs, choose_multiple_audio_files,
            filter_paths_for_audio_files,
        },
        message::Message,
    },
    player::{self, logic::Player},
    playlist::cmd::rename_playlist,
    queue::logic::{TrackQueue, mini_track::MiniTrack},
};

pub fn queue_dirs_with_file_dialog(queue: &mut TrackQueue) -> Option<Message> {
    if let Some(dirs) = choose_dirs() {
        dirs.iter().for_each(|dir| {
            if let Err(e) = queue.tx.send(audio_paths_from_dir(dir)) {
                log::error!("Error sending Path Vec to queue processing worker: {e}");
            };
        });
    }

    None
}

pub fn queue_files_with_file_dialog(
    queue: &mut TrackQueue,
    player: &mut Player,
) -> Option<Message> {
    if let Some(mut path_vec) = choose_multiple_audio_files()
        && !path_vec.is_empty()
    {
        if player.current.is_none()
            && queue.is_empty()
            && let Some(first) = path_vec.first()
        {
            queue.enqueue_paths(vec![first.to_path_buf()]);
            path_vec.remove(0);
        }
        if let Err(e) = queue.tx.send(path_vec) {
            log::error!("Error sending Path Vec to queue processing worker: {e}");
        }
    }
    None
}

pub fn queue_paths(paths: Vec<PathBuf>, queue: &mut TrackQueue) -> Option<Message> {
    let mut valid_paths = filter_paths_for_audio_files(paths);

    if !valid_paths.is_empty() {
        if let Some(first) = valid_paths.first() {
            queue.enqueue_paths(vec![first.to_path_buf()]);
            valid_paths.remove(0);
        }
        if let Err(e) = queue.tx.send(valid_paths) {
            log::error!("Error sending Path Vec to queue processing worker: {e}");
        };
    }

    None
}

pub fn queue_mini_track(mini_track: MiniTrack, queue: &mut TrackQueue) -> Option<Message> {
    queue.enqueue_mini_track(mini_track);

    None
}

pub fn move_queue_up(queue: &mut TrackQueue) -> Option<Message> {
    if let Err(e) = queue.move_selected_up() {
        log::error!("{}", e);
    };

    None
}

pub fn move_queue_down(queue: &mut TrackQueue) -> Option<Message> {
    if let Err(e) = queue.move_selected_down() {
        log::error!("{}", e);
    };

    None
}

pub fn toggle_arrange(queue: &mut TrackQueue) -> Option<Message> {
    queue.toggle_arrange();

    None
}

pub fn shuffle(queue: &mut TrackQueue) -> Option<Message> {
    queue.shuffle();

    None
}

pub fn clear(queue: &mut TrackQueue) -> Option<Message> {
    queue.clear();

    None
}

pub fn remove_selected(queue: &mut TrackQueue) -> Option<Message> {
    queue.remove_selected();

    None
}

pub fn scroll_to_start(queue: &mut TrackQueue) -> Option<Message> {
    if !queue.is_empty() {
        queue.selected_index = Some(0)
    }

    None
}

pub fn scroll_to_end(queue: &mut TrackQueue) -> Option<Message> {
    if !queue.is_empty() {
        queue.selected_index = Some(queue.len() - 1)
    }

    None
}

pub async fn skip_to_selected(app: &mut App) -> Option<Message> {
    app.queue.skip_to_selected();
    player::cmd::play_next_track(app).await;

    None
}

pub async fn save_current_as_playlist(app: &mut App) -> Option<Message> {
    if !app.queue.is_empty() {
        let index = app.playlist_ctl.create_playlist();
        let playlist = app.playlist_ctl.playlist_coll.get_playlist(index).unwrap();
        if let Some(current_track) = app.player.current.as_mut() {
            playlist
                .mini_tracks
                .push(Rc::new(RefCell::new(MiniTrack::new(
                    &current_track.real_path,
                ))));
            if !playlist.is_empty() {
                playlist.selected_track = Some(0);
            }
        }

        app.queue
            .tracks
            .iter()
            .for_each(|t| playlist.mini_tracks.push(t.clone()));
        rename_playlist("Name Your Playlist", Some(index), &mut app.playlist_ctl)
    } else {
        None
    }
}

pub async fn save_full_as_playlist(app: &mut App) -> Option<Message> {
    if !app.queue.is_empty() {
        let index = app.playlist_ctl.create_playlist();
        let playlist = app.playlist_ctl.playlist_coll.get_playlist(index).unwrap();
        app.player
            .previous
            .iter()
            .for_each(|t| playlist.mini_tracks.push(Rc::new(RefCell::new(MiniTrack::new(&t.to_path_buf())))));

        if let Some(current_track) = app.player.current.as_mut() {
            playlist
                .mini_tracks
                .push(Rc::new(RefCell::new(MiniTrack::new(
                    &current_track.real_path,
                ))));
            if !playlist.is_empty() {
                playlist.selected_track = Some(0);
            }
        }

        app.queue
            .tracks
            .iter()
            .for_each(|t| playlist.mini_tracks.push(t.clone()));
        rename_playlist("Name Your Playlist", Some(index), &mut app.playlist_ctl)
    } else {
        None
    }
}

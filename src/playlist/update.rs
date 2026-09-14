use crate::{
    app::App,
    global::message::Message,
    playlist::{cmd::*, message::PlaylistMessage},
};

pub fn update_playlist(app: &mut App, msg: PlaylistMessage) -> Option<Message> {
    match msg {
        PlaylistMessage::Create => create_playlist(&mut app.playlist_ctl),
        PlaylistMessage::AddTracks => add_tracks(&mut app.playlist_ctl),
        PlaylistMessage::Enqueue => enqueue(app),
        PlaylistMessage::PrependQueue => prepend_queue(app),
        PlaylistMessage::Navigate(direction) => move_cursor(direction, &mut app.playlist_ctl),
        PlaylistMessage::AddDir => add_dir(&mut app.playlist_ctl),
        PlaylistMessage::Delete => {
            delete_playlist(&mut app.user_confirmation.response, &mut app.playlist_ctl)
        }
        PlaylistMessage::RemoveTrack => remove_selected_track(&mut app.playlist_ctl),
        PlaylistMessage::Rename => rename_playlist("Rename Playlist" ,None, &mut app.playlist_ctl),
        PlaylistMessage::SaveSelected => save_selected_playlist(&mut app.playlist_ctl),
        PlaylistMessage::ToggleArrangeTracks => toggle_arrange(&mut app.playlist_ctl),
        PlaylistMessage::AskToSave(then_call) => ask_to_save(
            &mut app.user_confirmation.response,
            *then_call,
            &mut app.playlist_ctl,
        ),
        PlaylistMessage::ScrollToStart => scroll_to_start(&mut app.playlist_ctl),
        PlaylistMessage::ScrollToEnd => scroll_to_end(&mut app.playlist_ctl),
        PlaylistMessage::LoadedMetadata(index, mini_metadata) => {
            append_metadata(index, mini_metadata, &mut app.playlist_ctl)
        }
    }
}

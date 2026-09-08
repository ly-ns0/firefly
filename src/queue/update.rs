use crate::{
    app::App,
    global::message::Message,
    queue::{cmd::*, message::QueueMessage},
};

pub async fn update_queue(app: &mut App, msg: QueueMessage) -> Option<Message> {
    match msg {
        QueueMessage::Clear => clear(&mut app.queue),
        QueueMessage::MoveDown => move_queue_down(&mut app.queue),
        QueueMessage::MoveUp => move_queue_up(&mut app.queue),
        QueueMessage::QueueDirsWithFileDialog => queue_dirs_with_file_dialog(&mut app.queue),
        QueueMessage::QueuePaths(pathbufs) => queue_paths(pathbufs, &mut app.queue),
        QueueMessage::QueueFilesWithFileDialog => {
            queue_files_with_file_dialog(&mut app.queue, &mut app.player)
        }
        QueueMessage::RemoveSelected => remove_selected(&mut app.queue),
        QueueMessage::ScrollToEnd => scroll_to_end(&mut app.queue),
        QueueMessage::ScrollToStart => scroll_to_start(&mut app.queue),
        QueueMessage::Shuffle => shuffle(&mut app.queue),
        QueueMessage::ToggleArrange => toggle_arrange(&mut app.queue),
        QueueMessage::CreatedMiniTrack(mini_track) => queue_mini_track(mini_track, &mut app.queue),
        QueueMessage::SkipToSelected => skip_to_selected(app).await,
        QueueMessage::SaveCurrentAsPlaylist => save_current_as_playlist(app).await,
        QueueMessage::SaveFullAsPlaylist => save_full_as_playlist(app).await,
    }
}

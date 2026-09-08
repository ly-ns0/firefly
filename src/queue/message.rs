use std::path::PathBuf;

use crate::queue::logic::mini_track::MiniTrack;

/// Messages related to TrackQueue
pub enum QueueMessage {
    ToggleArrange,
    MoveUp,
    MoveDown,
    QueueFilesWithFileDialog,
    QueueDirsWithFileDialog,
    QueuePaths(Vec<PathBuf>),
    Shuffle,
    Clear,
    ScrollToStart,
    ScrollToEnd,
    RemoveSelected,
    CreatedMiniTrack(MiniTrack),
    SkipToSelected,
    SaveCurrentAsPlaylist, // Save queue currently visible
    SaveFullAsPlaylist, // Save all queue tracks whether or not it's already played
}

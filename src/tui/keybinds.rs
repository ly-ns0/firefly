use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::App,
    global::{logic::confirmation::Response, message::Message, view::focused_area::FocusedArea},
    player::{logic::DEFAULT_VOLUME_CHANGE_AMOUNT, message::PlayerMessage},
    playlist::message::PlaylistMessage,
    queue::message::QueueMessage,
    tui::Direction,
    user_input::{logic::InputMode, message::UserInputMessage},
};

pub fn handle_key_inputs(key_event: KeyEvent, app: &App) -> Option<Message> {
    match app.input_mode {
        InputMode::Commands => match app.focused_view_area {
            FocusedArea::Playlist => match key_event.code {
                KeyCode::Esc => Some(Message::Quit),
                KeyCode::Char('a') => Some(Message::Playlist(PlaylistMessage::ToggleArrangeTracks)),
                KeyCode::Char('n') => Some(Message::Playlist(PlaylistMessage::Create)),
                KeyCode::Char('W') => Some(Message::Playlist(PlaylistMessage::AddDir)),
                KeyCode::Char('w') => Some(Message::Playlist(PlaylistMessage::AddTracks)),
                KeyCode::Char('s') => Some(Message::Player(PlayerMessage::Next)),
                KeyCode::Char('p') => Some(Message::Player(PlayerMessage::PreviousTrack)),
                KeyCode::Char('l') => Some(Message::Player(PlayerMessage::ToggleLoop)),
                KeyCode::Char('q') => Some(Message::Queue(QueueMessage::QueueFilesWithFileDialog)),
                KeyCode::Char('Q') => Some(Message::Queue(QueueMessage::QueueDirsWithFileDialog)),
                KeyCode::Char(' ') => Some(Message::Player(PlayerMessage::TogglePlay)),
                KeyCode::Char('=') => Some(Message::Player(PlayerMessage::IncreaseVolume(
                    DEFAULT_VOLUME_CHANGE_AMOUNT,
                ))),
                KeyCode::Char('\\') => Some(Message::Playlist(PlaylistMessage::PrependQueue)),
                KeyCode::Char('-') => Some(Message::Player(PlayerMessage::DecreaseVolume(
                    DEFAULT_VOLUME_CHANGE_AMOUNT,
                ))),
                KeyCode::Enter => Some(Message::Playlist(PlaylistMessage::Enqueue)),
                KeyCode::Delete => Some(Message::Playlist(PlaylistMessage::RemoveTrack)),
                KeyCode::Right => Some(Message::Playlist(PlaylistMessage::Navigate(
                    Direction::Right,
                ))),
                KeyCode::Left => Some(Message::Playlist(PlaylistMessage::Navigate(
                    Direction::Left,
                ))),
                KeyCode::Up => Some(Message::Playlist(PlaylistMessage::Navigate(Direction::Up))),
                KeyCode::Down => Some(Message::Playlist(PlaylistMessage::Navigate(
                    Direction::Down,
                ))),
                KeyCode::F(2) => Some(Message::Playlist(PlaylistMessage::Rename)),
                KeyCode::F(9) => Some(Message::Playlist(PlaylistMessage::Delete)),
                KeyCode::F(5) => Some(Message::Playlist(PlaylistMessage::SaveSelected)),
                KeyCode::Home => Some(Message::Playlist(PlaylistMessage::ScrollToStart)),
                KeyCode::End => Some(Message::Playlist(PlaylistMessage::ScrollToEnd)),
                KeyCode::Char('h') => Some(Message::ShowHelp),
                KeyCode::Tab => Some(Message::CycleTabs),
                _ => None,
            },

            _ => match key_event.code {
                KeyCode::Esc => Some(Message::Quit),
                KeyCode::Char('n') => Some(Message::Player(PlayerMessage::LoadNow)),
                KeyCode::Char(' ') => Some(Message::Player(PlayerMessage::TogglePlay)),
                KeyCode::Char('=') => Some(Message::Player(PlayerMessage::IncreaseVolume(
                    DEFAULT_VOLUME_CHANGE_AMOUNT,
                ))),
                KeyCode::Char('-') => Some(Message::Player(PlayerMessage::DecreaseVolume(
                    DEFAULT_VOLUME_CHANGE_AMOUNT,
                ))),
                KeyCode::Right => Some(Message::Player(PlayerMessage::Seek(None))),
                KeyCode::Left => Some(Message::Player(PlayerMessage::Rewind(None))),
                KeyCode::Char('l') => Some(Message::Player(PlayerMessage::ToggleLoop)),
                KeyCode::Char('q') => Some(Message::Queue(QueueMessage::QueueFilesWithFileDialog)),
                KeyCode::Char('Q') => Some(Message::Queue(QueueMessage::QueueDirsWithFileDialog)),
                KeyCode::Up => Some(Message::Queue(QueueMessage::MoveUp)),
                KeyCode::Down => Some(Message::Queue(QueueMessage::MoveDown)),
                KeyCode::Char('a') => Some(Message::Queue(QueueMessage::ToggleArrange)),
                KeyCode::Char('s') => Some(Message::Player(PlayerMessage::Next)),
                KeyCode::Char('p') => Some(Message::Player(PlayerMessage::PreviousTrack)),
                KeyCode::Char('m') => Some(Message::Queue(QueueMessage::Shuffle)),
                KeyCode::Char('h') => Some(Message::ShowHelp),
                KeyCode::Enter => Some(Message::Queue(QueueMessage::SkipToSelected)),
                KeyCode::Delete => Some(Message::Queue(QueueMessage::RemoveSelected)),
                KeyCode::Backspace => Some(Message::Queue(QueueMessage::Clear)),
                KeyCode::Home => Some(Message::Queue(QueueMessage::ScrollToStart)),
                KeyCode::End => Some(Message::Queue(QueueMessage::ScrollToEnd)),
                KeyCode::Tab => Some(Message::CycleTabs),
                KeyCode::F(5) => Some(Message::Queue(QueueMessage::SaveCurrentAsPlaylist)),
                KeyCode::F(6) => Some(Message::Queue(QueueMessage::SaveFullAsPlaylist)),
                _ => None,
            },
        },
        InputMode::Insert(_, to_edit) => match key_event.code {
            KeyCode::Enter => Some(Message::UserInput(UserInputMessage::Submit(to_edit))),
            KeyCode::Char(c) => Some(Message::UserInput(UserInputMessage::Insert(c))),
            KeyCode::Backspace => Some(Message::UserInput(UserInputMessage::Delete)),
            KeyCode::Left => Some(Message::UserInput(UserInputMessage::MoveCursorLeft)),
            KeyCode::Right => Some(Message::UserInput(UserInputMessage::MoveCursorRight)),
            KeyCode::Esc => Some(Message::UserInput(UserInputMessage::ExitEarly(to_edit))),
            _ => None,
        },
        InputMode::Confirmation => match key_event.code {
            KeyCode::Char('y') => Some(Message::Confirm(Response::Yes)),
            KeyCode::Char('n') => Some(Message::Confirm(Response::No)),
            KeyCode::Enter => Some(Message::Confirm(Response::Yes)),
            KeyCode::Esc => Some(Message::Confirm(Response::No)),
            _ => None,
        },
    }
}

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Clear, List, ListItem, Padding, Paragraph, Widget},
};

use crate::global::view::center_xy;

pub fn draw(frame: &mut Frame, area: Rect) {
    let area = center_xy(area, 80, (area.height * 8) / 10);
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50); 2])
        .split(vertical_chunks[0]);

    let player_c = get_player_controls();
    let player_c_max_len = player_c
        .iter()
        .map(|(desc, _)| desc.len())
        .max()
        .unwrap_or(0);

    let player_control_items: Vec<ListItem> = player_c
        .into_iter()
        .map(|(desc, key)| {
            let padded_desc = format!("{:<width$}", desc, width = player_c_max_len);
            ListItem::from(format!("{}  {}", padded_desc, key))
        })
        .collect();

    let player_controls = List::new(player_control_items).block(
        Block::bordered()
            .padding(Padding::new(2, 2, 1, 1))
            .title("Player")
            .title_alignment(Alignment::Left)
            .border_style(Style::default()),
    );

    let playlist_c = get_playlist_controls();
    let playlist_c_max_len = playlist_c
        .iter()
        .map(|(desc, _)| desc.len())
        .max()
        .unwrap_or(0);
    let playlist_control_items: Vec<ListItem> = playlist_c
        .into_iter()
        .map(|(desc, key)| {
            let padded_desc = format!("{:<width$}", desc, width = playlist_c_max_len);
            ListItem::from(format!("{}  {}", padded_desc, key))
        })
        .collect();

    let playlist_controls = List::new(playlist_control_items).block(
        Block::bordered()
            .padding(Padding::new(2, 2, 1, 1))
            .title("Playlist")
            .title_alignment(Alignment::Right)
            .border_style(Style::default()),
    );

    let instruction = Paragraph::new(
        "Press TAB to cycle through panels. Each panel has its own keyboard actions.",
    )
    .bold()
    .centered();

    Clear.render(area, frame.buffer_mut());
    player_controls.render(horizontal_chunks[0], frame.buffer_mut());
    playlist_controls.render(horizontal_chunks[1], frame.buffer_mut());
    instruction.render(vertical_chunks[1], frame.buffer_mut());
}

fn get_player_controls<'a>() -> Vec<(&'a str, &'a str)> {
    vec![
        ("Load Now", "<N>"),
        ("Queue", "<Q>"),
        ("Queue Dir", "<ShiftQ>"),
        ("Play/Pause", "<Space>"),
        ("Shuffle Queue", "<M>"),
        ("Arrange Queue", "<A>"),
        ("Skip to Selected Track in Queue", "<Enter>"),
        ("Remove Selected Track in Queue", "<Del>"),
        ("Clear Queue", "<Backspace>"),
        ("Move Queue Up", "<↑>"),
        ("Move Queue Down", "<↓>"),
        ("Save current (upcoming) Queue as Playlist", "<F5>"),
        ("Save full (session) Queue as Playlist", "<F6>"),
        ("Prev/Skip", "<P/S>"),
        ("Rewind/Seek", "<←/→>"),
        ("Volume", "<=/->"),
        ("Loop", "<L>"),
        ("Focus Playlist", "<Tab>"),
        ("Quit", "<Esc>"),
    ]
    .into_iter()
    .collect()
}

fn get_playlist_controls<'a>() -> Vec<(&'a str, &'a str)> {
    vec![
        ("Navigate", "<↑↓←→>"),
        ("New Playlist", "<N>"),
        ("Rename Playlist", "<F2>"),
        ("Save Playlist", "<F5>"),
        ("Delete Playlist", "<F9>"),
        ("Add Tracks", "<W>"),
        ("Add Directory", "<ShiftW>"),
        ("Remove Track", "<Del>"),
        ("Arrange Tracks", "<A>"),
        ("Enqueue Selected Entry", "<Enter>"),
        ("Prepend Selected Entry to Queue", "<\\>"),
        ("Focus Player", "<Tab>"),
    ]
    .into_iter()
    .collect()
}

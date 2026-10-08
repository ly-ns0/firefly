pub mod format_conversion;
pub mod mpris;
pub mod playback_status;
pub mod track;

use color_eyre::eyre::{Result, eyre};
use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Property, Server, Signal, Time};
use rodio::{OutputStream, OutputStreamBuilder, SampleRate, Sink};
use rust_ffmpeg::FFmpegProcess;
use std::{
    net::SocketAddr,
    ops::Add,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::{Mutex, RwLock, mpsc::UnboundedSender};

use crate::{
    global::message::Message,
    player::logic::{
        mpris::{MprisPlayer, MprisPlayerState, get_bus_name},
        track::Track,
    },
    queue::logic::TrackQueue,
};

const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 2.0;
pub const DEFAULT_VOLUME_CHANGE_AMOUNT: f32 = 0.05;
pub const DEFAULT_SAMPLE_RATE: SampleRate = 48_000;

/// Deals player-related stuff like current track, previous tracks, looping bool, sink, and MPRIS server.
pub struct Player {
    pub current: Option<Track>,
    // Previous is Vec instead of VecDeque because it's a stack, not queue.
    pub previous: Vec<PathBuf>,
    pub looping: bool,
    pub stream: OutputStream,
    pub sink: Sink,
    pub ffmpeg_handle: Option<Arc<Mutex<FFmpegProcess>>>,
    pub mpris_server: Option<Server<MprisPlayer>>,
    pub mpris_state: Option<Arc<RwLock<MprisPlayerState>>>,
    pub cover_server_addr: Option<SocketAddr>,
}

impl Player {
    // Sender is needed because it deals with threads.
    pub async fn new(
        async_msg_tx: UnboundedSender<Message>,
        cover_server_addr: Option<SocketAddr>,
        sample_rate: SampleRate,
    ) -> Result<Player> {
        let (stream, sink) = Self::get_sink(sample_rate)?;
        let mpris_state = Arc::new(RwLock::new(MprisPlayerState::default()));

        let mpris_server = Server::new_with_all(
            &get_bus_name().await,
            MprisPlayer {
                tx: async_msg_tx,
                state: mpris_state.clone(),
            },
        )
        .await
        .ok();

        let mpris_state = if mpris_server.is_some() {
            Some(mpris_state)
        } else {
            None
        };

        Ok(Player {
            current: None,
            previous: Vec::<PathBuf>::new(),
            looping: false,
            stream,
            sink,
            ffmpeg_handle: None,
            mpris_server,
            mpris_state,
            cover_server_addr,
        })
    }

    pub fn new_track(&mut self, path: &Path) -> Result<()> {
        self.current = Some(Track::new(path, self.cover_server_addr)?);

        Ok(())
    }

    pub fn get_sink(sample_rate: SampleRate) -> Result<(OutputStream, Sink)> {
        let mut stream_handle = match OutputStreamBuilder::from_default_device()?
            .with_sample_rate(sample_rate)
            .open_stream()
        {
            Ok(handle) => handle,
            Err(_) => OutputStreamBuilder::open_default_stream()?,
        };
        let sink = Sink::connect_new(stream_handle.mixer());

        stream_handle.log_on_drop(false);

        Ok((stream_handle, sink))
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub async fn set_volume(&mut self, amount: f32) -> Result<()> {
        self.sink.set_volume(amount.clamp(MIN_VOLUME, MAX_VOLUME));
        self.sync_mpris_volume().await?;

        Ok(())
    }

    pub async fn set_position(&mut self, position: Duration) -> Result<()> {
        if let Some(current_track) = self.current.as_ref()
            && let Some(dur) = current_track.duration
            && dur > position
        {
            if self.sink.empty() {
                self.reload().await?;
            }
            if let Err(e) = self.sink.try_seek(position) {
                return Err(eyre!("{e}"));
            }
        }

        self.sync_and_notify_mpris_pos().await?;

        Ok(())
    }

    pub async fn seek(&mut self, track_dur: &Duration, seek_dur: Duration) -> Result<()> {
        let current_pos = self.sink.get_pos();
        if current_pos.add(seek_dur) < *track_dur {
            if let Err(e) = self.sink.try_seek(current_pos.add(seek_dur)) {
                return Err(eyre!("{e}"));
            };
        } else if track_dur.saturating_sub(current_pos) < seek_dur
            && track_dur.saturating_sub(current_pos) > Duration::from_secs(1)
            && let Err(e) = self
                .sink
                .try_seek(track_dur.saturating_sub(Duration::from_secs(1)))
        {
            return Err(eyre!("{e}"));
        };

        self.sync_and_notify_mpris_pos().await?;

        Ok(())
    }

    pub async fn rewind(&mut self, rewind_dur: Duration) -> Result<()> {
        if self.current.is_some() {
            let current_pos = self.sink.get_pos();
            if self.sink.empty() {
                self.reload().await?;
                log::info!("Reloading...");
            }
            let rewinded_pos = current_pos.saturating_sub(rewind_dur);

            if let Err(e) = self.sink.try_seek(rewinded_pos) {
                return Err(eyre!("{e}"));
            };
            log::info!("Rewinding...");
        }

        self.sync_and_notify_mpris_pos().await?;

        Ok(())
    }

    pub async fn reload(&mut self) -> Result<()> {
        if let Some(current) = self.current.as_mut() {
            self.sink.clear();
            let source = current.get_source()?;
            self.sink.append(source);
            self.sink.play();
        }

        self.sync_and_notify_playback_status().await?;

        Ok(())
    }

    pub async fn load_next_track(&mut self, queue: &mut TrackQueue) -> Result<()> {
        let path = match queue.dequeue() {
            Some(path) => path,
            None => return Err(eyre!("Queue is empty.")),
        };

        if let Some(current) = self.current.as_mut() {
            self.previous.push(current.real_path.clone());
        }

        self.new_track(&path)?;

        self.sync_and_notify_metadata().await?;

        Ok(())
    }

    pub async fn load_prev_track(&mut self, queue: &mut TrackQueue) -> Result<()> {
        let prev = match self.previous.pop() {
            Some(path) => path,
            None => return Err(eyre!("There are no previous tracks.")),
        };

        if let Some(current) = self.current.as_mut() {
            if queue.is_empty() {
                queue.selected_index = Some(0);
            }
            queue.prepend_track(&current.real_path);
        }

        self.new_track(&prev)?;

        self.sync_and_notify_metadata().await?;

        Ok(())
    }

    pub async fn toggle_loop(&mut self) -> Result<()> {
        self.looping = !self.looping;
        self.sync_and_notify_loop_status().await?;

        Ok(())
    }

    pub fn sink_status_to_mpris_playback_status(&self) -> PlaybackStatus {
        if self.sink.is_paused() {
            PlaybackStatus::Paused
        } else if self.sink.empty() || self.sink.empty() && self.current.is_some() {
            PlaybackStatus::Stopped
        } else {
            PlaybackStatus::Playing
        }
    }

    pub async fn toggle_play(&mut self) -> Result<()> {
        if self.sink.empty() && self.current.is_some() {
            self.reload().await?;
        } else if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }

        self.sync_and_notify_playback_status().await?;

        Ok(())
    }

    pub async fn play(&mut self) -> Result<()> {
        self.sink.play();
        self.sync_and_notify_playback_status().await?;

        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        self.sink.pause();
        self.sync_and_notify_playback_status().await?;

        Ok(())
    }

    pub async fn notify_mpris_volume(&mut self) -> Result<()> {
        if let Some(mpris_server) = self.mpris_server.as_ref() {
            let vol = self.volume() as f64;
            mpris_server
                .properties_changed([Property::Volume(vol)])
                .await?;
        }

        Ok(())
    }

    pub async fn sync_mpris_volume(&mut self) -> Result<()> {
        if let Some(mpris_state) = self.mpris_state.as_ref() {
            let vol = self.volume() as f64;
            mpris_state.write().await.volume = vol;
        }

        Ok(())
    }

    pub async fn sync_and_notify_loop_status(&mut self) -> Result<()> {
        if let Some(mpris_server) = self.mpris_server.as_ref()
            && let Some(mpris_state) = self.mpris_state.as_ref()
        {
            let loop_status = if self.looping {
                LoopStatus::Track
            } else {
                LoopStatus::None
            };

            mpris_server
                .properties_changed([Property::LoopStatus(loop_status)])
                .await?;

            mpris_state.write().await.loop_status = loop_status;
        }

        Ok(())
    }

    pub async fn sync_and_notify_mpris_pos(&mut self) -> Result<()> {
        if let Some(mpris_server) = self.mpris_server.as_ref()
            && let Some(mpris_state) = self.mpris_state.as_ref()
        {
            let pos = Time::from_secs(self.sink.get_pos().as_secs() as i64);
            mpris_server.emit(Signal::Seeked { position: pos }).await?;
            log::info!("Player set mpris position to {:?}.", pos);
            mpris_state.write().await.position = pos;
        }

        Ok(())
    }

    pub async fn sync_and_notify_playback_status(&mut self) -> Result<()> {
        if let Some(mpris_server) = self.mpris_server.as_ref()
            && let Some(mpris_state) = self.mpris_state.as_ref()
        {
            mpris_server
                .properties_changed([Property::PlaybackStatus(
                    self.sink_status_to_mpris_playback_status(),
                )])
                .await?;

            mpris_state.write().await.playback_status = self.sink_status_to_mpris_playback_status();
        }

        Ok(())
    }

    pub async fn sync_and_notify_metadata(&mut self) -> Result<()> {
        if let Some(mpris_server) = self.mpris_server.as_ref()
            && let Some(mpris_state) = self.mpris_state.as_ref()
        {
            if let Some(current) = self.current.as_ref() {
                mpris_server
                    .properties_changed([Property::Metadata(current.metadata.clone())])
                    .await?;

                mpris_state.write().await.metadata = current.metadata.clone();
            } else {
                mpris_server
                    .properties_changed([Property::Metadata(Metadata::new())])
                    .await?;

                mpris_state.write().await.metadata = Metadata::new();
            }
        }

        Ok(())
    }

    pub async fn sync_and_notify_mpris_all(&mut self) -> Result<()> {
        self.sync_and_notify_mpris_pos().await?;
        self.sync_and_notify_playback_status().await?;
        self.sync_and_notify_metadata().await?;
        self.notify_mpris_volume().await?;
        self.sync_and_notify_loop_status().await?;
        Ok(())
    }

    pub async fn sync_mpris_pos(&mut self) -> Result<()> {
        if self.current.is_some()
            && let Some(mpris_state) = self.mpris_state.as_ref()
        {
            let pos = Time::from_secs(self.sink.get_pos().as_secs() as i64);
            mpris_state.write().await.position = pos;
            log::info!("Player set mpris position to {:?}.", pos);
        }

        Ok(())
    }
}

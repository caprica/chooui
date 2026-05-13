// Copyright (C) 2026  Caprica Software Limited
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Native Rust audio playback engine and event processing.
//!
//! This module provides the core audio playback logic, leveraging `rodio` and
//! `symphonia` for high-quality audio decoding and playback control. It also
//! implements a multi-band equalizer using `biquad` filters.

use anyhow::{Context, Result};
use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};
use rodio::{Decoder, OutputStream, Sink, Source, Sample};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig, MediaPlayback, MediaPosition};
use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex, mpsc::{Receiver, Sender}},
    thread,
    time::Duration,
};

use crate::{
    events::AppEvent,
    player::{AudioPlayer, PlayerState},
    model::TrackInfo,
};

const BANDS: usize = 18;
const FREQUENCIES: [f32; BANDS] = [
    20.0, 40.0, 63.0, 100.0, 160.0, 250.0, 400.0, 500.0, 630.0, 800.0, 1200.0, 2500.0, 5000.0, 8000.0, 10000.0, 12000.0, 15000.0, 20000.0,
];

#[derive(Debug, Clone)]
pub(crate) enum AudioPlayerCommand {
    PlayTrack(TrackInfo),
    Play,
    Pause,
    TogglePause,
    Seek(i32),
    SeekAbsolute(Duration),
    Stop,
    AdjustVolume(i32),
    ToggleMute,
    ResetEqualizer,
    UpdateEqualizerAmp(usize, f64),
}

struct EqSettings {
    preamp_gain: f32,
    band_gains: [f32; BANDS],
    dirty: bool,
}

impl EqSettings {
    fn new() -> Self {
        Self {
            preamp_gain: 1.0,
            band_gains: [0.0; BANDS],
            dirty: true,
        }
    }
}

/// Spawns the audio worker thread to process playback commands.
pub(crate) fn spawn_player_worker(
    command_rx: Receiver<AudioPlayerCommand>,
    event_tx: Sender<AppEvent>,
) {
    let error_tx = event_tx.clone();

    thread::spawn(move || {
        if let Err(e) = audio_player_worker(command_rx, event_tx) {
            let _ = error_tx.send(AppEvent::FatalError(format!("Audio worker failure: {:?}", e)));
        }
    });
}

fn audio_player_worker(
    command_rx: Receiver<AudioPlayerCommand>,
    event_tx: Sender<AppEvent>,
) -> Result<()> {
    let (_stream, stream_handle) = OutputStream::try_default().context("Failed to open audio output stream")?;
    let sink = Sink::try_new(&stream_handle).context("Failed to create audio sink")?;

    let eq_settings = Arc::new(Mutex::new(EqSettings::new()));

    let mut current_volume = sink.volume();
    let mut is_muted = false;
    let mut player_state = PlayerState::Stopped;
    let mut last_pos = Duration::ZERO;
    let mut current_track_duration = Duration::ZERO;

    // Media Controls setup - unique name per instance like VLC
    let pid = std::process::id();
    let dbus_name = format!("choon_commander_{}", pid);

    let config = PlatformConfig {
        dbus_name: &dbus_name,
        display_name: "Choon Commander",
        hwnd: None,
    };

    let mut controls = MediaControls::new(config).ok();

    if let Some(ref mut c) = controls {
        // Explicitly set initial volume so MPRIS knows we support it
        let _ = c.set_volume(current_volume as f64);

        let event_tx_clone = event_tx.clone();
        let _ = c.attach(move |event| {
            match event {
                MediaControlEvent::Play => { let _ = event_tx_clone.send(AppEvent::Play); }
                MediaControlEvent::Pause => { let _ = event_tx_clone.send(AppEvent::Pause); }
                MediaControlEvent::Toggle => { let _ = event_tx_clone.send(AppEvent::TogglePause); }
                MediaControlEvent::Next => { let _ = event_tx_clone.send(AppEvent::NextTrack); }
                MediaControlEvent::Previous => { let _ = event_tx_clone.send(AppEvent::PreviousTrack); }
                MediaControlEvent::Stop => { let _ = event_tx_clone.send(AppEvent::StopPlayback); }
                MediaControlEvent::SetPosition(pos) => { let _ = event_tx_clone.send(AppEvent::Seek(pos.0)); }
                MediaControlEvent::SeekBy(dir, dur) => {
                    let secs = dur.as_secs() as i32;
                    match dir {
                        souvlaki::SeekDirection::Forward => { let _ = event_tx_clone.send(AppEvent::SeekBy(secs)); }
                        souvlaki::SeekDirection::Backward => { let _ = event_tx_clone.send(AppEvent::SeekBy(-secs)); }
                    }
                }
                _ => {}
            }
        });
    }

    // Send the library's initial volume to sync UI
    event_tx.send(AppEvent::VolumeChanged((current_volume * 100.0) as u32))?;

    loop {
        while let Ok(command) = command_rx.try_recv() {
            match command {
                AudioPlayerCommand::PlayTrack(track) => {
                    let file = File::open(&track.filename).context(format!("Failed to open file: {}", &track.filename))?;
                    let source = Decoder::new(BufReader::new(file)).context("Failed to decode audio file")?;

                    let duration_secs = source.total_duration().map(|d| d.as_secs()).unwrap_or(0);
                    current_track_duration = Duration::from_secs(duration_secs);
                    event_tx.send(AppEvent::DurationChanged(duration_secs))?;

                    let eq_source = EqualizerSourceInner::new(source, Arc::clone(&eq_settings));

                    sink.stop();
                    sink.append(eq_source);
                    sink.play();

                    event_tx.send(AppEvent::TitleChanged(track.track_title.clone()))?;

                    // Update Media Controls Metadata
                    if let Some(ref mut c) = controls {
                        let metadata = MediaMetadata {
                            title: Some(&track.track_title),
                            artist: Some(&track.artist_name),
                            album: Some(&track.album_title),
                            duration: Some(current_track_duration),
                            ..Default::default()
                        };
                        let _ = c.set_metadata(metadata);
                    }
                }
                AudioPlayerCommand::Play => {
                    sink.play();
                }
                AudioPlayerCommand::Pause => {
                    sink.pause();
                }
                AudioPlayerCommand::TogglePause => {
                    if sink.is_paused() {
                        sink.play();
                    } else {
                        sink.pause();
                    }
                }
                AudioPlayerCommand::Seek(delta) => {
                    let current_pos = sink.get_pos();
                    let new_pos = if delta >= 0 {
                        current_pos + Duration::from_secs(delta as u64)
                    } else {
                        current_pos.saturating_sub(Duration::from_secs(delta.abs() as u64))
                    };
                    let _ = sink.try_seek(new_pos);
                }
                AudioPlayerCommand::SeekAbsolute(pos) => {
                    let _ = sink.try_seek(pos);
                }
                AudioPlayerCommand::Stop => {
                    sink.stop();
                    last_pos = Duration::ZERO;
                }
                AudioPlayerCommand::AdjustVolume(delta) => {
                    current_volume = (current_volume + (delta as f32 / 100.0)).clamp(0.0, 1.0);
                    if !is_muted {
                        sink.set_volume(current_volume);
                    }
                    event_tx.send(AppEvent::VolumeChanged((current_volume * 100.0) as u32))?;
                }
                AudioPlayerCommand::ToggleMute => {
                    is_muted = !is_muted;
                    if is_muted {
                        sink.set_volume(0.0);
                    } else {
                        sink.set_volume(current_volume);
                    }
                }

                AudioPlayerCommand::ResetEqualizer => {
                    let mut s = eq_settings.lock().unwrap();
                    s.preamp_gain = 1.0;
                    s.band_gains = [0.0; BANDS];
                    s.dirty = true;
                }
                AudioPlayerCommand::UpdateEqualizerAmp(index, value) => {
                    let mut settings = eq_settings.lock().unwrap();
                    if index == 0 {
                        // Preamp: convert dB to linear gain
                        settings.preamp_gain = 10.0f32.powf(value as f32 / 20.0);
                    } else if index <= BANDS {
                        settings.band_gains[index - 1] = value as f32;
                    }
                    settings.dirty = true;
                }
            }
        }

        let is_idle = sink.empty();
        let is_paused = sink.is_paused();

        let new_player_state = AudioPlayer::player_state(is_paused, is_idle);
        if new_player_state != player_state {
            player_state = new_player_state;
            event_tx.send(AppEvent::PlayerStateChanged(player_state))?;

            if is_idle && last_pos.as_secs() > 0 {
                 event_tx.send(AppEvent::TrackFinished)?;
            }

            // Update Media Controls Playback Status
            if let Some(ref mut c) = controls {
                let playback = match player_state {
                    PlayerState::Playing => MediaPlayback::Playing { progress: Some(MediaPosition(sink.get_pos())) },
                    PlayerState::Paused => MediaPlayback::Paused { progress: Some(MediaPosition(sink.get_pos())) },
                    PlayerState::Stopped => MediaPlayback::Stopped,
                };
                let _ = c.set_playback(playback);
            }
        }

        if !is_idle && !is_paused {
            let pos = sink.get_pos();
            if pos != last_pos {
                event_tx.send(AppEvent::TimeChanged(pos.as_secs_f64()))?;
                last_pos = pos;

                // Periodically update position in media controls
                if let Some(ref mut c) = controls {
                    let _ = c.set_playback(MediaPlayback::Playing { progress: Some(MediaPosition(pos)) });
                }
            }
        } else if is_idle {
            last_pos = Duration::ZERO;
        }

        thread::sleep(Duration::from_millis(50));
    }
}

struct EqualizerSourceInner<S: Source>
where
    S::Item: Sample,
{
    input: S,
    settings: Arc<Mutex<EqSettings>>,
    filters: Vec<[DirectForm2Transposed<f32>; 2]>,
    sample_rate: u32,
    channels: u16,
    current_channel: u16,
    preamp_gain: f32,
}

impl<S: Source> EqualizerSourceInner<S>
where
    S::Item: Sample,
{
    fn new(input: S, settings: Arc<Mutex<EqSettings>>) -> Self {
        let sample_rate = input.sample_rate();
        let channels = input.channels();

        let (preamp_gain, band_gains) = {
            let s = settings.lock().unwrap();
            (s.preamp_gain, s.band_gains)
        };

        let mut source = Self {
            input,
            settings,
            filters: Vec::new(),
            sample_rate,
            channels,
            current_channel: 0,
            preamp_gain,
        };
        source.rebuild_filters(preamp_gain, band_gains);
        source
    }

    fn rebuild_filters(&mut self, preamp_gain: f32, band_gains: [f32; BANDS]) {
        self.preamp_gain = preamp_gain;
        self.filters.clear();
        for (i, &freq) in FREQUENCIES.iter().enumerate() {
            let gain = band_gains[i];
            let coeffs = Coefficients::<f32>::from_params(
                Type::PeakingEQ(gain),
                self.sample_rate.hz(),
                freq.hz(),
                Q_BUTTERWORTH_F32,
            ).unwrap();

            self.filters.push([
                DirectForm2Transposed::<f32>::new(coeffs),
                DirectForm2Transposed::<f32>::new(coeffs),
            ]);
        }
    }
}

impl<S: Source> Iterator for EqualizerSourceInner<S>
where
    S::Item: Sample,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel == 0 {
            let (dirty, preamp, gains) = {
                let mut s = self.settings.lock().unwrap();
                if s.dirty {
                    s.dirty = false;
                    (true, s.preamp_gain, s.band_gains)
                } else {
                    (false, 0.0, [0.0; BANDS])
                }
            };
            if dirty {
                self.rebuild_filters(preamp, gains);
            }
        }

        let mut sample = self.input.next()?.to_f32();
        sample *= self.preamp_gain;

        let chan = self.current_channel as usize;
        for filter_pair in &mut self.filters {
            let filter_idx = if chan < 2 { chan } else { 0 };
            sample = filter_pair[filter_idx].run(sample);
        }

        self.current_channel = (self.current_channel + 1) % self.channels;
        Some(sample)
    }
}

impl<S: Source> Source for EqualizerSourceInner<S>
where
    S::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.input.try_seek(pos)?;
        self.current_channel = 0;
        Ok(())
    }
}

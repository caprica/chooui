use anyhow::Result;

use crate::{
    App, MainView, PlayMode, RepeatMode,
    events::{AppEvent, CatalogEvent},
    model::{Album, Artist, SearchQuery, Track, TrackInfo},
    player::PlayerState,
    tasks::AppTask,
};

pub(super) fn handle_catalog_event(app: &mut App, catalog_event: CatalogEvent) {
    match catalog_event {
        CatalogEvent::Started => app.catalog.prepare_scan(&app.config.media_dirs),
        CatalogEvent::StartedDirectory(dir) => app.catalog.begin_scan_directory(&dir),
        CatalogEvent::ProcessedFile(count, _) => app.catalog.update_scan_directory(count),
        CatalogEvent::FinishedDirectory(_) => app.catalog.end_scan_directory(),
        CatalogEvent::Finished(_) => app.catalog.finish_scan(),
    }
}

pub(super) fn handle_catalog_updated(app: &mut App) {
    app.task_tx.send(AppTask::GetBrowserArtists).unwrap();
}

pub(super) fn handle_set_main_view(app: &mut App, main_view: MainView) {
    app.playlist_view.is_active = matches!(main_view, MainView::Playlist);
    app.search_view.is_active = matches!(main_view, MainView::Search);
    app.favourites_view.is_active = matches!(main_view, MainView::Favourites);
    app.catalog_view.is_active = matches!(main_view, MainView::Catalog);

    if matches!(main_view, MainView::Browse) {
        app.favourites_view.is_active = false;
        app.catalog_view.is_active = false;
        app.playlist_view.is_active = false;
        app.search_view.is_active = false;
    }
    app.main_view = main_view;
}

pub(super) fn handle_new_search_query(app: &mut App, query: SearchQuery) -> Result<()> {
    app.task_tx.send(AppTask::Search(query))?;

    Ok(())
}

pub(super) fn handle_search_results_ready(app: &mut App, results: Vec<TrackInfo>) -> Result<()> {
    app.search.set_tracks(results);
    app.search_view.track_table.reset_table_selection();
    app.event_tx.send(AppEvent::SetMainView(MainView::Search))?;

    Ok(())
}

pub(super) fn handle_add_selection_to_playlist(app: &mut App) {
    let tracks = app.search_view.track_table.clone_selected_tracks();
    app.queue.add_tracks(tracks);
}

pub(super) fn handle_play_track(app: &mut App, track: TrackInfo) -> Result<()> {
    app.play_mode = PlayMode::PlayOne;
    app.audio_player.play_file(&track.filename)?;
    app.now_playing = Some(track);

    Ok(())
}

pub(super) fn handle_play_playlist(app: &mut App) -> Result<()> {
    app.play_mode = PlayMode::Playlist;
    if app.current_queue_idx.is_none() {
        let lock = app.queue.tracks();
        let tracks = lock.lock().unwrap();
        if !tracks.is_empty() {
            app.current_queue_idx = Some(0);
            if let Some(track) = tracks.get(0).cloned() {
                app.audio_player.play_file(&track.filename)?;
                app.now_playing = Some(track);
            }
        }
    }

    Ok(())
}

pub(super) fn handle_add_tracks_to_playlist(app: &mut App, tracks: Vec<TrackInfo>) -> Result<()> {
    app.queue.add_tracks(tracks);
    app.event_tx
        .send(AppEvent::SetMainView(MainView::Playlist))?;
    app.search_view.track_table.clear_selection();
    app.playlist_view.track_table.ensure_table_selection();

    Ok(())
}

pub(super) fn handle_artist_selection_changed(app: &mut App, id: i32) -> Result<()> {
    app.task_tx.send(AppTask::GetBrowserAlbums(id))?;

    Ok(())
}

pub(super) fn handle_album_selection_changed(app: &mut App, id: i32) -> Result<()> {
    app.task_tx.send(AppTask::GetBrowserTracks(id))?;

    Ok(())
}

pub(super) fn handle_add_tracks_to_queue(app: &mut App, tracks: Vec<TrackInfo>) {
    app.queue.add_tracks(tracks);
}

pub(super) fn handle_set_browser_artists(app: &mut App, artists: Vec<Artist>) -> Result<()> {
    let first_id = artists.first().map(|a| a.id);
    app.media_browser.set_artists(artists);
    if let Some(id) = first_id {
        app.event_tx.send(AppEvent::ArtistSelectionChanged(id))?;
    }

    Ok(())
}

pub(super) fn handle_set_browser_albums(app: &mut App, albums: Vec<Album>) -> Result<()> {
    let first_id = albums.first().map(|a| a.id);
    app.media_browser.set_albums(albums);
    if let Some(id) = first_id {
        app.event_tx.send(AppEvent::AlbumSelectionChanged(id))?;
    }

    Ok(())
}

pub(super) fn handle_set_browser_tracks(app: &mut App, tracks: Vec<Track>) -> Result<()> {
    let first_id = tracks.first().map(|t| t.id);
    app.media_browser.set_tracks(tracks);
    if let Some(id) = first_id {
        app.event_tx.send(AppEvent::TrackSelectionChanged(id))?;
    }

    Ok(())
}

pub(super) fn handle_set_now_playing(app: &mut App, info: TrackInfo) {
    app.now_playing = Some(info);
}

pub(super) fn handle_player_state_changed(app: &mut App, state: PlayerState) {
    app.player_state = state;
}

pub(super) fn handle_title_changed(app: &mut App, title: String) {
    app.player_track_name = Some(title);
}

pub(super) fn handle_duration_changed(app: &mut App, dur: u64) {
    app.player_duration = Some(dur);
}

pub(super) fn handle_volume_changed(app: &mut App, vol: u32) {
    app.volume = Some(vol);
}

pub(super) fn handle_track_finished(app: &mut App) -> Result<()> {
    app.player_time = app.player_duration;
    if app.play_mode == PlayMode::Playlist {
        let lock = app.queue.tracks();
        let tracks = lock.lock().unwrap();
        let total_tracks = tracks.len();

        if total_tracks == 0 {
            app.current_queue_idx = None;
            return Ok(());
        }

        if let Some(idx) = app.current_queue_idx.as_mut() {
            if app.repeat_mode != RepeatMode::RepeatOne {
                let next_idx = *idx + 1;
                if next_idx < total_tracks {
                    *idx = next_idx;
                } else if app.repeat_mode == RepeatMode::RepeatAll {
                    *idx = 0;
                } else {
                    app.current_queue_idx = None;
                }
            }

            if let Some(valid_idx) = app.current_queue_idx {
                if let Some(track) = tracks.get(valid_idx) {
                    app.audio_player.play_file(&track.filename)?;
                    app.now_playing = Some((*track).clone());
                }
            } else {
                app.now_playing = None;
            }
        }
    }

    Ok(())
}

pub(super) fn handle_time_changed(app: &mut App, seconds: f64) {
    app.player_time = Some(seconds as u64);
    if let Some(duration) = app.player_duration {
        app.player_position = if duration > 0 {
            Some(seconds / duration as f64)
        } else {
            None
        };
    }
}

pub(super) fn handle_tick(_app: &mut App) {}

pub(super) fn handle_add_selected_artist_to_queue(app: &mut App) {
    match app.main_view {
        MainView::Search => {
            let tracks = app.search_view.track_table.clone_selected_artist_tracks();
            app.queue.add_tracks(tracks);
        }
        _ => {}
    }
}

pub(super) fn handle_add_selected_album_to_queue(app: &mut App) {
    match app.main_view {
        MainView::Search => {
            let tracks = app.search_view.track_table.clone_selected_album_tracks();
            app.queue.add_tracks(tracks);
        }
        _ => {}
    }
}

pub(super) fn handle_add_selected_track_to_queue(app: &mut App) {
    match app.main_view {
        MainView::Search => {
            if let Some(track) = app.search_view.track_table.clone_selected_track() {
                app.queue.add_tracks(vec![track]);
            }
        }
        _ => {}
    }
}

pub(super) fn handle_clear_queue(app: &mut App) {
    app.current_queue_idx = None;
    app.queue.clear();
}

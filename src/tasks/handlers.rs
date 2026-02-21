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

use anyhow::Result;

use crate::{
    db::{self, scan},
    events::AppEvent,
    model::{Rating, SearchQuery, TrackInfo},
    tasks::TaskContext,
};

const MIN_SEARCH_LEN: usize = 3;

pub(super) fn scan_catalog(ctx: &mut TaskContext) -> Result<()> {
    ctx.event_tx.send(AppEvent::SetBrowserArtists(vec![]))?;
    ctx.event_tx.send(AppEvent::SetBrowserAlbums(vec![]))?;
    ctx.event_tx.send(AppEvent::SetBrowserTracks(vec![]))?;

    let music_dirs = &ctx.config.media_dirs;

    if let Err(e) = scan::process_music_library(ctx.conn, music_dirs, ctx.event_tx) {
        eprintln!("Failure processing catalog: {:#}", e);
    }

    ctx.event_tx.send(AppEvent::CatalogUpdated)?;

    Ok(())
}

pub(super) fn search(ctx: &mut TaskContext, query: SearchQuery) -> Result<()> {
    let can_search = query.search.len() >= MIN_SEARCH_LEN
        || query.artist.len() >= MIN_SEARCH_LEN
        || query.album.len() >= MIN_SEARCH_LEN
        || query.track.len() >= MIN_SEARCH_LEN;

    if can_search {
        let search_results = db::search(ctx.conn, &query)?;
        ctx.event_tx
            .send(AppEvent::SearchResultsReady(search_results))?;
    }

    Ok(())
}

pub(super) fn get_browser_artists(ctx: &mut TaskContext) -> Result<()> {
    let artists = db::fetch_artist_names(ctx.conn)?;
    ctx.event_tx.send(AppEvent::SetBrowserArtists(artists))?;

    Ok(())
}

pub(super) fn get_browser_albums(ctx: &mut TaskContext, artist_id: i32) -> Result<()> {
    let albums = db::fetch_artist_album_titles(ctx.conn, artist_id)?;
    ctx.event_tx.send(AppEvent::SetBrowserAlbums(albums))?;

    Ok(())
}

pub(super) fn get_browser_tracks(ctx: &mut TaskContext, album_id: i32) -> Result<()> {
    let tracks = db::fetch_album_tracks(ctx.conn, album_id)?;
    ctx.event_tx.send(AppEvent::SetBrowserTracks(tracks))?;

    Ok(())
}

pub(super) fn add_artist_to_queue(ctx: &mut TaskContext, artist_id: i32) -> Result<()> {
    let tracks = db::fetch_artist_trackinfo(ctx.conn, artist_id)?;
    ctx.event_tx.send(AppEvent::AddTracksToQueue(tracks))?;

    Ok(())
}

pub(super) fn add_album_to_queue(ctx: &mut TaskContext, album_id: i32) -> Result<()> {
    let tracks = db::fetch_album_track_info(ctx.conn, album_id)?;
    ctx.event_tx.send(AppEvent::AddTracksToQueue(tracks))?;

    Ok(())
}

pub(super) fn add_track_to_queue(ctx: &mut TaskContext, track_id: i32) -> Result<()> {
    let tracks = vec![db::fetch_track_info(ctx.conn, track_id)?];
    ctx.event_tx.send(AppEvent::AddTracksToQueue(tracks))?;

    Ok(())
}

pub(super) fn play_track(ctx: &mut TaskContext, track: TrackInfo) -> Result<()> {
    let durable_id = track.durable_id;
    ctx.event_tx.send(AppEvent::PlayTrack(track))?;
    db::increment_play_count(ctx.conn, durable_id)?;

    Ok(())
}

pub(super) fn rate_track(ctx: &mut TaskContext, track: TrackInfo, rating: Rating) -> Result<()> {
    db::update_rating(ctx.conn, track.durable_id, rating)?;

    Ok(())
}

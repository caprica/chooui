
enum SearchType {
    Artist,
    Album,
    Track
}

pub(crate) fn search_by_artist_name(conn: &Connection, name: &str) -> Result<Vec<TrackInfo>> {
    search_tracks(conn, SearchType::Artist, name)
}

pub(crate) fn search_by_album_title(conn: &Connection, title: &str) -> Result<Vec<TrackInfo>> {
    search_tracks(conn, SearchType::Album, title)
}

pub(crate) fn search_by_track_title(conn: &Connection, title: &str) -> Result<Vec<TrackInfo>> {
    search_tracks(conn, SearchType::Track, title)
}

fn search_tracks(conn: &Connection, search_type: SearchType, search_value: &str) -> Result<Vec<TrackInfo>> {
    let join = match search_type {
        SearchType::Artist => "ar.name",
        SearchType::Album => "al.title",
        SearchType::Track => "tr.title",
    };

    let sql = format!("
        SELECT ar.name, al.title, tr.id, tr.track_number, tr.title, tr.duration, tr.year, tr.genre, tr.filename
        FROM tracks tr
        JOIN albums al ON tr.album_id = al.id
        JOIN artists ar ON al.artist_id = ar.id
        WHERE {join} LIKE ?
        ORDER BY ar.name, al.title, tr.track_number
    ");

    let param = format!("%{}%", search_value);

    let mut stmt = conn.prepare_cached(&sql)?;
    let results = stmt.query_map([param], TrackInfo::from_row)?.collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

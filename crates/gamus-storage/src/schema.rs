// @generated automatically by Diesel CLI.

diesel::table! {
    artist_sites (id) {
        id -> Text,
        artist_id -> Text,
        url -> Text,
    }
}

diesel::table! {
    artist_variations (id) {
        id -> Text,
        artist_id -> Text,
        variation -> Text,
    }
}

diesel::table! {
    artists (id) {
        id -> Text,
        name -> Text,
        bio -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    artworks (id) {
        id -> Text,
        release_id -> Text,
        path -> Text,
        mime_type -> Text,
        description -> Nullable<Text>,
        hash -> Nullable<Text>,
        credits -> Nullable<Text>,
    }
}

diesel::table! {
    library_files (id) {
        id -> Text,
        release_track_id -> Text,
        path -> Text,
        size_bytes -> BigInt,
        modified_unix -> BigInt,
        duration_ms -> BigInt,
        bitrate_kbps -> Nullable<Integer>,
        sample_rate_hz -> Nullable<Integer>,
        channels -> Nullable<Integer>,
        fingerprint -> Nullable<Text>,
        bpm -> Nullable<Float>,
        quality_score -> Nullable<Float>,
        quality_assessment -> Nullable<Text>,
        features -> Nullable<Binary>,
        added_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    release_genres (id) {
        id -> Text,
        release_id -> Text,
        genre -> Text,
    }
}

diesel::table! {
    release_main_artists (id) {
        id -> Text,
        release_id -> Text,
        artist_id -> Text,
    }
}

diesel::table! {
    release_styles (id) {
        id -> Text,
        release_id -> Text,
        style -> Text,
    }
}

diesel::table! {
    release_track_artists (id) {
        id -> Text,
        release_track_id -> Text,
        artist_id -> Text,
        role -> Text,
        position -> Nullable<Integer>,
    }
}

diesel::table! {
    release_tracks (id) {
        id -> Text,
        release_id -> Text,
        song_id -> Text,
        disc_number -> Integer,
        track_number -> Integer,
        title_override -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    release_types (id) {
        id -> Text,
        release_id -> Text,
        kind -> Text,
    }
}

diesel::table! {
    releases (id) {
        id -> Text,
        title -> Text,
        release_date -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::table! {
    song_comments (id) {
        id -> Text,
        song_id -> Text,
        comment -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    song_ratings (id) {
        id -> Text,
        song_id -> Text,
        value_fixed_point -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    songs (id) {
        id -> Text,
        title -> Text,
        acoustid -> Nullable<Text>,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(artist_sites -> artists (artist_id));
diesel::joinable!(artist_variations -> artists (artist_id));
diesel::joinable!(artworks -> releases (release_id));
diesel::joinable!(library_files -> release_tracks (release_track_id));
diesel::joinable!(release_genres -> releases (release_id));
diesel::joinable!(release_main_artists -> artists (artist_id));
diesel::joinable!(release_main_artists -> releases (release_id));
diesel::joinable!(release_styles -> releases (release_id));
diesel::joinable!(release_track_artists -> artists (artist_id));
diesel::joinable!(release_track_artists -> release_tracks (release_track_id));
diesel::joinable!(release_tracks -> releases (release_id));
diesel::joinable!(release_tracks -> songs (song_id));
diesel::joinable!(release_types -> releases (release_id));
diesel::joinable!(song_comments -> songs (song_id));
diesel::joinable!(song_ratings -> songs (song_id));

diesel::allow_tables_to_appear_in_same_query!(
  artist_sites,
  artist_variations,
  artists,
  artworks,
  library_files,
  release_genres,
  release_main_artists,
  release_styles,
  release_track_artists,
  release_tracks,
  release_types,
  releases,
  song_comments,
  song_ratings,
  songs,
);

use crate::db::{Operation, Query, Table};
use crate::models::{EpisodeId, MovieId, SeasonId, ShowId, WishId};
use rusqlite::types::ToSqlOutput;

pub fn movie_request<'a>(id: MovieId) -> Option<(Query<'a>, String)> {
    let sql = "UPDATE movie SET source='none', request='' WHERE id=:id";
    let params = vec![(":id", ToSqlOutput::from(id))];

    let query = Query {
        id: id.0,
        sql,
        table: Table::Movies,
        op: Operation::Update,
        params,
    };

    Some((query, String::default()))
}

pub fn show_request<'a>(id: ShowId) -> Option<(Query<'a>, String)> {
    let sql = "UPDATE tv_show SET source='none', request='' WHERE id=:id";
    let params = vec![(":id", ToSqlOutput::from(id))];

    let query = Query {
        id: id.0,
        sql,
        table: Table::Show,
        op: Operation::Update,
        params,
    };

    Some((query, String::default()))
}

pub fn season_request<'a>(id: SeasonId) -> Option<(Query<'a>, String)> {
    let sql = "UPDATE season SET source='none', request='' WHERE id=:id";
    let params = vec![(":id", ToSqlOutput::from(id))];

    let query = Query {
        id: id.0,
        sql,
        table: Table::Season,
        op: Operation::Update,
        params,
    };

    Some((query, String::default()))
}

pub fn episode_request<'a>(id: EpisodeId) -> Option<(Query<'a>, String)> {
    let sql = "UPDATE episode SET source='none', request='' WHERE id=:id";
    let params = vec![(":id", ToSqlOutput::from(id))];

    let query = Query {
        id: id.0,
        sql,
        table: Table::Episode,
        op: Operation::Update,
        params,
    };

    Some((query, String::default()))
}

pub fn wish_request<'a>(id: WishId) -> Option<(Query<'a>, String)> {
    let sql = "UPDATE wish SET source='none', request='' WHERE id=:id";
    let params = vec![(":id", ToSqlOutput::from(id))];

    let query = Query {
        id: id.0,
        sql,
        table: Table::Wishlist,
        op: Operation::Update,
        params,
    };

    Some((query, String::default()))
}

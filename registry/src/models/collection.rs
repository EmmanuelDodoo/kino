use chrono::{DateTime, Local};
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef};
use uuid::Uuid;

use super::{EpisodeId, MovieId, SeasonId, ShowId, datetime_to_sql};
use crate::db::{Operation, Query, Table};

use core::variants;

pub mod triggers;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CollectionId(Uuid);

impl CollectionId {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        Self::from_row_heler("id", row)
    }

    /// Expects relevant column named "collection_id"
    pub fn from_member(row: &Row<'_>) -> Result<Self> {
        Self::from_row_heler("collection_id", row)
    }

    pub fn from_row_heler(column: &'static str, row: &Row<'_>) -> Result<Self> {
        row.get::<_, String>(column)
            .map(|id| CollectionId(Uuid::try_parse(&id).unwrap()))
    }
}

impl From<CollectionId> for ToSqlOutput<'_> {
    fn from(value: CollectionId) -> Self {
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ItemId {
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Episode(EpisodeId),
}

impl ItemId {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let kind = row.get::<_, String>("media_type")?;

        match kind.as_str() {
            "movie" => MovieId::from_collection(row).map(Self::Movie),
            "show" => ShowId::from_collection(row).map(Self::Show),
            "season" => SeasonId::from_collection(row).map(Self::Season),
            "episode" => EpisodeId::from_collection(row).map(Self::Episode),
            _ => unreachable!("stored invalid collection media"),
        }
    }

    pub fn from_random(row: &Row<'_>, media: &str) -> Result<Self> {
        match media {
            "movie" => MovieId::from_row(row).map(Self::Movie),
            "show" => ShowId::from_row(row).map(Self::Show),
            "season" => SeasonId::from_row(row).map(Self::Season),
            "episode" => EpisodeId::from_row(row).map(Self::Episode),
            _ => unreachable!("invalid media"),
        }
    }

    pub fn name_str(&self) -> &'static str {
        match self {
            Self::Movie(_) => "movie",
            Self::Show(_) => "show",
            Self::Season(_) => "season",
            Self::Episode(_) => "episode",
        }
    }
}

impl From<MovieId> for ItemId {
    fn from(value: MovieId) -> Self {
        Self::Movie(value)
    }
}

impl From<ShowId> for ItemId {
    fn from(value: ShowId) -> Self {
        Self::Show(value)
    }
}

impl From<SeasonId> for ItemId {
    fn from(value: SeasonId) -> Self {
        Self::Season(value)
    }
}

impl From<EpisodeId> for ItemId {
    fn from(value: EpisodeId) -> Self {
        Self::Episode(value)
    }
}

impl From<ItemId> for ToSqlOutput<'_> {
    fn from(value: ItemId) -> Self {
        match value {
            ItemId::Movie(id) => id.into(),
            ItemId::Show(id) => id.into(),
            ItemId::Season(id) => id.into(),
            ItemId::Episode(id) => id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleCollection {
    pub id: CollectionId,
    pub name: String,
    pub view: CollectionView,
    pub icon: Option<u32>,
}

impl SimpleCollection {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = CollectionId::from_row(row)?;

        let name = row.get::<_, String>("name")?;
        let view = row.get::<_, CollectionView>("view")?;
        let icon = row.get::<_, Option<u32>>("icon")?;

        Ok(Self {
            id,
            name,
            view,
            icon,
        })
    }

    pub fn from_collection(collection: &Collection) -> Self {
        Self {
            id: collection.id,
            name: collection.name.clone(),
            view: collection.view,
            icon: collection.icon,
        }
    }
    #[must_use]
    pub fn remove<'a>(self) -> Query<'a> {
        let sql = "UPDATE collection SET removed=TRUE WHERE id=:id";

        let params = [(":id", ToSqlOutput::from(self.id))];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub posters: Vec<Option<String>>,
    pub view: CollectionView,
    pub icon: Option<u32>,
    pub theme: Option<u32>,
    pub added: DateTime<Local>,
}

impl Collection {
    pub fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = CollectionId::from_row(row)?;

        let name = row.get::<_, String>("name")?;
        let description = row.get::<_, Option<String>>("description")?;

        let view = row.get::<_, CollectionView>("view")?;

        let icon = row.get::<_, Option<u32>>("icon")?;
        let theme = row.get::<_, Option<u32>>("theme")?;

        let added = row.get::<_, DateTime<Local>>("created_at")?;

        let posters = {
            let one = row.get::<_, Option<String>>("poster1")?;
            let two = row.get::<_, Option<String>>("poster2")?;
            let three = row.get::<_, Option<String>>("poster3")?;
            let four = row.get::<_, Option<String>>("poster4")?;

            vec![one, two, three, four]
        };

        Ok(Self {
            id,
            name,
            description,
            posters,
            view,
            icon,
            theme,
            added,
        })
    }

    fn insert_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            name,
            posters: _posters,
            description,
            view,
            icon,
            theme,
            added,
        } = self;

        let id = ToSqlOutput::from(*id);
        let name = ToSqlOutput::from(name.clone());
        let description = match description {
            Some(description) => ToSqlOutput::from(description.clone()),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let view = ToSqlOutput::from(*view);
        let icon = match icon {
            Some(icon) => ToSqlOutput::from(*icon),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let theme = match theme {
            Some(theme) => ToSqlOutput::from(*theme),
            None => ToSqlOutput::Owned(Value::Null),
        };

        let added = datetime_to_sql(added);

        vec![
            (":id", id),
            (":name", name),
            (":description", description),
            (":view", view),
            (":icon", icon),
            (":theme", theme),
            (":added", added),
        ]
    }

    fn update_params<'a>(&self) -> Vec<(&'a str, ToSqlOutput<'a>)> {
        let Self {
            id,
            name,
            posters: _posters,
            description,
            view,
            icon,
            theme,
            added: _added,
        } = self;

        let id = ToSqlOutput::from(*id);
        let name = ToSqlOutput::from(name.clone());
        let description = match description {
            Some(description) => ToSqlOutput::from(description.clone()),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let view = ToSqlOutput::from(*view);
        let icon = match icon {
            Some(icon) => ToSqlOutput::from(*icon),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let theme = match theme {
            Some(theme) => ToSqlOutput::from(*theme),
            None => ToSqlOutput::Owned(Value::Null),
        };

        vec![
            (":id", id),
            (":name", name),
            (":description", description),
            (":view", view),
            (":icon", icon),
            (":theme", theme),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO collection (id, name, description, view, icon, created_at, theme) VALUES (:id, :name, :description, :view, :icon, :added, :theme)";
        let params = self.insert_params();

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn save<'a>(&self) -> Query<'a> {
        let sql = "UPDATE collection SET name=:name, description=:description, view=:view, icon=:icon, theme=:theme WHERE id=:id";
        let params = self.update_params();

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params,
            op: Operation::Insert,
        }
    }

    #[must_use]
    pub fn remove<'a>(self) -> Query<'a> {
        let sql = "UPDATE collection SET removed=TRUE WHERE id=:id";

        let params = [(":id", ToSqlOutput::from(self.id))];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Delete,
        }
    }

    #[must_use]
    pub fn set_name<'a>(&mut self, name: String) -> Query<'a> {
        self.name = name;

        let sql = "UPDATE collection SET name=:name WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":name", ToSqlOutput::from(self.name.clone())),
        ];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_description<'a>(&mut self, description: Option<String>) -> Query<'a> {
        self.description = description;

        let sql = "UPDATE collection SET description=:description WHERE id=:id";
        let description = match &self.description {
            Some(description) => ToSqlOutput::from(description.clone()),
            None => ToSqlOutput::Owned(Value::Null),
        };

        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":description", description),
        ];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_view<'a>(&mut self, view: CollectionView) -> Query<'a> {
        self.view = view;

        let sql = "UPDATE collection SET view=:view WHERE id=:id";
        let params = [
            (":id", ToSqlOutput::from(self.id)),
            (":view", ToSqlOutput::from(self.view)),
        ];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_icon<'a>(&mut self, icon: Option<u32>) -> Query<'a> {
        self.icon = icon;

        let sql = "UPDATE collection SET icon=:icon WHERE id=:id";
        let icon = match icon {
            Some(icon) => ToSqlOutput::from(icon),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let params = [(":id", ToSqlOutput::from(self.id)), (":icon", icon)];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    #[must_use]
    pub fn set_theme<'a>(&mut self, theme: Option<u32>) -> Query<'a> {
        self.theme = theme;

        let sql = "UPDATE collection SET theme=:theme WHERE id=:id";
        let theme = match theme {
            Some(theme) => ToSqlOutput::from(theme),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let params = [(":id", ToSqlOutput::from(self.id)), (":theme", theme)];

        Query {
            id: self.id.0,
            table: Table::Collection,
            sql,
            params: params.to_vec(),
            op: Operation::Update,
        }
    }

    pub fn new<'a>(
        name: String,
        description: Option<String>,
        view: CollectionView,
        icon: Option<u32>,
        theme: Option<u32>,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: CollectionId(Uuid::now_v7()),
            name,
            posters: vec![],
            description,
            view,
            icon,
            theme,
            added,
        };

        let query = new.insert();

        (new, query)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum CollectionView {
    Pinned = 0,
    #[default]
    Shown = 1,
    Hidden = 2,
}

impl CollectionView {
    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "pinned" => Some(Self::Pinned),
            "shown" => Some(Self::Shown),
            "hidden" => Some(Self::Hidden),
            _ => None,
        }
    }
}

impl FromSql for CollectionView {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| CollectionView::parse_str(s).ok_or(FromSqlError::InvalidType))
    }
}

impl From<CollectionView> for ToSqlOutput<'_> {
    fn from(value: CollectionView) -> Self {
        ToSqlOutput::from(value.to_string())
    }
}

impl std::fmt::Display for CollectionView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Shown => "shown",
                Self::Pinned => "pinned",
                Self::Hidden => "hidden",
            }
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Sort {
    #[default]
    Name,
    View,
    Added,
}

impl Sort {
    pub fn query(&self) -> &str {
        match self {
            Self::Name => "name",
            Self::View => "view",
            Self::Added => "created_at",
        }
    }
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Added => "Added",
                Self::View => "Visibility",
                Self::Name => "Name",
            }
        )
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Items {
    All,
    Movies,
    Shows,
    Seasons,
    Episodes,
}}

impl Items {
    pub const TAGS: [Items; 2] = [Self::Movies, Self::Shows];

    pub fn parse_str(s: &str) -> Option<Self> {
        match s {
            "all" => Some(Self::All),
            "movies" => Some(Self::Movies),
            "shows" => Some(Self::Shows),
            "seasons" => Some(Self::Seasons),
            "episodes" => Some(Self::Episodes),
            _ => None,
        }
    }
}

impl FromSql for Items {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()
            .and_then(|s| Items::parse_str(s).ok_or(FromSqlError::InvalidType))
    }
}

impl<'a> From<Items> for ToSqlOutput<'a> {
    fn from(value: Items) -> Self {
        ToSqlOutput::from(value.to_string())
    }
}

impl std::fmt::Display for Items {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::All => "all",
                Self::Shows => "shows",
                Self::Movies => "movies",
                Self::Seasons => "seasons",
                Self::Episodes => "episodes",
            }
        )
    }
}

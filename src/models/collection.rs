use chrono::{DateTime, Local, NaiveDate};
use rusqlite::Row;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef};
use std::path::PathBuf;
use uuid::Uuid;

use super::{
    DirectoryId, Episode, EpisodeId, Movie, MovieId, Season, SeasonId, Show, ShowId,
    datetime_to_sql, naivedate_to_sql,
};
use crate::db::{Database, Operation, Query, Table};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CollectionId(Uuid);

impl<'a> From<CollectionId> for ToSqlOutput<'_> {
    fn from(value: CollectionId) -> Self {
        // todo!: to_string is needed because the raw string is fed into the db via
        // the dummy inputs. Production shouldn't need this.
        ToSqlOutput::from(value.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum ItemId {
    Movie(MovieId),
    Show(ShowId),
    Season(SeasonId),
    Episode(EpisodeId),
}

impl ItemId {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = row.get::<_, String>("media_id")?;
        let kind = row.get::<_, String>("media_type")?;

        match kind.as_str() {
            "movie" => {
                let id = MovieId(Uuid::try_parse(&id).unwrap());
                Ok(Self::Movie(id))
            }
            "show" => {
                let id = ShowId(Uuid::try_parse(&id).unwrap());
                Ok(Self::Show(id))
            }
            "season" => {
                let id = SeasonId(Uuid::try_parse(&id).unwrap());
                Ok(Self::Season(id))
            }
            "episode" => {
                let id = EpisodeId(Uuid::try_parse(&id).unwrap());
                Ok(Self::Episode(id))
            }
            _ => unreachable!("stored invalid collection media"),
        }
    }
}

impl<'a> From<ItemId> for ToSqlOutput<'_> {
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
pub struct Collection {
    pub id: CollectionId,
    name: String,
    description: Option<String>,
    posters: Vec<Option<String>>,
    view: CollectionView,
    icon: Option<u32>,
    theme: Option<u32>,
    custom: Option<String>,
    added: DateTime<Local>,
}

impl Collection {
    pub fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let id = row.get::<_, String>("id")?;
        let id = CollectionId(Uuid::try_parse(&id).unwrap());

        let name = row.get::<_, String>("name")?;
        let description = row.get::<_, Option<String>>("description")?;

        let view = row.get::<_, CollectionView>("view")?;

        let icon = row.get::<_, Option<u32>>("icon")?;
        let theme = row.get::<_, Option<u32>>("theme")?;
        let custom = row.get::<_, Option<String>>("custom")?;

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
            custom,
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
            custom,
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

        let custom = match custom {
            Some(custom) => ToSqlOutput::from(custom.clone()),
            None => ToSqlOutput::Owned(Value::Null),
        };

        let added = datetime_to_sql(&added);

        vec![
            (":id", id),
            (":name", name),
            (":description", description),
            (":view", view),
            (":icon", icon),
            (":theme", theme),
            (":custom", custom),
            (":added", added),
        ]
    }

    #[must_use]
    pub fn insert<'a>(&self) -> Query<'a> {
        let sql = "INSERT INTO collection (id, name, description, view, icon, custom, created_at, theme) VALUES (:id, :name, :description, :view, :icon, :custom, :added, :theme)";
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
    pub fn delete<'a>(self) -> Query<'a> {
        let sql = "DELETE FROM collection WHERE id=:id";

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

    #[must_use]
    pub fn set_custom<'a>(&mut self, custom: Option<String>) -> Query<'a> {
        self.custom = custom;

        let sql = "UPDATE collection SET custom=:custom WHERE id=:id";
        let custom = match &self.custom {
            Some(custom) => ToSqlOutput::from(custom.clone()),
            None => ToSqlOutput::Owned(Value::Null),
        };
        let params = [(":id", ToSqlOutput::from(self.id)), (":custom", custom)];

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
        icon: Option<u32>,
        theme: Option<u32>,
        custom: Option<String>,
    ) -> (Self, Query<'a>) {
        let added = Local::now();

        let new = Self {
            id: CollectionId(Uuid::now_v7()),
            name,
            posters: vec![],
            description,
            view: CollectionView::Shown,
            icon,
            theme,
            custom,
            added,
        };

        let query = new.insert();

        (new, query)
    }

    pub fn dummy<'a>() -> (Self, Query<'a>) {
        let name = "Test Collection".into();
        let description = Some("Dummy collection for testing".into());
        let icon = None;
        let custom = None;
        let theme = None;

        Self::new(name, description, icon, theme, custom)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum CollectionView {
    Pinned,
    #[default]
    Shown,
    Hidden,
}

impl CollectionView {
    pub fn from_str(s: &str) -> Option<Self> {
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
            .and_then(|s| CollectionView::from_str(s).ok_or(FromSqlError::InvalidType))
    }
}

impl<'a> From<CollectionView> for ToSqlOutput<'a> {
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

#[derive(Debug, Clone)]
pub enum Item {
    Movie(Movie),
    Show(Show),
    Season(Season),
    Episode(Episode),
}

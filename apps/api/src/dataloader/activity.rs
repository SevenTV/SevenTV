use itertools::Itertools;
use scuffle_foundations::dataloader::{DataLoader, Loader, LoaderOutput};
use shared::database::{EmoteActivity, EmoteId};

pub struct EmoteActivityByEmoteIdLoader {
    db: clickhouse::Client,
}

impl EmoteActivityByEmoteIdLoader {
    pub fn new(db: clickhouse::Client) -> DataLoader<Self> {
        DataLoader::new("EmoteActivityByEmoteIdLoader", Self { db })
    }
}

impl Loader for EmoteActivityByEmoteIdLoader {
    type Error = ();
    type Key = EmoteId;
    type Value = Vec<EmoteActivity>;

    #[tracing::instrument(name = "EmoteActivityByEmoteIdLoader::load", skip(self), fields(key_count = keys.len()))]
    async fn load(&self, keys: Vec<Self::Key>) -> LoaderOutput<Self> {
        let activities: Self::Value = self.db.query("SELECT * FROM emote_activities WHERE emote_id IN (?)")
            .bind(keys.into_iter().map(|id| id.as_uuid()).collect::<Vec<_>>())
            .fetch_all()
            .await
            .map_err(|err| {
                tracing::error!("failed to load emote activity: {err}");
            })?;
        
        Ok(activities.into_iter().into_group_map_by(|e| EmoteId::from(e.emote_id)))
    }
}

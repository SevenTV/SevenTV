use std::collections::HashMap;
use std::sync::Arc;

use shared::database::emote_set::{EmoteSet, EmoteSetId};
use shared::database::global::GlobalConfig;

use super::JobOutcome;
use crate::global::Global;
use crate::types;

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub global_config: &'a mut GlobalConfig,
    pub emote_sets: &'a mut HashMap<EmoteSetId, EmoteSet>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let RunInput { global, global_config, emote_sets } = input;

    let Some(system) = global.main_source_db.collection::<types::System>("system").find_one(bson::doc! {}).await? else {
        anyhow::bail!("system not found");
    };

    let emote_set_id = EmoteSetId::from(system.emote_set_id);
    if !emote_sets.contains_key(&emote_set_id) {
        anyhow::bail!("emote set not found");
    }

    global_config.emote_set_id = emote_set_id;
    global_config.country_currency_overrides = HashMap::from_iter([
        
    ]);

    let mut outcome = JobOutcome::new("system");

    outcome.processed_documents += 1;

	Ok(outcome)
}

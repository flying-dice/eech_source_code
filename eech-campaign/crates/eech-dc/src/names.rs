//! Ids as names: the campaign configuration names DCS objects, so events and
//! snapshots cross into Lua with those names (`#slot.generation` for entities
//! the campaign created itself, such as missions and crates).

use std::collections::HashMap;

use eech_campaign::EntityId;

pub type Names = HashMap<EntityId, String>;

/// Every `EntityId` in a serialised value, replaced by its name.
pub fn name_ids(value: &mut serde_json::Value, names: &Names) {
    match value {
        serde_json::Value::Object(map) => {
            if map.len() == 3 && map.contains_key("slot") && map.contains_key("generation") && map.contains_key("instance") {
                if let Ok(id) = serde_json::from_value::<EntityId>(serde_json::Value::Object(map.clone())) {
                    *value = serde_json::Value::String(names.get(&id).cloned().unwrap_or_else(|| id.to_string()));
                    return;
                }
            }
            for v in map.values_mut() {
                name_ids(v, names);
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| name_ids(v, names)),
        _ => {}
    }
}

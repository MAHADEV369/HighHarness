//! Interventions store, per `HARNESS_PRIMITIVES.md` §10.3.

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::{HxError, HxResult};
use crate::store::interventions_dir;

/// Write an intervention record.
pub fn write(root: &Path, id: &str, payload: Value) -> HxResult<()> {
    let dir = interventions_dir(root);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", id));
    fs::write(&path, serde_json::to_string_pretty(&payload)?)?;
    Ok(())
}

/// Read an intervention record.
pub fn read(root: &Path, id: &str) -> HxResult<Value> {
    let path = interventions_dir(root).join(format!("{}.json", id));
    let raw = fs::read_to_string(&path)?;
    let v: Value = serde_json::from_str(&raw)?;
    // W5: verify self_hash on non-legacy rows
    if let Some(h) = v.get("self_hash").and_then(|x| x.as_str()) {
        let mut for_hash = v.clone();
        if let Some(obj) = for_hash.as_object_mut() {
            obj.remove("self_hash");
        }
        let json = serde_json::to_string(&for_hash)?;
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(json.as_bytes());
        let computed = format!("{:x}", hasher.finalize());
        if computed != *h {
            return Err(HxError::AuditForgery(
                "intervention self_hash mismatch".to_string(),
            ));
        }
    }
    Ok(v)
}

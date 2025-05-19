use log::debug;
use temporal_sdk_core::protos::coresdk::activity_result::{
    activity_resolution::Status::Completed, ActivityResolution,
};
use serde::de::DeserializeOwned;

pub fn parse_activity_result<T>(result: &ActivityResolution) -> Result<T, anyhow::Error>
where
    T: DeserializeOwned,
{
    if result.completed_ok() {
        if let Some(Completed(completed)) = &result.status {
            if let Some(payload) = &completed.result {
                if payload.data.is_empty() {
                    debug!("Empty payload, returning default value");
                    return serde_json::from_str("null").map_err(|e| e.into());
                }

                return serde_json::from_slice(&payload.data).map_err(|e| e.into());
            }
        } else {
            debug!("Activity did not complete successfully: {:?}", result.status);
        }
    }

    Err(anyhow::anyhow!("Activity failed"))
}

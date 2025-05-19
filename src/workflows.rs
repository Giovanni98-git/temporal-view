use std::time::Duration;
use log::{debug, info, warn};
use temporal_sdk::{ActContext, ActExitValue, ActivityError, ActivityOptions, WfContext, WfExitValue, WorkflowResult};
use anyhow::Result;
use temporal_sdk_core_protos::{coresdk::AsJsonPayloadExt, temporal::api::common::v1::RetryPolicy};
use tokio::time::sleep;
use prost_wkt_types::Duration as ProstDuration;

use crate::helpers::parse_activity_result::parse_activity_result;

// activité
pub async fn repeat_activity(
    _ctx: ActContext,
    _payload: Option<String>,
) -> Result<ActExitValue<String>, ActivityError> {
    log::info!("🚀 Starting repeat_activity");
    let mut elapsed = Duration::ZERO;
    let interval = Duration::from_secs(5);
    let total_duration = Duration::from_secs(60);

    while elapsed < total_duration {
        sleep(interval).await;
        elapsed += interval;
        info!("{} seconds passed", elapsed.as_secs());
    }

    let result = format!("⏱ Done after {} seconds", elapsed.as_secs());
    Ok(ActExitValue::Normal(result))
}


pub async fn repeat_workflow(ctx: WfContext) -> WorkflowResult<()> {
    use log::{debug, info, warn};
    debug!("🚀 Starting repeat_workflow");

    let activity_result = ctx
        .activity(ActivityOptions {
            activity_type: "repeat_activity".to_string(),
            input: "".as_json_payload()?,
            retry_policy: Some(RetryPolicy {
                initial_interval: Some(ProstDuration {
                    seconds: 1,
                    nanos: 0,
                }),
                maximum_attempts: 1,
                ..Default::default()
            }),
            start_to_close_timeout: Some(Duration::from_secs(70)),
            ..Default::default()
        })
        .await;

    match parse_activity_result::<String>(&activity_result) {
        Ok(result) => {
            info!("✅ Activity result: {}", result);
            Ok(WfExitValue::Normal(()))
        }
        Err(err) => {
            warn!("❌ Activity failed: {:?}", err);
            Ok(WfExitValue::Evicted)
        }
    }
}

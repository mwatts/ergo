use crate::{error::Error, vault::VaultPostgresPool};
use serde::{Deserialize, Serialize};

use super::Task;

#[derive(Debug, Serialize, Deserialize)]
pub struct InputCategory {
    pub input_category_id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub input_id: i64,
    pub input_category_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub payload_schema: serde_json::Value, // TODO make this a JsonSchema
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputsLog {
    pub inputs_log_id: i64,
    pub input_id: i64,
    pub payload: serde_json::Value,
    pub error: serde_json::Value,
    pub time: chrono::DateTime<chrono::Utc>,
}

pub fn validate_input_payload(
    input_id: i64,
    payload_schema: &serde_json::Value,
    payload: &serde_json::Value,
) -> Result<(), Error> {
    let compiled_schema = jsonschema::JSONSchema::compile(&payload_schema)?;
    compiled_schema.validate(&payload)?;
    Ok(())
}

pub async fn enqueue_input(
    pg: &VaultPostgresPool<()>,
    task_id: i64,
    input_id: i64,
    task_trigger_id: i64,
    payload_schema: &serde_json::Value,
    payload: &serde_json::Value,
) -> Result<(), Error> {
    validate_input_payload(input_id, payload_schema, payload)?;

    sqlx::query!(
        r##"INSERT INTO event_queue
        (task_id, input_id, task_trigger_id, payload) VALUES
        ($1, $2, $3, $4)"##,
        task_id,
        input_id,
        task_trigger_id,
        payload
    )
    .execute(pg)
    .await?;

    Ok(())
}

pub async fn apply_input(
    pg: &VaultPostgresPool<()>,
    task_id: i64,
    input_id: i64,
    task_trigger_id: i64,
    payload: &serde_json::Value,
) -> Result<(), Error> {
    let mut task = Task::from_db(pg, task_id).await?.ok_or(Error::NotFound)?;
    task.apply_input(input_id, task_trigger_id, payload).await?;

    Ok(())
}

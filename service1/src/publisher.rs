use lapin::{
    options::{BasicPublishOptions, QueueDeclareOptions},
    publisher_confirm::Confirmation,
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use std::sync::OnceLock;
use crate::config::AppConfig;

static SHARED_CHANNEL: OnceLock<Mutex<Option<Arc<AsyncMutex<Channel>>>>> = OnceLock::new();

fn shared_channel() -> &'static Mutex<Option<Arc<AsyncMutex<Channel>>>> {
    SHARED_CHANNEL.get_or_init(|| Mutex::new(None))
}

pub async fn init_channel(app_config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = app_config.amqp_addr.clone();
    let conn = Connection::connect(&*addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            &*app_config.amqp_queue.to_string(),
            QueueDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut lock = shared_channel().lock().unwrap();
    *lock = Some(Arc::new(AsyncMutex::new(channel)));
    Ok(())
}

pub fn get_channel() -> Option<Arc<AsyncMutex<Channel>>> {
    let lock = shared_channel().lock().unwrap();
    lock.clone()
}

pub async fn queue_publish(json_payload: &str) -> Result<Confirmation, Box<dyn std::error::Error>> {
    let channel = get_channel().ok_or("Channel not initialized")?;
    let ch = channel.lock().await;
    let confirm = ch
        .basic_publish(
            "",
            "sensor_queue",
            BasicPublishOptions::default(),
            json_payload.as_bytes(),
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?
        .await?;

    Ok(confirm)
}

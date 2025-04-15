use crate::config::AppConfig;
use crate::sensors::SensorReading;
use lapin::{
    options::*,
    BasicProperties,
    Channel,
    Connection,
    ConnectionProperties,
    Error as LapinError,
};
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time::Instant;

async fn connect_to_rabbitmq(amqp_addr: &str) -> Result<Connection, LapinError> {
    println!("[AMQP Connect] Attempting connection to {}...", amqp_addr);
    let options = ConnectionProperties::default();
    Connection::connect(amqp_addr, options).await
}

async fn create_rabbitmq_channel(connection: &Connection) -> Result<Channel, LapinError> {
    println!("[AMQP Channel] Creating channel...");
    connection.create_channel().await
}

fn serialize_reading_to_json(reading: &SensorReading) -> Result<String, serde_json::Error> {
    serde_json::to_string(reading)
}

async fn publish_message_to_rabbitmq(
    channel: &Channel,
    queue_name: &str,
    json_payload: &str,
) -> Result<(), LapinError> {
    let payload_bytes = json_payload.as_bytes();
    let properties = BasicProperties::default()
        .with_content_type("application/json".into())
        .with_delivery_mode(2);
    let publish_options = BasicPublishOptions::default();

    match channel
        .basic_publish(
            "",
            queue_name,
            publish_options,
            payload_bytes,
            properties,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn poll_rabbitmq_status(connection: &Option<Connection>, channel: &Option<Channel>) -> bool {
    match (connection, channel) {
        (Some(conn), Some(chan)) => conn.status().connected() && chan.status().connected(),
        _ => false,
    }
}

async fn run_rabbitmq_publisher_task(
    mut message_receiver: Receiver<SensorReading>,
    config: AppConfig,
) {
    println!("[Publisher Task] Starting (No Queue Declare, Ignore Publish Fail)...");
    let mut connection: Option<Connection> = None;
    let mut channel: Option<Channel> = None;
    let mut last_connection_attempt = Instant::now();
    let connection_retry_delay = Duration::from_secs(10);

    loop {
        if !poll_rabbitmq_status(&connection, &channel)
            && last_connection_attempt.elapsed() > connection_retry_delay
        {
            println!("[Publisher Task] Background check: Connection/Channel not ready. Attempting reconnect...");
            last_connection_attempt = Instant::now();

            if let Some(ch) = channel.take() { let _ = ch.close(200, "Reconnect Attempt").await; }
            if let Some(conn) = connection.take() { let _ = conn.close(200, "Reconnect Attempt").await; }

            match connect_to_rabbitmq(&config.amqp_addr).await {
                Ok(new_conn) => {
                    match create_rabbitmq_channel(&new_conn).await {
                        Ok(new_channel) => {
                            println!("[Publisher Task] Reconnect: Success! Connection & Channel OK.");
                            connection = Some(new_conn);
                            channel = Some(new_channel);
                        }
                        Err(_) => {
                            println!("[Publisher Task] Reconnect: Failed channel creation.");
                            let _ = new_conn.close(200, "Channel creation failed").await;
                        }
                    }
                }
                Err(_) => {
                    println!("[Publisher Task] Reconnect: Failed connection attempt.");
                }
            }
        }

        tokio::select! {
            biased;
            maybe_reading = message_receiver.recv() => {
                 match maybe_reading {
                     Some(reading) => {
                         let sensor_id_for_log = reading.sensor_id;

                         if poll_rabbitmq_status(&connection, &channel) {
                             match serialize_reading_to_json(&reading) {
                                 Ok(json_payload) => {
                                      let current_channel = channel.as_ref().expect("Internal error: Poll succeeded but channel is None");

                                      let _ = publish_message_to_rabbitmq(
                                          current_channel,
                                          &config.amqp_queue,
                                          &json_payload,
                                      ).await;
                                 }
                                 Err(e) => {
                                     eprintln!("[Publisher Task] JSON serialization failed (Skipped): {}", e);
                                 }
                             }
                         } else {
                             println!("[Publisher Task] Connection poll FAILED. Printing fallback to console.");
                              match serialize_reading_to_json(&reading) {
                                 Ok(json_payload) => {
                                     println!("Fallback Output (Sensor: {}): {}", sensor_id_for_log, json_payload);
                                 }
                                 Err(e) => {
                                     eprintln!("[Publisher Task] Failed to serialize for console fallback (Skipped): {}", e);
                                 }
                             }
                         }
                     }
                     None => {
                         println!("[Publisher Task] Input channel closed. Shutting down.");
                         if let Some(ch) = channel.take() { let _ = ch.close(200, "Normal Exit").await; }
                         if let Some(conn) = connection.take() { let _ = conn.close(200, "Normal Exit").await; }
                         return;
                     }
                 }
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}

pub fn spawn_publisher_task(
    message_receiver: Receiver<SensorReading>,
    config: AppConfig,
) -> JoinHandle<()> {
    println!("[System] Spawning the RabbitMQ publisher task...");
    let handle = tokio::spawn(async move {
        run_rabbitmq_publisher_task(message_receiver, config).await;
    });
    println!("[System] Publisher task spawned.");
    handle
}
use crate::config::{Sensor, SensorType};
use chrono::{DateTime, Utc};
use rand::{rng as ThreadRng, Rng};
use std::f64::consts::PI;
use serde::Serialize;
use tokio::sync::mpsc::{Sender};
use tokio::task::JoinHandle;
use tokio::time::{interval, Instant};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SensorReading {
    pub sensor_id: Uuid,
    pub sensor_type: String,
    pub value: f64,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
}

/// Generate a random sensor value for the sensor and the time
fn generate_sensor_value(sensor: &Sensor, elapsed_secs: f64) -> f64 {
    let mut rng = ThreadRng();
    let noise_val = rng.random_range(-sensor.noise_range..sensor.noise_range);
    let sine_val = sensor.amplitude * (2.0 * PI * elapsed_secs / sensor.period_secs).sin();
    let raw_value = sensor.base_value + sine_val + noise_val;
    match sensor.sensor_type {
        SensorType::Temperature => raw_value.clamp(-50.0, 100.0),
        SensorType::Humidity => raw_value.clamp(0.0, 100.0),
        SensorType::Pressure => raw_value.clamp(800.0, 1200.0),
    }
}

/// Creates a SensorReading structure for the current time and value.
/// Internal helper function for this module.
fn create_sensor_reading(sensor: &Sensor, current_value: f64) -> SensorReading {
    SensorReading {
        sensor_id: sensor.id,
        sensor_type: sensor.sensor_type.to_string(), // Use Display impl from config
        value: current_value,
        timestamp: Utc::now(),
    }
}

/// Sends a single sensor reading through the MPSC channel.
/// Internal helper function.
async fn send_reading_to_channel(
    reading: SensorReading,
    tx: &Sender<SensorReading>,
) -> Result<(), String> {
    if let Err(e) = tx.send(reading).await {
        let error_msg = format!("Failed to send reading to output channel: {}", e);
        Err(error_msg)
    } else {
        Ok(())
    }
}

/// The core async task function for simulating a single sensor.
/// This runs independently for each sensor instance.
async fn run_single_sensor_simulation_task(
    sensor_config: Sensor,
    output_sender: Sender<SensorReading>, // Renamed for clarity
    simulation_start_time: Instant,
) {
    println!("[Sensor Task {}] Starting simulation.", sensor_config.id);
    let mut tick_interval = interval(sensor_config.sampling_rate);
    tick_interval.tick().await; // Consume the initial immediate tick

    loop {
        tick_interval.tick().await;
        let elapsed_seconds = simulation_start_time.elapsed().as_secs_f64();
        let current_value = generate_sensor_value(&sensor_config, elapsed_seconds);
        let reading_to_send = create_sensor_reading(&sensor_config, current_value);
        let sensor_id_for_log = reading_to_send.sensor_id;
        if send_reading_to_channel(reading_to_send, &output_sender)
            .await
            .is_err()
        {
            eprintln!(
                "[Sensor Task {}] Output channel closed. Stopping simulation task.",
                sensor_id_for_log
            );
            break;
        }
    }
    println!("[Sensor Task {}] Simulation task stopped.", sensor_config.id);
}

/// Spawns all the individual sensor simulation tasks based on the provided configurations.
/// Takes ownership of the sender end of the channel to distribute clones.
pub fn spawn_sensor_tasks(
    sensors: Vec<Sensor>,
    channel_sender: Sender<SensorReading>,
    start_time: Instant,
) -> Vec<JoinHandle<()>> {
    println!("Spawning sensor simulation tasks...");
    let mut task_handles = Vec::new();
    for sensor in sensors {
        println!(
            "  Initializing task for Sensor: ID={}, Type={}, Rate={:?}",
            sensor.id, sensor.sensor_type, sensor.sampling_rate
        );
        let sender_clone = channel_sender.clone();
        let sensor_clone = sensor.clone();
        let handle = tokio::spawn(async move {
            run_single_sensor_simulation_task(sensor_clone, sender_clone, start_time).await;
        });
        task_handles.push(handle);
    }
    drop(channel_sender);
    println!("All sensor tasks spawned.");
    task_handles
}

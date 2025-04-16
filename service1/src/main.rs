use std::error::Error;
use tokio::sync::mpsc::{self};
use tokio::time::Instant;
mod config;
mod sensors;
mod publisher;

async fn wait_for_shutdown_signal() {
    println!("Service running. Press Ctrl+C to stop.");
    if let Err(e) = tokio::signal::ctrl_c().await {
        eprintln!("Failed to listen for Ctrl+C signal: {}", e);
    } else {
        println!("\nCtrl+C received. Initiating shutdown...");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting Modular Sensor Simulator Service ---");

    // Read configuration
    let app_config = config::read_configuration();

    // Define sensors
    let sensors_to_simulate = config::define_sensors();
    println!("==================== ::Sensors Configured:: ===================");
    for sensor in &sensors_to_simulate {
        println!("  -> {:?}", sensor);
    }
    println!(" ============================ :::: ============================ ");

    // Init RabbitMQ channel
    publisher::init_channel(app_config).await?;

    // Channel for communication between sensor tasks and publisher task
    let (output_sender, output_receiver) = mpsc::channel::<sensors::SensorReading>(100);

    // Start simulation
    let simulation_start_time = Instant::now();
    let _sensor_task_handles =
        sensors::spawn_sensor_tasks(sensors_to_simulate, output_sender, simulation_start_time);

    // Spawn publisher task
    let _publisher_task_handle = tokio::spawn(async move {
        use tokio_stream::wrappers::ReceiverStream;
        use futures::StreamExt;

        let mut stream = ReceiverStream::new(output_receiver);
        while let Some(reading) = stream.next().await {
            match serde_json::to_string(&reading) {
                Ok(json) => {
                    if let Err(e) = publisher::queue_publish(&json).await {
                        eprintln!("Publish error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Serialization error: {}", e);
                }
            }
        }
    });

    // Wait for shutdown
    wait_for_shutdown_signal().await;

    println!("--- Sensor Simulator Service Shutting Down ---");
    Ok(())
}
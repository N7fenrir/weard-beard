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
    let app_config = config::read_configuration();
    let sensors_to_simulate = config::define_sensors();
    println!("==================== ::Sensors Configured:: ===================");
    for sensor in &sensors_to_simulate { // Iterate using reference
        println!("  -> {:?}", sensor);
    }
    println!(" ============================ :::: ============================ ");
    let (output_sender, output_receiver) = mpsc::channel::<sensors::SensorReading>(100);
    let simulation_start_time = Instant::now();
    let _sensor_task_handles =
        sensors::spawn_sensor_tasks(sensors_to_simulate, output_sender, simulation_start_time);
    let _publisher_task_handle = publisher::spawn_publisher_task(output_receiver, app_config);
    wait_for_shutdown_signal().await;
    println!("--- Sensor Simulator Service Shutting Down ---");
    Ok(())
}
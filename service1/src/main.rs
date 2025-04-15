use std::error::Error;
use tokio::sync::mpsc::{self};
use tokio::time::Instant;
use sensors::print_readings;

mod config;
mod sensors;

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
    println!("--- Starting Core Sensor Simulator Service ---");

    let sensors_to_simulate = config::define_sensors();

    println!("==================== ::Sensors:: ============================ ");
    for sensor in &sensors_to_simulate {
        println!("Sensor Available {:?} ", sensor);
    }
    println!(" ============================ :::: ============================ ");

    let (output_sender, output_receiver) = mpsc::channel::<sensors::SensorReading>(100); // Buffer size 100

    let simulation_start_time = Instant::now();

    let _sensor_task_handles =
        sensors::spawn_sensor_tasks(sensors_to_simulate, output_sender, simulation_start_time);

    let _printer_task_handle = tokio::spawn(async move {
        print_readings(output_receiver).await;
    });

    wait_for_shutdown_signal().await;

    println!("--- Sensor Simulator Service Shutting Down ---");

    Ok(())
}
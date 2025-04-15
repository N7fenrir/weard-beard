use std::error::Error;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Starting Core Sensor Simulator Service ---");

    let sensors_to_simulate = config::define_sensors();

    println!("=================================================");
    for sensor in sensors_to_simulate {
        println!("Sensor Available {:?} ", sensor);
    }
    println!("=================================================");

    println!("--- Sensor Simulator Service Shutting Down ---");

    Ok(())
}
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum SensorType { Temperature, Humidity, Pressure }
impl std::fmt::Display for SensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorType::Temperature => write!(f, "Temperature"),
            SensorType::Humidity => write!(f, "Humidity"),
            SensorType::Pressure => write!(f, "Pressure"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sensor {
    pub id: Uuid,
    pub sensor_type: SensorType,
    pub sampling_rate: Duration,
    pub base_value: f64,
    pub amplitude: f64,
    pub period_secs: f64,
    pub noise_range: f64,
}

/// Defines the set of sensors to be simulated with varying rates.
pub fn define_sensors() -> Vec<Sensor> {
    println!("Defining sensor configurations with varied rates...");
    vec![
        Sensor {
            id: Uuid::new_v4(),
            sensor_type: SensorType::Temperature,
            sampling_rate: Duration::from_millis(300),
            base_value: 20.0, amplitude: 5.0, period_secs: 60.0, noise_range: 0.5,
        },
        Sensor {
            id: Uuid::new_v4(),
            sensor_type: SensorType::Humidity,
            sampling_rate: Duration::from_millis(1500),
            base_value: 55.0, amplitude: 10.0, period_secs: 90.0, noise_range: 1.5,
        },
        Sensor {
            id: Uuid::new_v4(),
            sensor_type: SensorType::Pressure,
            sampling_rate: Duration::from_millis(750),
            base_value: 1013.0, amplitude: 2.0, period_secs: 45.0, noise_range: 0.2,
        },
        Sensor {
            id: Uuid::new_v4(),
            sensor_type: SensorType::Temperature,
            sampling_rate: Duration::from_millis(2100),
            base_value: 22.0, amplitude: 1.0, period_secs: 30.0, noise_range: 0.3,
        }
    ]
}

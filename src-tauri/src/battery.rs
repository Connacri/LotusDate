#[cfg(not(target_os = "android"))]
pub struct BatteryMonitor {
    manager: battery::Manager,
}

#[cfg(target_os = "android")]
pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn new() -> Self {
        #[cfg(not(target_os = "android"))]
        {
            BatteryMonitor {
                manager: battery::Manager::new().unwrap(),
            }
        }
        #[cfg(target_os = "android")]
        {
            BatteryMonitor
        }
    }

    pub fn level(&self) -> f32 {
        #[cfg(not(target_os = "android"))]
        {
            if let Ok(mut batteries) = self.manager.batteries() {
                if let Some(Ok(bat)) = batteries.next() {
                    use battery::units::ratio::percent;
                    return bat.state_of_charge().get::<percent>() / 100.0;
                }
            }
            1.0
        }
        #[cfg(target_os = "android")]
        {
            1.0
        }
    }
}
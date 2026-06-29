use battery::{Manager, State};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BatteryMonitor {
    manager: Manager,
}

impl BatteryMonitor {
    pub fn new() -> Self {
        Self {
            manager: Manager::new().unwrap(),
        }
    }

    pub fn level(&self) -> f32 {
        let mut total = 0.0;
        let mut count = 0;
        for b in self.manager.batteries().unwrap() {
            let bat = b.unwrap();
            total += bat.state_of_charge() as f32;
            count += 1;
        }
        if count > 0 {
            total / count as f32
        } else {
            1.0 // assume full if no battery found (desktop)
        }
    }

    pub fn should_be_full_node(&self) -> bool {
        // Node complet seulement si > 30% ou en charge
        let level = self.level();
        if level > 0.3 {
            return true;
        }
        for b in self.manager.batteries().unwrap() {
            let bat = b.unwrap();
            if bat.state() == State::Charging || bat.state() == State::Full {
                return true;
            }
        }
        false
    }
}
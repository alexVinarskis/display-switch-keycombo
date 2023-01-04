//
// Copyright © 2020 Haim Gelfenbeyn
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use anyhow::{Context, Result};

use crate::configuration::{Configuration, SwitchDirection};
use crate::{display_control};
use crate::key_combination::KeyCombination;
use crate::logging;
use crate::platform::{wake_displays, KeyDetect};
use crate::key;

pub struct App {
    pub config: Configuration,
    pub key_combinations: Vec<(KeyCombination, SwitchDirection)>,
    pressed_keys: Vec<rdev::Key>,
}

impl key::KeyDetectCallback for App {
    fn key_pressed(&mut self, key: &rdev::Key) {
        if !self.pressed_keys.contains(key) {
            self.pressed_keys.push(key.clone());
        }
        self.key_combinations.iter().for_each(|(key_combination, switch_direction)| {
            if key_combination.is_match(&self.pressed_keys) {
                info!("Detected key combination: {:?}", key_combination);
                std::thread::spawn(|| {
                    if let Err(error) = wake_displays() {
                        error!("Error: {:?}", error)
                    };
                });
                display_control::switch(&self.config, switch_direction.clone());
            }
        });
    }

    fn key_released(&mut self, key: &rdev::Key) {
        self.pressed_keys.retain(|value| value != key);
    }
}

impl App {
    pub fn new() -> Result<Self> {
        logging::init_logging().context("failed to initialize logging")?;
        let config = Configuration::load().context("failed to load configuration")?;

        let mut key_combinations: Vec<(KeyCombination, SwitchDirection)> = Vec::new();
        for (potential_key_combination, switch_direction) in [
            (&config.key_combinations.combo_a, SwitchDirection::A),
            (&config.key_combinations.combo_b, SwitchDirection::B),
            (&config.key_combinations.combo_c, SwitchDirection::C),
            (&config.key_combinations.combo_d, SwitchDirection::D)] {
            match potential_key_combination {
                Some(key_combination) => key_combinations.push((key_combination.clone(), switch_direction)),
                None => (),
            }
        }
        debug!("Valid key combinations: {:?}", key_combinations);
        Ok(Self { config, key_combinations, pressed_keys: Vec::new() })
    }

    pub fn run(self) -> Result<()> {
        display_control::log_current_source();
        let key_detector = KeyDetect::new(Box::new(self));
        key_detector.detect();
        Ok(())
    }
}

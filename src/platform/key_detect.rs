//
// Copyright © 2022 Alex Vinarskis
// This code is licensed under MIT license (see LICENSE.txt for details)
//

use rdev::{listen, Event, EventType};
use crate::key::KeyDetectCallback;

pub struct KeyDetect {
    callback: Box<dyn KeyDetectCallback + Send>,
}

impl KeyDetect {
    pub fn new(callback: Box<dyn KeyDetectCallback + Send>) -> Self {
        KeyDetect { callback }
    }

    pub fn detect(mut self) {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => self.callback.key_pressed(&key),
                EventType::KeyRelease(key) => self.callback.key_released(&key),
                _ => {}
            }
        };
        if let Err(error) = listen(callback) {
            error!("Error: {:?}", error)
        };
    }
}

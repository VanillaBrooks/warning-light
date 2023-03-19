use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

use anyhow::Result;

use std::thread;
use std::time::Duration;

const GPIO_RELAY_PIN: u8 = 11;

pub(crate) struct Pi {
    pin: OutputPin,
}

impl Pi {
    pub(crate) fn new() -> Result<Self> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(GPIO_RELAY_PIN)?.into_output();

        Ok(Self { pin })
    }

    pub(crate) fn activate_light(&mut self, duration: Duration) {
        self.pin.set_high();

        thread::sleep(duration);

        self.pin.set_low();
    }
}

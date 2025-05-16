use esp_idf_hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull};

pub struct Button<'a, T: InputPin> {
    pin: PinDriver<'a, T, Input>,
    invert: bool,
}

impl<'a, T: InputPin + OutputPin> Button<'a, T> {
    pub fn new(pin: T, invert: bool) -> anyhow::Result<Self> {
        let mut pin = PinDriver::input(pin)?;
        if invert {
            pin.set_pull(Pull::Down)?;
        } else {
            pin.set_pull(Pull::Up)?;
        }
        Ok(Self { pin, invert })
    }

    pub fn read(&mut self) -> anyhow::Result<bool> {
        let state = self.pin.is_low();
        if self.invert {
            Ok(!state)
        } else {
            Ok(state)
        }
    }
}

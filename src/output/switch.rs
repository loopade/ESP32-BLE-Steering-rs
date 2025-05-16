use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

pub struct Switch<'a, T: OutputPin> {
    pin: PinDriver<'a, T, Output>,
    invert: bool,
}

impl<'a, T: OutputPin> Switch<'a, T> {
    pub fn new(pin: T, invert: bool) -> anyhow::Result<Self> {
        let pin = PinDriver::output(pin)?;
        Ok(Self { pin, invert })
    }

    pub fn on(&mut self) -> anyhow::Result<()> {
        if self.invert {
            self.pin.set_low()?;
        } else {
            self.pin.set_high()?;
        }
        Ok(())
    }

    pub fn off(&mut self) -> anyhow::Result<()> {
        if self.invert {
            self.pin.set_high()?;
        } else {
            self.pin.set_low()?;
        }
        Ok(())
    }
}

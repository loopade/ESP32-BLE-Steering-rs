use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::config::Calibration::Line;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::Resolution::Resolution12Bit;
use esp_idf_hal::gpio::{ADCPin, Input, InputPin, OutputPin, PinDriver, Pull};

pub struct Joystick<'a, X: ADCPin, Y: ADCPin, BTN: InputPin> {
    x_adc: AdcChannelDriver<'a, X, &'a AdcDriver<'a, X::Adc>>,
    y_adc: AdcChannelDriver<'a, Y, &'a AdcDriver<'a, Y::Adc>>,
    button: PinDriver<'a, BTN, Input>,
    deadzone: u16,
    x_mid: u16,
    y_mid: u16,
    x_min: u16,
    y_min: u16,
    x_max: u16,
    y_max: u16,
    output_min: i16,
    output_max: i16,
}

impl<'a, X: ADCPin, Y: ADCPin, BTN: InputPin + OutputPin> Joystick<'a, X, Y, BTN> {
    pub fn new(
        x_adc: &'a AdcDriver<'a, X::Adc>,
        y_adc: &'a AdcDriver<'a, Y::Adc>,
        x_pin: X,
        y_pin: Y,
        button_pin: BTN,
        deadzone: u16,
        output_min: i16,
        output_max: i16,
    ) -> anyhow::Result<Self> {
        let config = AdcChannelConfig {
            attenuation: DB_11,
            resolution: Resolution12Bit,
            calibration: Line,
        };

        let mut x_adc = AdcChannelDriver::new(x_adc, x_pin, &config)?;
        let mut y_adc = AdcChannelDriver::new(y_adc, y_pin, &config)?;
        let mut button = PinDriver::input(button_pin)?;
        button.set_pull(Pull::Up)?;

        // Initialize the joystick's zero position
        let mut x_avg = 0;
        let mut y_avg = 0;
        const N: u16 = 10; // Number of samples to average for initial position
        for _i in 0..N {
            x_avg += x_adc.read()?;
            y_avg += y_adc.read()?;
        }
        let x_mid = x_avg / N;
        let y_mid = y_avg / N;

        Ok(Self {
            x_adc,
            y_adc,
            button,
            deadzone, // Default deadzone value
            x_mid,
            y_mid,
            x_min: 200,  // 0.2 V
            y_min: 200,  // 0.2 V
            x_max: 3100, // 3.1 V
            y_max: 3100, // 3.1 V
            output_min,
            output_max,
        })
    }

    pub fn read(&mut self) -> anyhow::Result<(i16, i16, bool)> {
        let x_val = self.x_adc.read()?;
        let y_val = self.y_adc.read()?;
        let btn_pressed = self.button.is_low();

        let mut x_val = x_val as f32;
        let mut y_val = y_val as f32;
        let x_mid = self.x_mid as f32;
        let y_mid = self.y_mid as f32;
        let x_min = self.x_min as f32;
        let y_min = self.y_min as f32;
        let x_max = self.x_max as f32;
        let y_max = self.y_max as f32;
        let deadzone = self.deadzone as f32;
        let output_min = self.output_min as f32;
        let output_max = self.output_max as f32;
        let output_mid = (output_max + output_min) / 2.0;
        // map the joystick values to a range
        if x_val > x_mid {
            x_val = (x_val - x_mid) / (x_max - x_mid) * (output_max - output_mid) + output_mid;
        } else {
            x_val = (x_val - x_min) / (x_mid - x_min) * (output_mid - output_min) + output_min;
        }
        if y_val > y_mid {
            y_val = (y_val - y_mid) / (y_max - y_mid) * (output_max - output_mid) + output_mid;
        } else {
            y_val = (y_val - y_min) / (y_mid - y_min) * (output_mid - output_min) + output_min;
        }
        // Apply deadzone
        if x_val > output_mid - deadzone && x_val < output_mid + deadzone {
            x_val = output_mid;
        }
        if y_val > output_mid - deadzone && y_val < output_mid + deadzone {
            y_val = output_mid;
        }
        // Clamp the values to the output range
        x_val = x_val.clamp(output_min, output_max);
        y_val = y_val.clamp(output_min, output_max);
        
        Ok((x_val as i16, y_val as i16, btn_pressed))
    }
}

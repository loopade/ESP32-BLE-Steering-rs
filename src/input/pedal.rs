use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::config::Calibration::Line;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::Resolution::Resolution12Bit;
use esp_idf_hal::gpio::ADCPin;
use log::info;

pub struct Pedal<'a, X: ADCPin, Y: ADCPin> {
    accelerator_adc: AdcChannelDriver<'a, X, &'a AdcDriver<'a, X::Adc>>,
    brake_adc: AdcChannelDriver<'a, Y, &'a AdcDriver<'a, Y::Adc>>,
    input_min: u16,
    input_max: u16,
    output_min: i16,
    output_max: i16,
}

impl<'a, X: ADCPin, Y: ADCPin> Pedal<'a, X, Y> {
    pub fn new(
        accelerator_adc: &'a AdcDriver<'a, X::Adc>,
        brake_adc: &'a AdcDriver<'a, Y::Adc>,
        accelerator_pin: X,
        brake_pin: Y,
        deadzone: u16, // Deadzone value in mV
        output_min: i16,
        output_max: i16,
    ) -> anyhow::Result<Self> {
        let config = AdcChannelConfig {
            attenuation: DB_11,
            resolution: Resolution12Bit,
            calibration: Line,
        };

        let accelerator_adc = AdcChannelDriver::new(accelerator_adc, accelerator_pin, &config)?;
        let brake_adc = AdcChannelDriver::new(brake_adc, brake_pin, &config)?;

        Ok(Self {
            accelerator_adc,
            brake_adc,
            input_min: 150 + deadzone, // 0.15V
            input_max: 2450,           // 2.45V
            output_min,
            output_max,
        })
    }

    pub fn read(&mut self) -> anyhow::Result<(i16, i16)> {
        let accelerator_val = self.accelerator_adc.read()?;
        let brake_val = self.brake_adc.read()?;

        let mut accelerator_val = accelerator_val as f32;
        let mut brake_val = brake_val as f32;
        let input_min = self.input_min as f32;
        let input_max = self.input_max as f32;
        let output_min = self.output_min as f32;
        let output_max = self.output_max as f32;

        accelerator_val = accelerator_val.clamp(input_min, input_max);
        brake_val = brake_val.clamp(input_min, input_max);

        // map the Pedal values to a range
        accelerator_val = (accelerator_val - input_min) / (input_max - input_min)
            * (output_max - output_min)
            + output_min;
        brake_val = (brake_val - input_min) / (input_max - input_min) * (output_max - output_min)
            + output_min;

        Ok((accelerator_val as i16, brake_val as i16))
    }
}

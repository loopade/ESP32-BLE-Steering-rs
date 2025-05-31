use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::config::Calibration::Line;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::Resolution::Resolution12Bit;
use esp_idf_hal::gpio::ADCPin;

pub struct Pedal<'a, X: ADCPin, Y: ADCPin> {
    accelerator_adc: AdcChannelDriver<'a, X, &'a AdcDriver<'a, X::Adc>>,
    brake_adc: AdcChannelDriver<'a, Y, &'a AdcDriver<'a, Y::Adc>>,
    deadzone: u16,
    accelerator_min: u16,
    brake_min: u16,
    accelerator_max: u16,
    brake_max: u16,
    output_min: i16,
    output_max: i16,
}

impl<'a, X: ADCPin, Y: ADCPin> Pedal<'a, X, Y> {
    pub fn new(
        accelerator_adc: &'a AdcDriver<'a, X::Adc>,
        brake_adc: &'a AdcDriver<'a, Y::Adc>,
        accelerator_pin: X,
        brake_pin: Y,
        deadzone: u16,
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
            deadzone,              // Default deadzone value
            accelerator_min: 200,  // 0.2 V
            brake_min: 200,        // 0.2 V
            accelerator_max: 2700, // 2.7 V
            brake_max: 2700,       // 2.7 V
            output_min,
            output_max,
        })
    }

    pub fn read(&mut self) -> anyhow::Result<(i16, i16)> {
        let accelerator_val = self.accelerator_adc.read()?;
        let brake_val = self.brake_adc.read()?;

        let mut accelerator_val = accelerator_val as f32;
        let mut brake_val = brake_val as f32;
        let accelerator_min = self.accelerator_min as f32;
        let brake_min = self.brake_min as f32;
        let accelerator_max = self.accelerator_max as f32;
        let brake_max = self.brake_max as f32;
        let deadzone = self.deadzone as f32;
        let output_min = self.output_min as f32;
        let output_max = self.output_max as f32;
        // map the Pedal values to a range
        accelerator_val = (accelerator_val - accelerator_min) / (accelerator_max - accelerator_min)
            * (output_max - output_min)
            + output_min;
        brake_val = (brake_val - brake_min) / (brake_max - brake_min) * (output_max - output_min)
            + output_min;

        // Apply deadzone
        if accelerator_val < deadzone {
            accelerator_val = output_min;
        }
        if brake_val < deadzone {
            brake_val = output_min;
        }

        // Clamp the values to the output range
        accelerator_val = accelerator_val.clamp(output_min, output_max);
        brake_val = brake_val.clamp(output_min, output_max);

        Ok((accelerator_val as i16, brake_val as i16))
    }
}

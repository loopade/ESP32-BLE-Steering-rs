use esp_idf_hal::delay::Delay;
use esp_idf_hal::i2c::I2cDriver;
use log::warn;
use mpu9250::DMP_FIRMWARE;
use mpu9250::{Dmp, I2cDevice, Mpu9250};
use std::f64::consts::PI;

fn quaternion_to_roll(q: [f64; 4], roll: f64) -> f64 {
    // atan2(2.0f * (q[0] * q[1] + q[2] * q[3]),
    // q[0] * q[0] - q[1] * q[1] - q[2] * q[2] + q[3] * q[3])
    let mut new_roll = (2.0 * (q[0] * q[1] + q[2] * q[3]))
        .atan2(q[0] * q[0] - q[1] * q[1] - q[2] * q[2] + q[3] * q[3]);
    new_roll = new_roll * 180.0 / PI - 90.0;
    if new_roll < -180.0 {
        new_roll += 360.0;
    }
    // newRoll  ∈ [-180, 180]
    // 比较新旧角度，如果差值大于180度，说明是跨越了0度，需要做修正
    if (new_roll - roll).abs() > 180.0 {
        if new_roll > roll {
            new_roll -= 360.0;
        } else {
            new_roll += 360.0;
        }
    }

    new_roll
}

pub struct MpuSensor<'a> {
    mpu: Option<Mpu9250<I2cDevice<I2cDriver<'a>>, Dmp>>,
    roll: f64,
}

impl<'a> MpuSensor<'a> {
    pub fn new(i2c: I2cDriver<'a>) -> anyhow::Result<Self> {
        let mut delay = Delay::new_default();
        let mpu =
            Mpu9250::dmp_default(i2c, &mut delay, &DMP_FIRMWARE).ok();
        Ok(Self { mpu, roll: 0.0 })
    }

    pub fn roll(&mut self) -> Option<f64> {
        let mpu = match self.mpu {
            Some(ref mut mpu) => mpu,
            None => {
                warn!("MPU sensor not initialized");
                return None;
            }
        };
        let all = match mpu.dmp_all::<[f32; 3], [f64; 4]>() {
            Ok(all) => all,
            Err(e) => {
                warn!("Failed to read DMP data: {:?}", e);
                return None;
            }
        };
        match all.quaternion {
            Some(q) => {
                self.roll = quaternion_to_roll(q, self.roll);
                Some(self.roll)
            }
            None => {
                warn!("No quaternion data available");
                None
            }
        }
    }
}

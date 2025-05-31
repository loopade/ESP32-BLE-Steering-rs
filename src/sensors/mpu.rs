use esp_idf_hal::delay::Delay;
use esp_idf_hal::i2c::I2cDriver;
use log::warn;
use mpu9250::{I2cDevice, Imu, Mpu9250};
use std::f32::consts::PI;
use std::time::Instant;

fn quaternion_to_roll(q: [f32; 4], roll: f32) -> f32 {
    // atan2(2.0f * (q[0] * q[1] + q[2] * q[3]),
    // q[0] * q[0] - q[1] * q[1] - q[2] * q[2] + q[3] * q[3])
    let mut new_roll = (-2.0 * (q[0] * q[1] + q[2] * q[3]))
        .atan2(q[0] * q[0] - q[1] * q[1] - q[2] * q[2] + q[3] * q[3]);
    new_roll = new_roll * 180.0 / PI - 90.0;
    if new_roll < -180.0 {
        new_roll += 360.0;
    }

    let mut delta = new_roll - roll;
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta < -180.0 {
        delta += 360.0;
    }

    roll + delta
}

pub struct MpuSensor<'a> {
    mpu: Option<Mpu9250<I2cDevice<I2cDriver<'a>>, Imu>>,
    roll: f32,
    q: [f32; 4],
    gbias: [f32; 3],
    beta: f32,
    zeta: f32,
    updated: Instant,
}

impl<'a> MpuSensor<'a> {
    pub fn new(i2c: I2cDriver<'a>) -> anyhow::Result<Self> {
        let mut delay = Delay::new_default();
        let mpu = match Mpu9250::imu_default(i2c, &mut delay) {
            Ok(mpu) => Some(mpu),
            Err(e) => {
                warn!("Failed to initialize MPU sensor: {:?}", e);
                None
            }
        };

        let gyro_meas_error = PI * (40.0 / 180.0);
        let beta = (3.0 / 4.0_f32).sqrt() * gyro_meas_error;
        let gyro_meas_drift = PI * (2.0 / 180.0);
        let zeta = (3.0 / 4.0_f32).sqrt() * gyro_meas_drift;
        let updated = Instant::now();

        Ok(Self {
            mpu,
            roll: 0.0,
            q: [1.0, 0.0, 0.0, 0.0],
            gbias: [0.0, 0.0, 0.0],
            beta,
            zeta,
            updated,
        })
    }

    pub fn roll(&mut self) -> Option<f32> {
        let mpu = match self.mpu {
            Some(ref mut mpu) => mpu,
            None => {
                warn!("MPU sensor not initialized");
                return None;
            }
        };
        let all = match mpu.all::<[f32; 3]>() {
            Ok(all) => all,
            Err(e) => {
                warn!("Failed to read DMP data: {:?}", e);
                return None;
            }
        };
        self.madgwick_quaternion_update(
            all.accel[0],
            all.accel[1],
            all.accel[2],
            all.gyro[0],
            all.gyro[1],
            all.gyro[2],
        );

        self.updated = Instant::now();

        self.roll = quaternion_to_roll(self.q.map(|x| x), self.roll);
        Some(self.roll)
    }

    pub fn madgwick_quaternion_update(
        &mut self,
        ax: f32,
        ay: f32,
        az: f32,
        gyrox: f32,
        gyroy: f32,
        gyroz: f32,
    ) {
        let delta_t = self.updated.elapsed().as_secs_f32();

        let &[mut q1, mut q2, mut q3, mut q4] = &self.q;
        let [ref mut gbiasx, ref mut gbiasy, ref mut gbiasz] = &mut self.gbias;

        let half_q1 = 0.5 * q1;
        let half_q2 = 0.5 * q2;
        let half_q3 = 0.5 * q3;
        let half_q4 = 0.5 * q4;
        let two_q1 = 2.0 * q1;
        let two_q2 = 2.0 * q2;
        let two_q3 = 2.0 * q3;
        let two_q4 = 2.0 * q4;

        let norm_acc = (ax * ax + ay * ay + az * az).sqrt();
        if norm_acc == 0.0 {
            return; // 防止除零错误
        }
        let inv_norm = 1.0 / norm_acc;
        let ax = ax * inv_norm;
        let ay = ay * inv_norm;
        let az = az * inv_norm;

        // 计算目标函数
        let f1 = two_q2 * q4 - two_q1 * q3 - ax;
        let f2 = two_q1 * q2 + two_q3 * q4 - ay;
        let f3 = 1.0 - two_q2 * q2 - two_q3 * q3 - az;

        // 计算雅可比矩阵元素
        let j_11or24 = two_q3;
        let j_12or23 = two_q4;
        let j_13or22 = two_q1;
        let j_14or21 = two_q2;
        let j_32 = 2.0 * j_14or21;
        let j_33 = 2.0 * j_11or24;

        // 计算梯度向量 (∇f · J)
        let mut hat_dot1 = j_14or21 * f2 - j_11or24 * f1;
        let mut hat_dot2 = j_12or23 * f1 + j_13or22 * f2 - j_32 * f3;
        let mut hat_dot3 = j_12or23 * f2 - j_33 * f3 - j_13or22 * f1;
        let mut hat_dot4 = j_14or21 * f1 + j_11or24 * f2;

        // 归一化梯度
        let norm_grad =
            (hat_dot1 * hat_dot1 + hat_dot2 * hat_dot2 + hat_dot3 * hat_dot3 + hat_dot4 * hat_dot4)
                .sqrt();

        if norm_grad > 0.0 {
            let inv_norm_grad = 1.0 / norm_grad;
            hat_dot1 *= inv_norm_grad;
            hat_dot2 *= inv_norm_grad;
            hat_dot3 *= inv_norm_grad;
            hat_dot4 *= inv_norm_grad;
        }

        // 计算陀螺仪偏置误差
        let gerrx = two_q1 * hat_dot2 - two_q2 * hat_dot1 - two_q3 * hat_dot4 + two_q4 * hat_dot3;
        let gerry = two_q1 * hat_dot3 + two_q2 * hat_dot4 - two_q3 * hat_dot1 - two_q4 * hat_dot2;
        let gerrz = two_q1 * hat_dot4 - two_q2 * hat_dot3 + two_q3 * hat_dot2 - two_q4 * hat_dot1;

        // 更新陀螺仪偏置
        *gbiasx += gerrx * delta_t * self.zeta;
        *gbiasy += gerry * delta_t * self.zeta;
        *gbiasz += gerrz * delta_t * self.zeta;

        // 应用偏置补偿
        let gyrox = gyrox - *gbiasx;
        let gyroy = gyroy - *gbiasy;
        let gyroz = gyroz - *gbiasz;

        // 计算四元数导数
        let q_dot1 = -half_q2 * gyrox - half_q3 * gyroy - half_q4 * gyroz;
        let q_dot2 = half_q1 * gyrox + half_q3 * gyroz - half_q4 * gyroy;
        let q_dot3 = half_q1 * gyroy - half_q2 * gyroz + half_q4 * gyrox;
        let q_dot4 = half_q1 * gyroz + half_q2 * gyroy - half_q3 * gyrox;

        // 应用梯度下降并积分
        q1 += (q_dot1 - (self.beta * hat_dot1)) * delta_t;
        q2 += (q_dot2 - (self.beta * hat_dot2)) * delta_t;
        q3 += (q_dot3 - (self.beta * hat_dot3)) * delta_t;
        q4 += (q_dot4 - (self.beta * hat_dot4)) * delta_t;

        // 归一化最终四元数
        let norm_quat = (q1 * q1 + q2 * q2 + q3 * q3 + q4 * q4).sqrt();
        if norm_quat > 0.0 {
            let inv_norm_quat = 1.0 / norm_quat;
            self.q = [
                q1 * inv_norm_quat,
                q2 * inv_norm_quat,
                q3 * inv_norm_quat,
                q4 * inv_norm_quat,
            ];
        }
    }
}

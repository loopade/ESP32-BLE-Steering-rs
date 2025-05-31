use esp_idf_hal::adc::oneshot::AdcDriver;
use esp_idf_hal::gpio::{IOPin, OutputPin, PinDriver};
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_hal::timer::{TimerConfig, TimerDriver};
use esp_idf_hal::units::Hertz;
use futures::join;
use log::{info, warn};

mod sensors;
use sensors::MpuSensor;

mod input;
use input::Button;
use input::Joystick;
use input::Keypad;
use input::Pedal;

mod output;
use output::Switch;

mod ble;
use ble::Steering;

const AX_MAX: i16 = 32767;
const AX_MIN: i16 = -32767;
const SM_MAX: i16 = 32767;
const SM_MIN: i16 = 0;
const STEERING_ROTATION_ANGLE: f32 = 900.0;

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let mut timer00 = TimerDriver::new(peripherals.timer00, &TimerConfig::new())?;
    let mut timer01 = TimerDriver::new(peripherals.timer01, &TimerConfig::new())?;
    let mut timer10 = TimerDriver::new(peripherals.timer10, &TimerConfig::new())?;
    let timer11 = TimerDriver::new(peripherals.timer11, &TimerConfig::new())?;

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;

    let i2c_config = I2cConfig::new().baudrate(Hertz(400_000));
    let i2c = I2cDriver::new(i2c, sda, scl, &i2c_config)?;
    let mut mpu = MpuSensor::new(i2c)?;

    let mut led = Switch::new(peripherals.pins.gpio2, false)?;
    let mut motor = Switch::new(peripherals.pins.gpio15, false)?;

    let mut gear_drive = Button::new(peripherals.pins.gpio18, false)?;
    let mut gear_reverse = Button::new(peripherals.pins.gpio19, false)?;

    let adc = AdcDriver::new(peripherals.adc1)?;

    let mut joystick = match Joystick::new(
        &adc,
        &adc,
        peripherals.pins.gpio34,
        peripherals.pins.gpio35,
        peripherals.pins.gpio12,
        1600,
        AX_MIN,
        AX_MAX,
    ) {
        Ok(joystick) => {
            info!("Joystick initialized successfully");
            joystick
        }
        Err(e) => {
            warn!("Failed to initialize joystick: {:?}", e);
            return Err(e);
        }
    };

    let mut pedal = match Pedal::new(
        &adc,
        &adc,
        peripherals.pins.gpio32,
        peripherals.pins.gpio33,
        2000,
        SM_MIN,
        SM_MAX,
    ) {
        Ok(pedal) => {
            info!("Pedal initialized successfully");
            pedal
        }
        Err(e) => {
            warn!("Failed to initialize pedal: {:?}", e);
            return Err(e);
        }
    };

    led.off()?;
    motor.off()?;

    let ms00 = timer00.tick_hz() / 1000;
    let ms01 = timer01.tick_hz() / 1000;
    let ms10 = timer10.tick_hz() / 1000;

    // row: 26 27 14 12
    // col: 4 16 17 5
    let mut keypad = match Keypad::new(
        [
            PinDriver::input(peripherals.pins.gpio4.downgrade())?,
            PinDriver::input(peripherals.pins.gpio16.downgrade())?,
            PinDriver::input(peripherals.pins.gpio17.downgrade())?,
            PinDriver::input(peripherals.pins.gpio5.downgrade())?,
        ],
        [
            PinDriver::output(peripherals.pins.gpio25.downgrade_output())?,
            PinDriver::output(peripherals.pins.gpio26.downgrade_output())?,
            PinDriver::output(peripherals.pins.gpio27.downgrade_output())?,
            PinDriver::output(peripherals.pins.gpio14.downgrade_output())?,
        ],
        timer11,
    ) {
        Ok(keypad) => {
            info!("Keypad initialized successfully");
            keypad
        }
        Err(e) => {
            warn!("Failed to initialize keypad: {:?}", e);
            return Err(e);
        }
    };

    let ble_steering = match Steering::new() {
        Ok(steering) => {
            info!("BLE steering initialized successfully");
            steering
        }
        Err(e) => {
            warn!("Failed to initialize BLE steering: {:?}", e);
            return Err(e);
        }
    };

    block_on(async {
        join!(
            async {
                loop {
                    timer00.delay(10 * ms00).await.expect("Timer delay failed");
                    match mpu.roll() {
                        Some(roll) => {
                            let roll = roll.clamp(-STEERING_ROTATION_ANGLE / 2.0, STEERING_ROTATION_ANGLE / 2.0);
                            let report_ratio =
                                (SM_MAX - SM_MIN) as f32 / STEERING_ROTATION_ANGLE;
                            let roll = (roll + STEERING_ROTATION_ANGLE / 2.0) * report_ratio;
                            ble_steering.set_steering(roll as i16);
                        }
                        None => {}
                    }
                }
            },
            async {
                loop {
                    if ble_steering.connected() {
                        ble_steering.send_report();
                        timer10.delay(7 * ms10).await.expect("Timer delay failed");
                    } else {
                        let _ = led.off();
                        timer10.delay(500 * ms10).await.expect("Timer delay failed");
                        let _ = led.on();
                        timer10.delay(500 * ms10).await.expect("Timer delay failed");
                    }
                }
            },
            async {
                loop {
                    let mut states: u32 = 0;
                    match keypad.scan(5).await {
                        Ok(_) => {
                            states |= keypad.states() as u32;
                        }
                        Err(e) => {
                            warn!("Error scanning keypad: {:?}", e);
                        }
                    }
                    match joystick.read() {
                        Ok((x, y, pressed)) => {
                            ble_steering.set_axes(x, y);
                            if pressed {
                                states |= 1 << 16; // Button pressed
                            } else {
                                states &= !(1 << 16); // Button released
                            }
                        }
                        Err(e) => {
                            warn!("Error reading joystick: {:?}", e);
                        }
                    }
                    match pedal.read() {
                        Ok((accelerator, brake)) => {
                            ble_steering.set_pedals(accelerator, brake);
                        }
                        Err(e) => {
                            warn!("Error reading pedal: {:?}", e);
                        }
                    }
                    match gear_drive.read() {
                        Ok(pressed) => {
                            if pressed {
                                states |= 1 << 17; // Gear drive pressed
                            } else {
                                states &= !(1 << 17); // Gear drive released
                            }
                        }
                        Err(e) => {
                            warn!("Error reading gear drive: {:?}", e);
                        }
                    }
                    match gear_reverse.read() {
                        Ok(pressed) => {
                            if pressed {
                                states |= 1 << 18; // Gear reverse pressed
                            } else {
                                states &= !(1 << 18); // Gear reverse released
                            }
                        }
                        Err(e) => {
                            warn!("Error reading gear reverse: {:?}", e);
                        }
                    }
                    ble_steering.set_buttons(states);
                    timer01.delay(5 * ms01).await.expect("Timer delay failed");
                }
            }
        );
        Ok(())
    })
}

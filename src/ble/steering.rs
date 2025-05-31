#![allow(dead_code)]

use esp32_nimble::{
    enums::*, hid::*, utilities::mutex::Mutex, BLEAdvertisementData, BLECharacteristic, BLEDevice,
    BLEHIDDevice, BLEServer,
};
use log::info;
use std::sync::Arc;
use zerocopy::IntoBytes;
use zerocopy_derive::{Immutable, IntoBytes};

const STEERING_ID: u8 = 0x03;

const HID_REPORT_DESCRIPTOR: &[u8] = hid!(
    (USAGE_PAGE, 0x01),       // Generic Desktop
    (USAGE, 0x05),            // Gamepad
    (COLLECTION, 0x01),       // Application
    (REPORT_ID, STEERING_ID), // Report ID 3
    // ----------------------------------- Buttons
    (USAGE_PAGE, 0x09),      // Button
    (LOGICAL_MINIMUM, 0x00), // 0
    (LOGICAL_MAXIMUM, 0x01), // 1
    (REPORT_SIZE, 1),
    (USAGE_MINIMUM, 0x01), // Button 1
    (USAGE_MAXIMUM, 32),   // Button 32
    (REPORT_COUNT, 32),    // 32 buttons
    (HIDINPUT, 0x02),      // INPUT (Data,Var,Abs)
    // ------------------------------------ Steerings
    (USAGE_PAGE, 0x02),            // Simulation Controls
    (LOGICAL_MINIMUM, 0x00, 0x00), // -32767
    (LOGICAL_MAXIMUM, 0xFF, 0x7F), // 32767
    (REPORT_SIZE, 16),
    (REPORT_COUNT, 3),  // 2 axes
    (COLLECTION, 0x00), // Physical
    (USAGE, 0xC8),      // Steering
    (USAGE, 0xC4),      // Accelerator
    (USAGE, 0xC5),      // Brake
    (HIDINPUT, 0x02),   // INPUT (Data,Var,Abs)
    (END_COLLECTION),   // Physical(End)
    // ----------------------------------- Axes
    (USAGE_PAGE, 0x01),            // Generic Desktop
    (LOGICAL_MINIMUM, 0x01, 0x80), // -32767
    (LOGICAL_MAXIMUM, 0xFF, 0x7F), // 32767
    (REPORT_SIZE, 16),
    (REPORT_COUNT, 2),  // 2 axes
    (COLLECTION, 0x00), // Physical
    (USAGE, 0x30),      // X
    (USAGE, 0x31),      // Y
    (HIDINPUT, 0x02),   // INPUT (Data,Var,Abs)
    (END_COLLECTION),   // Physical(End)
    // ------------------------------------ Application(End)
    (END_COLLECTION)
);
#[derive(IntoBytes, Immutable, Debug)]
#[repr(packed)]
struct SteeringReport {
    buttons: u32,
    steering: i16,
    accelerator: i16,
    brake: i16,
    x: i16,
    y: i16,
}

pub struct Steering {
    server: &'static mut BLEServer,
    input_steering: Arc<Mutex<BLECharacteristic>>,
    steering_report: Arc<Mutex<SteeringReport>>,
}

impl Steering {
    pub fn new() -> anyhow::Result<Self> {
        let device = BLEDevice::take();
        device
            .security()
            .set_auth(AuthReq::all())
            .set_io_cap(SecurityIOCap::NoInputNoOutput)
            .resolve_rpa();

        let server = device.get_server();
        let mut hid = BLEHIDDevice::new(server);

        let input_steering = hid.input_report(STEERING_ID);

        hid.manufacturer("Baohuiming.net");
        hid.pnp(0x02, 0x2838, 0x0100, 0x0525);
        hid.hid_info(0x00, 0x01);

        hid.report_map(HID_REPORT_DESCRIPTOR);

        hid.set_battery_level(100);

        let ble_advertising = device.get_advertising();
        ble_advertising.lock().scan_response(false).set_data(
            BLEAdvertisementData::new()
                .name("ESP32 Gamepad R1")
                .appearance(0x03C1)
                .add_service_uuid(hid.hid_service().lock().uuid()),
        )?;
        ble_advertising.lock().start()?;

        let steering_report = Arc::new(Mutex::new(SteeringReport {
            buttons: 0,
            x: 0,
            y: 0,
            accelerator: 0,
            brake: 0,
            steering: 0,
        }));

        Ok(Self {
            server,
            input_steering,
            steering_report,
        })
    }

    pub fn connected(&self) -> bool {
        self.server.connected_count() > 0
    }

    pub fn set_steering(&self, value: i16) {
        let mut report = self.steering_report.lock();
        report.steering = value;
    }

    pub fn set_pedals(&self, accelerator_value: i16, brake_value: i16) {
        let mut report = self.steering_report.lock();
        report.accelerator = accelerator_value;
        report.brake = brake_value;
    }

    pub fn set_axes(&self, x_value: i16, y_value: i16) {
        let mut report = self.steering_report.lock();
        report.x = x_value;
        report.y = y_value;
    }

    pub fn set_buttons(&self, buttons: u32) {
        let mut report = self.steering_report.lock();
        report.buttons = buttons
    }

    pub fn send_report(&self) {
        // SteeringReport { buttons: 1, x: 2047, y: 2047, accelerator: 0, brake: 0, steering: 0 }
        // [1, 0, 255, 7, 255, 7, 0, 0, 0, 0, 0, 0]
        let report = self.steering_report.lock();
        let report_bytes = report.as_bytes();
        // info!("Sending steering report: {:?}", report_bytes);
        self.input_steering.lock().set_value(&report_bytes).notify();
    }
}

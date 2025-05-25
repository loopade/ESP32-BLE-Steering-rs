 # ESP32-BLE-Steering-rs
自制ESP32蓝牙方向盘

从[Arduino版本](https://github.com/loopade/ESP32-BLE-Steering)
移植而来，后续将只更新rust版本。

DIY ESP32 Bluetooth Steering

This is a port from the [Arduino version](
https://github.com/loopade/ESP32-BLE-Steering). 
Future updates will only be in Rust.


## 准备工作
* ESP32(D/E)开发板，足够多的GPIO、ADC接口
* MPU6500/9250/9255陀螺仪
* 若干电位器（摇杆、踏板）
* 按钮、拨动开关
* 小型振动电机

## 管脚图
![ESP32-devkitC-V4](https://docs.espressif.com/projects/esp-idf/en/v5.1/esp32/_images/esp32-devkitC-v4-pinout.png "ESP32-devkitC-V4")

具体接线见pinouts目录下的文件：

[DevKit-C](
https://github.com/loopade/ESP32-BLE-Steering-rs/blob/master/pinouts/DevKit-C.csv
)

[DevKit-V1](
https://github.com/loopade/ESP32-BLE-Steering-rs/blob/master/pinouts/DevKit-V1.csv
)

## ❗注意事项：
* 摇杆和踏板需接在3.3v
* 陀螺仪需接在5v

<hr/>

## Previous work
* ESP32(D/E)-Devkit (Enough GPIO/ADC pins)
* MPU6500/9250/9255 gyroscope
* Some potentiometer (joystick and pedal)
* Buttons and switches
* Small vibration motor

## Pinout
![ESP32-devkitC-V4](https://docs.espressif.com/projects/esp-idf/en/v5.1/esp32/_images/esp32-devkitC-v4-pinout.png "ESP32-devkitC-V4")

More details can be found in the pinouts directory:

[DevKit-C](
https://github.com/loopade/ESP32-BLE-Steering-rs/blob/master/pinouts/DevKit-C.csv
)

[DevKit-V1](
https://github.com/loopade/ESP32-BLE-Steering-rs/blob/master/pinouts/DevKit-V1.csv
)

## ❗Attention
* Connect joystick and pedal to +3.3v
* Connect gyroscope to +5v

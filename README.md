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

| 功能                | 引脚 | 引脚 | 功能                              |
|---------------------|------|------|-----------------------------------|
| 外接VCC(摇杆, 踏板) | 3V3  | GND  | 外接GND(踏板, 电机, 陀螺仪, 排挡) |
|                     | EN   | 23   |                                   |
|                     | VP   | 22   | 陀螺仪SCL                         |
|                     | VN   | TX   |                                   |
| 摇杆X轴             | 34   | RX   |                                   |
| 摇杆Y轴             | 35   | 21   | 陀螺仪SDA                         |
| 油门踏板            | 32   | GND  | 陀螺仪GND                         |
| 刹车踏板            | 33   | 19   | 排挡后退挡                        |
| 摇杆按钮            | 25   | 18   | 排挡前进挡                        |
| 键盘行4             | 26   | 5    | 键盘列4                           |
| 键盘行3             | 27   | 17   | 键盘列3                           |
| 键盘行2             | 14   | 16   | 键盘列2                           |
| 键盘行1             | 12   | 4    | 键盘列1                           |
|                     | GND  | 0    |                                   |
|                     | 13   | 2    | LED                               |
|                     | D2   | 15   | 电机VCC                           |
|                     | D3   | D1   |                                   |
|                     | CMD  | D0   |                                   |
| 外接VCC(陀螺仪VCC)  | 5V   | CLK  |                                   |

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

| Function                       | Pin | Pin | Function                                     |
|--------------------------------|-----|-----|----------------------------------------------|
| External VCC (joystick, pedal) | 3V3 | GND | External GND (pedal, motor, gyroscope, gear) |
|                                | EN  | 23  |                                              |
|                                | VP  | 22  | Gyroscope SCL                                |
|                                | VN  | TX  |                                              |
| Joystick X-axis                | 34  | RX  |                                              |
| Joystick Y-axis                | 35  | 21  | Gyroscope SDA                                |
| Accelerator pedal              | 32  | GND | Gyroscope GND                                |
| Brake pedal                    | 33  | 19  | Gear reverse                                 |
| Joystick button                | 25  | 18  | Gear forward                                 |
| Keyboard Row 4                 | 26  | 5   | Keyboard Column 4                            |
| Keyboard Row 3                 | 27  | 17  | Keyboard Column 3                            |
| Keyboard Row 2                 | 14  | 16  | Keyboard Column 2                            |
| Keyboard Row 1                 | 12  | 4   | Keyboard Column 1                            |
|                                | GND | 0   |                                              |
|                                | 13  | 2   | LED                                          |
|                                | D2  | 15  | Motor VCC                                    |
|                                | D3  | D1  |                                              |
|                                | CMD | D0  |                                              |
| External VCC (gyroscope VCC)   | 5V  | CLK |                                              |

## ❗Attention
* Connect joystick and pedal to +3.3v
* Connect gyroscope to +5v

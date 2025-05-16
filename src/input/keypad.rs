use bitflags::bitflags;
use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Input, Output, PinDriver, Pull};
use esp_idf_hal::timer::TimerDriver;
// use std::collections::VecDeque;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct KeyState: u16 {
        const PRESSED     = 1 << 0;
        const JUST_PRESS  = 1 << 1;
        const JUST_RELEASE= 1 << 2;
    }
}

/// Describes the hardware-level matrix of switches.
///
/// Generic parameters are in order: The type of column pins,
/// the type of row pins, the number of columns and rows.
/// **NOTE:** In order to be able to put different pin structs
/// in an array they have to be downgraded (stripped of their
/// numbers etc.). Most HAL-s have a method of downgrading pins
/// to a common (erased) struct. (for example see
/// [stm32f0xx_hal::gpio::PA0::downgrade](https://docs.rs/stm32f0xx-hal/0.17.1/stm32f0xx_hal/gpio/gpioa/struct.PA0.html#method.downgrade))
pub struct Keypad<'a, const CS: usize, const RS: usize> {
    cols: [PinDriver<'a, AnyIOPin, Input>; CS],
    rows: [PinDriver<'a, AnyOutputPin, Output>; RS],
    timer: TimerDriver<'a>,
    ms: u64,
    states: u16,
    // prev_states: u16,
    // events: VecDeque<(usize, usize, KeyState)>,
}

impl<'a, const CS: usize, const RS: usize> Keypad<'a, CS, RS> {
    /// Creates a new Matrix.
    ///
    /// Assumes columns are pull-up inputs,
    /// and rows are output pins which are set high when not being scanned.
    pub fn new(
        cols: [PinDriver<'a, AnyIOPin, Input>; CS],
        rows: [PinDriver<'a, AnyOutputPin, Output>; RS],
        timer: TimerDriver<'a>,
    ) -> anyhow::Result<Self> {
        let ms = timer.tick_hz() / 1000;
        let mut res = Self {
            cols,
            rows,
            timer,
            ms,
            states: 0,
            // prev_states: 0,
            // events: VecDeque::new(),
        };
        res.clear()?;
        Ok(res)
    }

    fn clear(&mut self) -> anyhow::Result<()> {
        for r in self.rows.iter_mut() {
            r.set_high()?;
        }

        for c in self.cols.iter_mut() {
            c.set_pull(Pull::Up)?;
        }

        Ok(())
    }

    // fn update_states(&mut self, new_states: u16) {
    //     let changed = self.states ^ new_states;
    //     self.prev_states = self.states;
    //     self.states = new_states;

    //     for idx in 0..(CS * RS) {
    //         let mask = 1 << idx;
    //         if (changed & mask) == 0 {
    //             continue;
    //         }

    //         let event = if (new_states & mask) != 0 {
    //             KeyState::PRESSED | KeyState::JUST_PRESS
    //         } else {
    //             KeyState::JUST_RELEASE
    //         };

    //         // 存储事件(行,列,状态)
    //         let row = idx / CS;
    //         let col = idx % CS;
    //         self.events.push_back((row, col, event));
    //     }
    // }

    /// Scans the matrix and checks which keys are pressed.
    ///
    /// Every row pin in order is pulled low, and then each column
    /// pin is tested; if it's low, the key is marked as pressed.
    /// Scans the pins and checks which keys are pressed (state is "low").
    ///
    /// Delay function allows pause to let input pins settle
    pub async fn scan(&mut self, delay_ms: u64) -> anyhow::Result<()> {
        let mut current_states: u16 = 0;

        for (row_idx, row_pin) in self.rows.iter_mut().enumerate() {
            row_pin.set_low()?;

            self.timer.delay(self.ms * delay_ms).await?;

            for (col_idx, col_pin) in self.cols.iter().enumerate() {
                let key_idx = row_idx * CS + col_idx;

                if col_pin.is_low() {
                    // press
                    current_states |= 1 << key_idx;
                } else {
                    // release
                    current_states &= !(1 << key_idx);
                }
            }

            row_pin.set_high()?;
        }

        self.states = current_states;
        Ok(())
    }

    pub fn states(&self) -> u16 {
        self.states
    }

    // pub fn take_events(&mut self) -> VecDeque<(usize, usize, KeyState)> {
    //     std::mem::take(&mut self.events)
    // }
}

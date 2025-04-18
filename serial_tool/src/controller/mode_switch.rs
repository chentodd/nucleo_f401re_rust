use log::debug;
use log::error;
use std::time::Duration;
use std::time::Instant;

use crate::proto::motor_::MotorTx;
use crate::proto::motor_::Operation;
use crate::ErrorType;
use crate::DEFAULT_CONTROL_MODE;

#[derive(Debug, Default)]
enum ModeSwitchState {
    #[default]
    Idle,
    Start,
    Wait(Instant),
    Done,
    Error,
}

pub struct ModeSwitch<const TIMEOUTSEC: u64> {
    states: Vec<(Operation, ModeSwitchState)>,
    ignited: bool,
    prev_mode: Operation,
    output_mode: Result<Operation, ErrorType>,
}

impl<const TIMEOUTSEC: u64> ModeSwitch<TIMEOUTSEC> {
    pub fn new() -> Self {
        Self {
            states: Vec::new(),
            ignited: false,
            prev_mode: DEFAULT_CONTROL_MODE,
            output_mode: Ok(DEFAULT_CONTROL_MODE),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.states.is_empty()
    }

    pub fn reset(&mut self) {
        self.states.clear();
        self.ignited = false;
        self.output_mode = Ok(self.prev_mode);
    }

    pub fn ignite(&mut self, target_mode: Operation) {
        if self.ignited {
            // Processing mode switch, do not accept other requests
            return;
        }

        if self.output_mode.is_ok_and(|x| x == target_mode) {
            // Output mode is same as target mode, no need to do switching
            return;
        }

        // When user asks to switch connection, we will do:
        // 1. Send stop mode to the board, make sure motor is not moving
        // 2. Send target operation mode
        self.ignited = true;
        self.states.clear();
        self.states.push((target_mode, ModeSwitchState::Idle));
        self.states.push((Operation::Stop, ModeSwitchState::Idle));
    }

    pub fn process(&mut self, motor_data: Option<&MotorTx>) -> Result<Operation, ErrorType> {
        if let Some(motor_data) = motor_data {
            if let Some((req_mode, switch_state)) = self.states.last_mut() {
                debug!(
                    "process, {}, {}, {:?}",
                    req_mode.0, motor_data.operation_display.0, switch_state
                );

                match switch_state {
                    ModeSwitchState::Idle => {
                        if *req_mode != motor_data.operation_display {
                            *switch_state = ModeSwitchState::Start;
                        } else {
                            *switch_state = ModeSwitchState::Done;
                        }

                        debug!(
                            "ModeSwitchState.Idle, {}, {}",
                            req_mode.0, motor_data.operation_display
                        );
                    }
                    ModeSwitchState::Start => {
                        self.prev_mode = self.output_mode.unwrap();
                        self.output_mode = Ok(*req_mode);
                        error!("ModeSwitch.Start: {}", self.output_mode.unwrap().0);
                        *switch_state = ModeSwitchState::Wait(Instant::now());
                    }
                    ModeSwitchState::Wait(prev) => {
                        let now = Instant::now();
                        if now.duration_since(*prev) >= Duration::from_secs(TIMEOUTSEC) {
                            self.output_mode = Err(ErrorType::ModeSwitchTimeout);

                            // Do nothing, wait user to reset the errors
                            *switch_state = ModeSwitchState::Error;
                        } else if *req_mode == motor_data.operation_display {
                            error!("set done");
                            *switch_state = ModeSwitchState::Done;
                        }
                    }
                    ModeSwitchState::Done => {
                        debug!("ModeSwitchState.Done");
                        self.states.pop();
                        self.prev_mode = motor_data.operation_display;

                        if self.states.is_empty() {
                            self.ignited = false;
                        }
                    }
                    _ => (),
                }
            }
        }

        self.output_mode
    }
}

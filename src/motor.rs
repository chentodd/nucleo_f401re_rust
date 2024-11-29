#[cfg(feature = "print-debug")]
use defmt::info;

use embassy_stm32::gpio::Output;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::SimplePwmChannel;
use embassy_stm32::timer::GeneralInstance4Channel;

use crate::encoder::Encoder;
use crate::pid::Pid;

pub struct BldcMotor24H<'a, T1: GeneralInstance4Channel, T2: GeneralInstance4Channel> {
    encoder: Encoder<'a, T1, 400>,
    pwm_channel: SimplePwmChannel<'a, T2>,
    dir_pin: Output<'a>,
    _break_pin: Output<'a>,
    pid: Pid,
    period_s: f32,
}

impl<'a, T1: GeneralInstance4Channel, T2: GeneralInstance4Channel> BldcMotor24H<'a, T1, T2> {
    pub fn new(
        encoder: Encoder<'a, T1, 400>,
        mut pwm_channel: SimplePwmChannel<'a, T2>,
        dir_pin: Output<'a>,
        break_pin: Output<'a>,
        pid: Pid,
        period_s: f32,
    ) -> Self {
        // 24H motor, 0% duty: full speed, 100% duty: 0 speed
        pwm_channel.set_polarity(OutputPolarity::ActiveLow);
        pwm_channel.enable();

        Self {
            encoder: encoder,
            pwm_channel: pwm_channel,
            dir_pin: dir_pin,
            _break_pin: break_pin,
            pid: pid,
            period_s: period_s,
        }
    }

    pub fn set_target_velocity(&mut self, target_velocity_rpm: f32) {
        self.pid.set_target_velocity(target_velocity_rpm);
    }

    pub fn get_current_velocity(&mut self) -> f32 {
        self.encoder.get_curr_velocity_in_rpm(self.period_s)
    }

    pub fn run_pid_velocity_control(&mut self) {
        let curr_velocity_rpm = self.encoder.get_curr_velocity_in_rpm(self.period_s);

        #[cfg(feature = "print-debug")]
        info!("{}, {}", curr_velocity_rpm, self.encoder.curr_enc_count);

        let control_effort: f32 = self.pid.run(curr_velocity_rpm, self.period_s);
        let dir = if control_effort >= 0.0 { 1.0 } else { -1.0 };

        let duty_cycle_percent: u8 = (control_effort * dir * 100.0) as u8;
        if dir < 0.0 {
            self.dir_pin.set_high();
        } else {
            self.dir_pin.set_low();
        }

        self.pwm_channel.set_duty_cycle_percent(duty_cycle_percent);
    }
}
#![allow(dead_code)]

#[derive(Default)]
pub struct PidBuilder {
    setpoint: f32,

    // controller gains
    kp: f32,
    ki: f32,
    kd: f32,

    // output limits
    lim_out: Option<(f32, f32)>,

    // integrator limits
    lim_int: Option<(f32, f32)>,
}
impl PidBuilder {
    fn new() -> Self {
        PidBuilder {
            ..Default::default()
        }
    }
    /// The controller set point
    pub fn setpoint(mut self, sp: f32) -> Self {
        self.setpoint = sp;
        self
    }
    /// The proportional gain
    pub fn gain_p(mut self, kp: f32) -> Self {
        self.kp = kp;
        self
    }
    /// The integral gain
    pub fn gain_i(mut self, ki: f32) -> Self {
        self.ki = ki;
        self
    }
    /// The derivative gain
    pub fn gain_d(mut self, kd: f32) -> Self {
        self.kd = kd;
        self
    }
    /// Limit controller output to prevent damage to DUT
    pub fn limit_output(mut self, min: f32, max: f32) -> Self {
        self.lim_out = Some((min, max));
        self
    }
    /// Limit integrator value to prevent windup
    pub fn limit_integrator(mut self, min: f32, max: f32) -> Self {
        self.lim_int = Some((min, max));
        self
    }
    /// Finalize the builder and return a ready-to-use PI controller.
    ///
    /// See [`PidController`] for examples.
    pub fn build(self) -> PidController {
        let (lim_min, lim_max) = self.lim_out.unwrap_or((f32::NEG_INFINITY, f32::INFINITY));
        let (lim_min_int, lim_max_int) = self.lim_int.unwrap_or((lim_min, lim_max));
        PidController {
            setpoint: self.setpoint,
            kp: self.kp,
            ki: self.ki,
            kd: self.kd,
            lim_min,
            lim_max,
            lim_min_int,
            lim_max_int,
            integrator: 0.0,
            differentiator: 0.0,
            prev_measurement: 0.0,
        }
    }
}

/// A proportional-integral-derivative (PID) controller.
///
/// # Examples
///
/// ```
/// let mut pid_c = PidController::builder()
///     .setpoint(0.0)
///     .gain_p(5.0)
///     .gain_i(3.0)
///     .limit_output(-1.0, 1.0)
///     .build();
///
/// loop {
///     let meas = make_a_new_measurement();
///     let out = pid_c.update(meas);
///     apply_new_output_value(out);
/// }
/// ```
pub struct PidController {
    /// feedback set point
    pub setpoint: f32,

    /// proportional gain
    pub kp: f32,
    /// integral gain
    pub ki: f32,
    /// derivative gain
    pub kd: f32,

    // output limits
    lim_min: f32,
    lim_max: f32,

    // integrator limits
    lim_min_int: f32,
    lim_max_int: f32,

    // controller "memory"
    integrator: f32,
    differentiator: f32,
    prev_measurement: f32,
}
impl PidController {
    /// Start building a new PID controller.
    ///
    /// Call [`PidBuilder::build`] to get a ready-to-use controller.
    /// See [`PidController`] for examples.
    pub fn builder() -> PidBuilder {
        PidBuilder::new()
    }

    /// Provide a new measurement and generate a new output value.
    ///
    /// See [`PidController`] for examples.
    pub fn update(&mut self, measurement: f32) -> f32 {
        let error = self.setpoint - measurement;
        let proportional = self.kp * error;

        self.integrator += self.ki * error;
        // clamp integrator to prevent integral windup
        self.integrator = self.integrator.clamp(self.lim_min_int, self.lim_max_int);

        // Note: derivative on measurement, therefore minus sign in front of equation!
        // self.differentiator = -self.kd * (measurement - self.prev_measurement);
        const TAU: f32 = 3.5; // 0.5 --> no filter
        self.differentiator = -(2.0 * self.kd * (measurement - self.prev_measurement)
            + (2.0 * TAU - 1.0) * self.differentiator)
            / (2.0 * TAU + 1.0);

        let mut output = proportional + self.integrator + self.differentiator;
        // clamp output to prevent damage to DUT
        output = output.clamp(self.lim_min, self.lim_max);

        self.prev_measurement = measurement;

        // return controller output
        output
    }
}

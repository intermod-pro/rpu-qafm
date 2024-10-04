use crate::pid::PidController;
use crate::read_cycle_counter;
use crate::set_dc_bias;
use crate::wait_for_new_data;
use crate::{BiasDac, Data, Params};

/// Function implementing the user logic, including setup and main loop.
///
/// # DC bias connections
/// - DC bias port 1 (channel 0): Z piezo
/// - DC bias port 2 (channel 1): X piezo
/// - DC bias port 3 (channel 2): Y piezo
///
/// # Parameter map
/// | idx | dir   | low 32 bits                 | high 32 bits                |
/// |-----|-------|-----------------------------|-----------------------------|
/// |  0  | write | nr of processed iterations  | CPU cycle counter           |
/// |  1  | write | amp^2 (error signal)        | Z bias (control signal)     |
/// |  2  | read  | lockin amplitude scale      | (unused)                    |
/// |  3  | read  | feedback set point          | proportional gain           |
/// |  4  | read  | feedback integral gain      | derivative gain             |
/// |  5  | read  | scanner X bias              | scanner Y bias              |
/// |  6  | read  | Z bias low limit            | Z bias high limit           |
///
pub fn user_logic(data: Data, bias_dac: BiasDac, params: Params) -> ! {
    // read lockin scale
    let scale = read_scale(&params);

    // read feedback set point and gain parameters
    let (sp, kp, ki, kd) = read_pid_params(&params);
    let (low_lim, high_lim) = read_z_limits(&params);

    // initialize PID controller
    let mut pid_c = PidController::builder()
        .setpoint(sp)
        .gain_p(kp)
        .gain_i(ki)
        .gain_d(kd)
        .limit_output(low_lim, high_lim)
        .build();

    // no iterations processed yet
    let mut irq_count: u32 = 0;
    write_irq_count(&params, irq_count);

    // main loop
    loop {
        // wait until new lockin data is available, then
        // read new data, assume intermediate frequency is zero
        let (data_i, data_q) = get_new_data(&data);

        // rescale
        let data_i = data_i * scale;
        let data_q = data_q * scale;

        // calculate amplitude A^2 = I^2 + Q^2
        let amp2 = (data_i * data_i) + (data_q * data_q);

        // new feedback value
        let bias_norm = pid_c.update(amp2);

        // set new DC bias: Z piezo
        set_dc_bias(&bias_dac, 0, bias_norm); // port 1

        // let APU know current amp^2 (error) and bias (control) values
        write_pid_error_control(&params, amp2, bias_norm);

        // update feedback parameters for next iteration
        (pid_c.setpoint, pid_c.kp, pid_c.ki, pid_c.kd) = read_pid_params(&params);

        // set X and Y scanner bias
        let (bias_x, bias_y) = read_scanner_xy(&params);
        set_dc_bias(&bias_dac, 1, bias_x); // port 2
        set_dc_bias(&bias_dac, 2, bias_y); // port 3

        // let APU know how many iterations we have processed
        irq_count += 1;
        write_irq_count(&params, irq_count);
    }
}

/// Wait until new data is available, then return I and Q quadrature of first frequency.
///
/// Assumes:
/// - using Lockin (not SymmetricLockin)
/// - using digital downconversion (adc_mode=AdcMode.Mixed)
/// - using zero IF
fn get_new_data(data: &Data) -> (f32, f32) {
    wait_for_new_data();
    let (data_i, data_q) = u64_to_f32x2(data.idx(0).read());
    (data_i, data_q)
}

/// Read PID controller parameters from memory:
/// - set point
/// - proportional gain
/// - integral gain
/// - derivative gain
fn read_pid_params(params: &Params) -> (f32, f32, f32, f32) {
    let (sp, kp) = u64_to_f32x2(params.idx(3).read());
    let (ki, kd) = u64_to_f32x2(params.idx(4).read());
    (sp, kp, ki, kd)
}

/// Read normalized bias for X and Y piezo
fn read_scanner_xy(params: &Params) -> (f32, f32) {
    let (x, y) = u64_to_f32x2(params.idx(5).read());
    (x, y)
}

/// Read output limits for normalized bias for Z piezo
fn read_z_limits(params: &Params) -> (f32, f32) {
    let (low, high) = u64_to_f32x2(params.idx(6).read());

    // low should be at least 0.0 and high at most 1.0
    let low = f32::max(low, 0.0);
    let high = f32::min(high, 1.0);

    (low, high)
}

/// Read scaling factor for lockin data
fn read_scale(params: &Params) -> f32 {
    let (scale, _) = u64_to_f32x2(params.idx(2).read());
    scale
}

/// Write number of processed IRQs back to APU.
///
/// Write also current count of CPU cycles, so it's possible to calculate a rate.
fn write_irq_count(params: &Params, count: u32) {
    let val = u32x2_to_u64(count, read_cycle_counter());
    params.idx(0).write(val);
}

/// Write error signal and control signal from PID controller back to APU
fn write_pid_error_control(params: &Params, error: f32, control: f32) {
    let val = f32x2_to_u64(error, control);
    params.idx(1).write(val);
}

/// Convenience function to extract two f32 values from one u64 value
fn u64_to_f32x2(val: u64) -> (f32, f32) {
    let low = f32::from_bits(val as u32);
    let high = f32::from_bits((val >> 32) as u32);
    (low, high)
}

/// Convenience function to pack two f32 values into one u64 value
fn f32x2_to_u64(low: f32, high: f32) -> u64 {
    let low = low.to_bits();
    let high = high.to_bits();
    let mut val = low as u64;
    val |= (high as u64) << 32;
    val
}

/// Convenience function to pack two u32 values into one u64 value
fn u32x2_to_u64(low: u32, high: u32) -> u64 {
    let mut val = low as u64;
    val |= (high as u64) << 32;
    val
}

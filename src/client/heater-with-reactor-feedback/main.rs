pub mod app;
//use std::ops::Deref;
use local_ip_address::local_ip;

pub use app::*;
use uom::si::power::kilowatt;
use uom::si::thermal_conductance::watt_per_kelvin;
use uom::si::thermodynamic_temperature::degree_celsius;
use uom::{si::frequency::hertz, ConstZero};
use uom::si::ratio::ratio;
use uom::si::f64::*;
use chem_eng_real_time_process_control_simulator::alpha_nightly::prelude::*;

use crate::panels::{second_order_transfer_fn::SecondOrderStableTransferFn, decaying_sinusoid::DecayingSinusoid};
fn main() -> eframe::Result<()> {

    use core::time;
    use std::{thread, time::SystemTime, ops::DerefMut};
    use uom::si::{f64::*, time::{millisecond, second}};
    use crate::panels::opcua_panel::try_connect_to_server_and_run_client;
    
    

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        initial_window_size: Some([400.0, 300.0].into()),
        min_window_size: Some([300.0, 220.0].into()),
        ..Default::default()
    };
    let gui_app = GuiClient::new();

    let plot_values_ptr_clone = gui_app.plot_points_ptr.clone();
    let rad_value_ptr_clone = gui_app.rad_value_ptr.clone();
    let time_now = SystemTime::now();

    // for input output plot
    let user_input_ptr_clone = gui_app.user_input.clone();
    let input_output_plots_ptr_clone = gui_app.input_output_plots_ptr.clone();

    // for opcua 

    let pressure_pascals_input_clone = gui_app.loop_pressure_drop_pump_pressure_pascals_input.clone();
    let mass_flowrate_output_clone = gui_app.mass_flowrate_kg_per_s_output.clone();
    let isothermal_ciet_plot_ptr_clone = gui_app.isothermal_ciet_plots_ptr.clone();
    let opcua_ip_addr_ptr_clone = gui_app.opcua_server_ip_addr.clone();

    let bt12_temp_deg_c_ptr_clone = gui_app.bt12_temp_deg_c.clone();
    let bt11_temp_deg_c_ptr_clone = gui_app.bt11_temp_deg_c.clone();
    let heater_power_kilowatts_ptr_clone = gui_app.heater_power_kilowatts.clone();
    let heater_v2_bare_ciet_plots_ptr_clone = gui_app.heater_v2_bare_ciet_plots_ptr.clone();


    // this is for testing second order transfer fn 
    // G(s)
    let mut g_s_second_order_underdamped = SecondOrderStableTransferFn::new(
        1.0, // process gain
        Time::new::<second>(1.0),  // process time
        0.45, // damping factor
        0.0, 
        0.0, 
        Time::new::<second>(1.0)
    );


    // decaying sinusoids 
    let mut g_s_decaying_sine = DecayingSinusoid::new_sine(
        1.0, 
        Frequency::new::<hertz>(0.5), 
        0.0, 
        0.0, 
        Time::new::<second>(1.0),
        Frequency::new::<hertz>(1.5), 
    );

    let mut _g_s_decaying_cosine = DecayingSinusoid::new_cosine(
        1.0, 
        Frequency::new::<hertz>(0.5), 
        0.0, 
        0.0, 
        Time::new::<second>(1.0),
        Frequency::new::<hertz>(1.5), 
    );

    //          0.000119s - 2.201e-7
    // G(s) = -----------------------------
    //          s^2 + 0.0007903 s + 6.667e-7
    let mut heater_inlet_temp_to_heater_outlet_temp_transfer_fn: TransferFn 
        = TransferFnSecondOrder::new(
            Time::ZERO * Time::ZERO, 
            Time::new::<second>(0.000119), 
            - Ratio::new::<ratio>(2.201e-7), 
            Time::new::<second>(1.0)* Time::new::<second>(1.0), 
            Time::new::<second>(0.0007903), 
            Ratio::new::<ratio>(6.667e-7),
        ).unwrap().into();

    //          -1.87086e-6 + 0.00101128 s + 0.000119 s^2
    // G(s) = ------------------------------------------- *3401.36
    //          s^2 + 0.0007903 s + 6.667e-7
    //
    // For now, I'll just do without the gain. 
    // The gain of 3401.36 
    // is in units of kelvin/watt
    //
    let mut heater_inlet_temp_to_heater_power_part_1: TransferFn 
        = TransferFnSecondOrder::new(
            Time::new::<second>(0.000119)* Time::new::<second>(1.0), 
            Time::new::<second>(0.000101128), 
            - Ratio::new::<ratio>(1.87086e-6), 
            Time::new::<second>(1.0)* Time::new::<second>(1.0), 
            Time::new::<second>(0.0007903), 
            Ratio::new::<ratio>(6.667e-7),
        ).unwrap().into();

    //          -1.87086e-6 + 0.00101128 s + 0.000119 s^2
    // G(s) = ------------------------------------------- *(-340.136) * 
    //          s^2 + 0.0007903 s + 6.667e-7
    //
    //          
    //            (4.5)
    //          -------------
    //          0.1 s + 1
    // 
    //
    // looks like I just a first order transfer function
    // For now, I'll just do without the gain. 
    // The gain of 340.136 is in units of kelvin/watt
    let mut heater_inlet_temp_to_heater_power_part_2: TransferFn 
        = TransferFnFirstOrder::new(
            Time::ZERO, 
            Ratio::new::<ratio>(4.5), 
            Time::new::<second>(0.1), 
            Ratio::new::<ratio>(1.0), 
        ).unwrap().into();

    let mut heater_inlet_temp_to_heater_power_part_3: TransferFn 
        = TransferFnSecondOrder::new(
            Time::new::<second>(0.000119)* Time::new::<second>(1.0), 
            Time::new::<second>(0.000101128), 
            - Ratio::new::<ratio>(1.87086e-6), 
            Time::new::<second>(1.0)* Time::new::<second>(1.0), 
            Time::new::<second>(0.0007903), 
            Ratio::new::<ratio>(6.667e-7),
        ).unwrap().into();

    // now spawn a new writer for the heater 

    let mut reference_csv_writer = 
        heater_inlet_temp_to_heater_outlet_temp_transfer_fn.
        spawn_writer("reference_heater_inlet_and_outlet_temp"
            .to_owned()).unwrap();

    let mut reactor_feedback_csv_writer = 
        heater_inlet_temp_to_heater_power_part_2.spawn_writer(
            "reactor_inlet_outlet_temp".to_owned()).unwrap();

    // this is the thread for the user input and 
    // transfer fn
    thread::spawn(move||{
        loop {
            let time_elapsed_ms = time_now.elapsed().unwrap().as_millis();
            let time_elapsed_s: f64 = time_elapsed_ms as f64 / 1000 as f64;


            // push values to vecto64
            //
            //dbg!([time_elapsed_s,5.0]);
            let rad_value: f32 = 
                rad_value_ptr_clone.lock().unwrap().deref_mut().clone();

            plot_values_ptr_clone.lock().unwrap().deref_mut()
                .push([time_elapsed_s,rad_value as f64]);

            // user inputs and outputs must be editable in real-time and 
            // plotable
            let user_input: f32 = 
                user_input_ptr_clone.lock().unwrap().deref_mut().clone();


            let current_time = Time::new::<millisecond>(time_elapsed_ms as f64);


            let model_output_1 = g_s_decaying_sine.set_user_input_and_calc_output(
                current_time, user_input as f64);

            let model_output_2 = g_s_second_order_underdamped.set_user_input_and_calc_output(
                current_time, user_input as f64);
            
            let model_output = model_output_1 + model_output_2;

            //dbg!(&g_s_second_order_underdamped);
            //dbg!(&g_s_decaying_cosine);

            input_output_plots_ptr_clone.lock().unwrap().deref_mut()
                .push([time_elapsed_s,user_input as f64,
                model_output as f64]);

            thread::sleep(time::Duration::from_millis(100));
        }

    });

    // this is the portion where we do opc-ua

    // move client into the thread
    // plus the pointers
    thread::spawn(move || {

        // this is a simple connection loop, but doesn't reconnect 
        // if there is a disconnection

        let my_local_ip = local_ip().unwrap();
        let ip_addr: String = my_local_ip.to_string();        
        let endpoint: String = "opc.tcp://".to_owned()
        +&ip_addr+":4840/rust_ciet_opcua_server";

        let mut connection_result = try_connect_to_server_and_run_client(
            &endpoint,
            2,
            pressure_pascals_input_clone.clone(),
            mass_flowrate_output_clone.clone(),
            bt12_temp_deg_c_ptr_clone.clone(),
            bt11_temp_deg_c_ptr_clone.clone(),
            heater_power_kilowatts_ptr_clone.clone());

        // now, normally it should be well connected, if not, then 
        // retry 
        loop {

            let ip_addr: String = opcua_ip_addr_ptr_clone.lock().unwrap().deref_mut()
            .to_string();
            let endpoint: String = "opc.tcp://".to_owned()
            +&ip_addr+":4840/rust_ciet_opcua_server";

            if let Err(_) = connection_result.clone() {
                connection_result = try_connect_to_server_and_run_client(
                    &endpoint,
                    2,
                    pressure_pascals_input_clone.clone(),
                    mass_flowrate_output_clone.clone(),
                    bt12_temp_deg_c_ptr_clone.clone(),
                    bt11_temp_deg_c_ptr_clone.clone(),
                    heater_power_kilowatts_ptr_clone.clone());

            }

            let time_elapsed_ms = time_now.elapsed().unwrap().as_millis();
            let time_elapsed_s: f64 = time_elapsed_ms as f64 / 1000 as f64;
            let current_time = Time::new::<second>(time_elapsed_s);

            let loop_pressure_drop_pascals: f32 = 
                pressure_pascals_input_clone.lock().unwrap().deref_mut().clone();
            let mass_flowrate_kg_per_s: f32 = 
                mass_flowrate_output_clone.lock().unwrap().deref_mut().clone();

            isothermal_ciet_plot_ptr_clone.lock().unwrap().deref_mut()
                .push([
                    time_elapsed_s,
                    loop_pressure_drop_pascals as f64,
                    mass_flowrate_kg_per_s as f64
                ]);

            let bt11_temp_deg_c: f32 = 
            bt11_temp_deg_c_ptr_clone.lock().unwrap().deref_mut().clone();
            let bt12_temp_deg_c: f32 = 
            bt12_temp_deg_c_ptr_clone.lock().unwrap().deref_mut().clone();

            // changes in inlet temperature will result in reactor feedback 
            let bt_11_temp_deviation: TemperatureInterval = 
                TemperatureInterval::new::<uom::si::temperature_interval::degree_celsius>(
                    (bt12_temp_deg_c - 79.12) as f64);

            // deviation will be fed into transfer function
            // for reference
            let bt12_expected_outlet_temp: ThermodynamicTemperature 
                = get_expected_temperature(bt_11_temp_deviation, 
                    current_time, 
                    &mut heater_inlet_temp_to_heater_outlet_temp_transfer_fn);
            // bind the writer first 

            let reference_reactor_feedback_writer_ptr = 
                &mut reference_csv_writer;

            // record reference feedback
            heater_inlet_temp_to_heater_outlet_temp_transfer_fn.
                csv_write_values(
                    reference_reactor_feedback_writer_ptr, 
                    current_time, 
                    (bt11_temp_deg_c as f64).into(), 
                    bt12_expected_outlet_temp.get::<degree_celsius>().into()
                ).unwrap();

            // now let's obtain the power signal

            let reactor_power_signal: Power = 
                get_reactor_feedback(
                    bt_11_temp_deviation, 
                    current_time, 
                    &mut heater_inlet_temp_to_heater_power_part_1, 
                    &mut heater_inlet_temp_to_heater_power_part_2, 
                    &mut heater_inlet_temp_to_heater_power_part_3);

            // heater ptr lock 

            let mut heater_ptr_lock = 
                heater_power_kilowatts_ptr_clone.lock().unwrap();

            *heater_ptr_lock = reactor_power_signal.get::<kilowatt>() as f32;

            let heater_power_kilowatts: f32 = 
                reactor_power_signal.get::<kilowatt>() as f32;


            // write csv

            let reactor_feedback_csv_writer_ptr = 
                &mut reactor_feedback_csv_writer;

            heater_inlet_temp_to_heater_power_part_1.
                csv_write_values(
                    reactor_feedback_csv_writer_ptr, 
                    current_time, 
                    (bt11_temp_deg_c as f64).into(), 
                    (bt12_temp_deg_c as f64).into()
                ).unwrap();
            
            heater_v2_bare_ciet_plots_ptr_clone.lock().unwrap().deref_mut()
                .push([
                    time_elapsed_s,
                    bt11_temp_deg_c as f64,
                    heater_power_kilowatts as f64,
                    bt12_temp_deg_c as f64,
                ]);


            

            thread::sleep(time::Duration::from_millis(100));
        }

        // now, if the client connects correctly, then we should be able 
        // to append the plots for the pointer

    });


    // last but not least, the main thread runs eframe natively
    eframe::run_native(
        "OPC-UA GUI Client",
        native_options,
        Box::new(|_cc| Box::new(gui_app)),
    )
}

fn get_reactor_feedback(bt_11_deviation: TemperatureInterval,
    current_time: Time,
    transfer_fn_part1: &mut TransferFn,
    transfer_fn_part2: &mut TransferFn,
    transfer_fn_part3: &mut TransferFn) -> Power {

    let gain_for_part1 = ThermalConductance::new::<watt_per_kelvin>(
        3401.36);

    let gain_for_part2 = -ThermalConductance::new::<watt_per_kelvin>(
        340.136);

    let one_kelvin_interval = 
        TemperatureInterval::new::<uom::si::temperature_interval::kelvin>(1.0);

    let user_input: Ratio = bt_11_deviation/one_kelvin_interval;

    // this is for transfer function 
    //
    // 3401.36 * 
    // (-1.87085e-6 + 0.00101128 s + 0.000119 s^2)/
    // (s^2 + 0.0007903 + 6.667e-7)

    let output_second_order_term: Power = gain_for_part1 
        * one_kelvin_interval
        * transfer_fn_part1.set_user_input_and_calc(user_input, 
            current_time).unwrap();

    
    // this is for transfer function 
    //
    // -340.136 * 
    // (4.5)/(0.1 s+1)
    // (-1.87085e-6 + 0.00101128 s + 0.000119 s^2)/
    // (s^2 + 0.0007903 + 6.667e-7)
    //
    // in time domain, there is an intermediate input...
    let intermediate_input: Ratio = 
        transfer_fn_part2.set_user_input_and_calc(user_input, 
            current_time).unwrap();

    let output_third_order_term: Power = 
        gain_for_part2
        * one_kelvin_interval
        * transfer_fn_part3.set_user_input_and_calc(
            intermediate_input, current_time).unwrap();

    let power_signal = output_second_order_term + output_third_order_term
        + Power::new::<uom::si::power::kilowatt>(8.0);

    // check if power signal less than 0 
    if power_signal < Power::ZERO {
        return Power::ZERO;
    }

    return power_signal;

}

fn get_expected_temperature(bt_11_deviation: TemperatureInterval,
    current_time: Time,
    reactor_feedback_reference_transfer_fn: &mut TransferFn) -> ThermodynamicTemperature {


    let one_kelvin_interval = 
        TemperatureInterval::new::<uom::si::temperature_interval::kelvin>(1.0);

    let bt_11_deviation_ratio = bt_11_deviation/one_kelvin_interval;

    let bt_12_temperature_interval_ratio: Ratio = reactor_feedback_reference_transfer_fn
        .set_user_input_and_calc(
            bt_11_deviation_ratio, current_time).unwrap();

    let bt_12_temperature_interval: TemperatureInterval
        = bt_12_temperature_interval_ratio * one_kelvin_interval;

    let bt_12_expected_temperature: ThermodynamicTemperature 
        = ThermodynamicTemperature::new::<degree_celsius>(102.41)
        + bt_12_temperature_interval;

    return bt_12_expected_temperature;

}

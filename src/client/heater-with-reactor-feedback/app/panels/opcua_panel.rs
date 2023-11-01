use std::{ops::DerefMut, thread, time};

use egui::Ui;
use uom::si::power::kilowatt;

use crate::GuiClient;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use opcua::client::prelude::*;
use opcua::sync::RwLock;
use std::sync::{Arc, Mutex};
use uom::si::f64::*;

impl GuiClient {
    
    pub fn ciet_isothermal_panel_ui(&mut self, ui: &mut Ui) {

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("IP Address for Server (IPv4): ");
            ui.text_edit_singleline(
                self.opcua_server_ip_addr.lock().unwrap().deref_mut());
        });
        ui.separator();
        ui.add(egui::Spinner::new());
        // slider changes the user input value
        // and we release the mutex lock immediately
        {
            let mut binding = self.loop_pressure_drop_pump_pressure_pascals_input.lock().unwrap();
            let user_input_value = binding.deref_mut();
            ui.add(egui::Slider::new(user_input_value, -20000.0..=20000.0).
                text("user loop pressure drop input (Pa)"));

        }


        let mut opcua_plot = Plot::new("loop pressure drop plot").legend(Legend::default());

        // sets the aspect for plot 
        opcua_plot = opcua_plot.width(500.0);
        opcua_plot = opcua_plot.view_aspect(16.0/9.0);
        opcua_plot = opcua_plot.data_aspect(2.5);
        opcua_plot = opcua_plot.auto_bounds_x();
        opcua_plot = opcua_plot.auto_bounds_y();

        // let's create a line in the plot
        let opcua_plot_pts: Vec<[f64;3]> = self.
            isothermal_ciet_plots_ptr.lock().unwrap().deref_mut()
            .iter().map(|&values|{
                values}
            ).collect();

        let time_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,_,_] = *tuple;

                time
            }
        ).collect();

        let opcua_user_input_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [_,opcua_user_input,_] = *tuple;

                opcua_user_input
            }
        ).collect();

        let opcua_user_output_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [_,_,opcua_user_output] = *tuple;

                opcua_user_output
            }
        ).collect();


        let time_input_vec: Vec<[f64;2]> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,opcua_user_input,_] = *tuple;

                [time, opcua_user_input]
            }
        ).collect();

        let time_output_vec: Vec<[f64;2]> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,_,opcua_model_output] = *tuple;

                [time, opcua_model_output]
            }
        ).collect();

        let max_time = time_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);
        let max_user_input = opcua_user_input_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);
        let current_user_input = opcua_user_input_vec.clone().into_iter().last();

        let current_user_input = match current_user_input {
            Some(float) => float,
            None => 0.0,
        };

        // include max x and y values 
        opcua_plot = opcua_plot.include_x(max_time);
        opcua_plot = opcua_plot.include_y(max_user_input);

        // axis labels 
        opcua_plot = opcua_plot.x_axis_label(
            "time (seconds), current time (seconds): ".to_owned() 
            + &max_time.to_string());
        opcua_plot = opcua_plot.y_axis_label(
            "Pressure (Pa) ; \n  current pressure (Pa): ".to_owned()
            + &current_user_input.to_string());

        // now truncate values that are too old
        // show only last minute 
        let time_window_seconds = 10.0;
        if max_time as f64 > time_window_seconds as f64 {
            // i want to delete time older than time_window_seconds
            let index_result = time_vec.clone().iter().position(
                |&time| {
                    // we check if the time is less than the oldest 
                    // allowable time 
                    let oldest_allowable_time = max_time - time_window_seconds;
                    time < oldest_allowable_time
                }
            );
            let _ = match index_result {
                Some(index) => {
                    self.isothermal_ciet_plots_ptr.lock().unwrap().deref_mut().remove(index);
                },
                None => {
                    // do nothing 
                    ()
                },
            };

        }




        // second plot for the 
        ui.separator();
        let mut opcua_mass_flow_plot = Plot::new("mass flowrate plot").legend(Legend::default());

        // sets the aspect for plot 
        opcua_mass_flow_plot = opcua_mass_flow_plot.width(500.0);
        opcua_mass_flow_plot = opcua_mass_flow_plot.view_aspect(16.0/9.0);
        opcua_mass_flow_plot = opcua_mass_flow_plot.data_aspect(2.5);
        opcua_mass_flow_plot = opcua_mass_flow_plot.auto_bounds_x();
        opcua_mass_flow_plot = opcua_mass_flow_plot.auto_bounds_y();
        opcua_mass_flow_plot = opcua_mass_flow_plot.x_axis_label(
            "time (seconds)");
        let current_user_output = opcua_user_output_vec.clone().into_iter().last();

        let mut current_user_output = match current_user_output {
            Some(float) => float,
            None => 0.0,
        };

        // 4dp rounding
        current_user_output = 
            (current_user_output * 10000.0).round()/10000.0;


        opcua_mass_flow_plot = opcua_mass_flow_plot.y_axis_label(
            "mass flowrate (kg/s) \n 
            current mass flowrate: ".to_owned() +
            &current_user_output.to_string());

        ui.horizontal(|ui| {
            opcua_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(
                    time_input_vec.clone()
                )).name("opc-ua user input (loop pressure drop [Pa])"));
            });
            opcua_mass_flow_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(
                    time_output_vec
                )).name("mass flowrate kg/s"));
            });
        });
    }


    pub fn ciet_heater_bare_panel_ui(&mut self, ui: &mut Ui) {

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("IP Address for Server (IPv4): ");
            ui.text_edit_singleline(
                self.opcua_server_ip_addr.lock().unwrap().deref_mut());
        });
        ui.separator();
        ui.add(egui::Spinner::new());
        // slider changes the user input value
        // and we release the mutex lock immediately
        //
        // also have a binding for heater inlet temperature, release 
        // mutex lock immediately
        {
            let mut binding = self.heater_power_kilowatts.lock().unwrap();
            let user_heater_power_input_value = binding.deref_mut();

            let heater_power: Power = Power::new::<kilowatt>(
                *user_heater_power_input_value as f64);

            let heater_power_kilowatts_string = ((heater_power.get::<kilowatt>()
                *1000.0).round()/1000.0).to_string();

            let heater_power_diagnostic: String = 
            "Heater Power: ".to_string() + 
            &heater_power_kilowatts_string + " kilowatts";
            
            ui.horizontal(|ui| {
                ui.set_height(0.0);
                ui.label(&heater_power_diagnostic);
            });
            let mut inlet_temp_binding 
            = self.bt11_temp_deg_c.lock().unwrap();

            ui.add(egui::Slider::new(inlet_temp_binding.deref_mut(), 65.0..=100.0).
                text("user set inlet temperature (degree_celsius)"));

        }


        let mut bt11_bt12_temp_plot = Plot::new("heater inlet and outlet temp degC").legend(Legend::default());

        // sets the aspect for plot 
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.width(500.0);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.view_aspect(16.0/9.0);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.data_aspect(2.5);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.auto_bounds_x();
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.auto_bounds_y();

        // let's create a line in the plot
        let opcua_plot_pts: Vec<[f64;4]> = self.
            heater_v2_bare_ciet_plots_ptr.lock().unwrap().deref_mut()
            .iter().map(|&values|{
                values}
            ).collect();

        let time_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,_,_,_] = *tuple;

                time
            }
        ).collect();

        // it will be arranged [time, bt11, heater_power, bt12]
        let bt11_temp_input_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [_,bt11_temp,_,_] = *tuple;

                bt11_temp
            }
        ).collect();

        // it will be arranged [time, bt11, heater_power, bt12]
        let bt12_temp_output_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [_,_,_,heater_outlet_temp] = *tuple;

                heater_outlet_temp
            }
        ).collect();

        let heater_power_input_vec: Vec<f64> = opcua_plot_pts.iter().map(
            |tuple|{
                let [_,_,heater_power_kw,_] = *tuple;

                heater_power_kw
            }
        ).collect();


        // it will be arranged [time, bt11, heater_power, bt12]
        let time_bt11_vec: Vec<[f64;2]> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,bt11_temp,_,_] = *tuple;

                [time, bt11_temp]
            }
        ).collect();

        // it will be arranged [time, bt11, heater_power, bt12]
        let time_bt12_vec: Vec<[f64;2]> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,_,_,bt12_temp] = *tuple;

                [time, bt12_temp]
            }
        ).collect();

        // it will be arranged [time, bt11, heater_power, bt12]
        let time_heater_power_vec: Vec<[f64;2]> = opcua_plot_pts.iter().map(
            |tuple|{
                let [time,_,heater_power_kw,_] = *tuple;

                [time, heater_power_kw]
            }
        ).collect();

        // now, we also get the expected bt12 outlet temp, which 
        // acts as a set point for the controller
        let time_simulated_reactor_feedback_outlet_temp_vec: Vec<[f64;2]> 
            = self.reactor_feedback_plot_points_ptr.lock().unwrap().deref_mut()
            .iter().map(|&values| {
                values
            }).collect();
        

        let max_time = time_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);
        let max_user_input = bt11_temp_input_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);

        let current_bt11_option = bt11_temp_input_vec.clone().into_iter().last();
        let current_bt11 = match current_bt11_option {
            Some(float) => float,
            None => 0.0,
        };


        // axis labels 
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.x_axis_label(
            "time (seconds), current time (seconds): ".to_owned() 
            + &max_time.to_string());
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.y_axis_label(
            "temperature degree_celsius".to_owned());

        // now truncate values that are too old
        // show only last minute 
        let bt_11_bt_12_time_window_seconds = 45.0;
        if max_time as f64 > bt_11_bt_12_time_window_seconds as f64 {
            // i want to delete time older than time_window_seconds
            let index_result = time_vec.clone().iter().position(
                |&time| {
                    // we check if the time is less than the oldest 
                    // allowable time 
                    let oldest_allowable_time = max_time - bt_11_bt_12_time_window_seconds;
                    time < oldest_allowable_time
                }
            );
            let _ = match index_result {
                Some(index) => {
                    self.heater_v2_bare_ciet_plots_ptr.lock().unwrap().deref_mut().remove(index);
                },
                None => {
                    // do nothing 
                    ()
                },
            };

        }

        if max_time as f64 > bt_11_bt_12_time_window_seconds as f64 {
            // i want to delete time older than time_window_seconds
            let index_result = time_vec.clone().iter().position(
                |&time| {
                    // we check if the time is less than the oldest 
                    // allowable time 
                    let oldest_allowable_time = max_time - bt_11_bt_12_time_window_seconds;
                    time < oldest_allowable_time
                }
            );
            let _ = match index_result {
                Some(index) => {
                    self.reactor_feedback_plot_points_ptr.lock().unwrap().deref_mut().remove(index);
                },
                None => {
                    // do nothing 
                    ()
                },
            };

        }

        let current_bt12_option = bt12_temp_output_vec.clone().into_iter().last();
        let current_bt12 = match current_bt12_option {
            Some(float) => float,
            None => 0.0,
        };



        // include max x and y values 
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.include_x(max_time);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.include_y(max_user_input);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.include_y(current_bt11);
        bt11_bt12_temp_plot = bt11_bt12_temp_plot.include_y(current_bt12);
        // second plot for the 
        ui.separator();
        let mut power_plot = Plot::new("mass flowrate plot").legend(Legend::default());

        // sets the aspect for plot 
        power_plot = power_plot.width(500.0);
        power_plot = power_plot.view_aspect(16.0/9.0);
        power_plot = power_plot.data_aspect(2.5);
        power_plot = power_plot.auto_bounds_x();
        power_plot = power_plot.auto_bounds_y();
        power_plot = power_plot.x_axis_label(
            "time (seconds)");
        let current_user_output = heater_power_input_vec.clone().into_iter().last();

        let mut heater_power_kilowatt = match current_user_output {
            Some(float) => float,
            None => 0.0,
        };

        // 4dp rounding
        heater_power_kilowatt = 
            (heater_power_kilowatt * 10000.0).round()/10000.0;


        power_plot = power_plot.y_axis_label(
            "heater power (kW) \n
            current heater power: ".to_owned() +
            &heater_power_kilowatt.to_string());

        ui.horizontal(|ui|{
            bt11_bt12_temp_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(
                    time_bt11_vec.clone()
                )).name("bt11 (heater inlet) temperature deg C"));
                plot_ui.line(Line::new(PlotPoints::from(
                    time_bt12_vec.clone()
                )).name("bt12 (heater outlet) temperature deg C"));
                plot_ui.line(Line::new(PlotPoints::from(
                    time_simulated_reactor_feedback_outlet_temp_vec.clone()
                )).name("simulated reactivity bt12 (heater outlet) temperature deg C"));
            });
            power_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from(
                    time_heater_power_vec
                )).name("Heater Power (kW)"));
            });

        });
    }

}


pub fn subscribe_to_variables(session: Arc<RwLock<Session>>, ns: u16) -> Result<(), StatusCode> {
    let session = session.read();
    // Creates a subscription with a data change callback
    let subscription_id = session.create_subscription(
        2000.0,
        10,
        30,
        0,
        0,
        true,
        DataChangeCallback::new(|changed_monitored_items| {
            println!("Data change from server:");
            changed_monitored_items
                .iter()
                .for_each(|item| print_value(item));
        }),
    )?;
    println!("Created a subscription with id = {}", subscription_id);

    // Create some monitored items
    let items_to_create: Vec<MonitoredItemCreateRequest> = ["v1", 
    "calculation_time_ms", "v3", "v4"]
        .iter()
        .map(|v| NodeId::new(ns, *v).into())
        .collect();
    let _ = session.create_monitored_items(
        subscription_id,
        TimestampsToReturn::Both,
        &items_to_create,
    )?;


    Ok(())
}
pub fn print_value(item: &MonitoredItem) {
    let node_id = &item.item_to_monitor().node_id;
    let data_value = item.last_value();
    if let Some(ref value) = data_value.value {
        println!("Item \"{}\", Value = {:?}", node_id, value);
    } else {
        println!(
            "Item \"{}\", Value not found, error: {}",
            node_id,
            data_value.status.as_ref().unwrap()
        );
    }
}
pub fn try_connect_to_server_and_run_client(endpoint: &str,
    ns: u16,
    loop_pressure_drop_input_ptr: Arc<Mutex<f32>>,
    isothermal_mass_flow_output_ptr: Arc<Mutex<f32>>,
    bt12_temp_deg_c_output_ptr: Arc<Mutex<f32>>,
    bt11_temp_deg_c_input_ptr: Arc<Mutex<f32>>,
    heater_power_kilowatts_input_ptr: Arc<Mutex<f32>>,
) -> Result<(),StatusCode>{

    // Make the client configuration
    let mut client = ClientBuilder::new()
        .application_name("Simple Client")
        .application_uri("urn:SimpleClient")
        .product_uri("urn:SimpleClient")
        .trust_server_certs(true)
        .create_sample_keypair(true)
        .session_retry_limit(5)
        .client()
        .unwrap();

    let session = client.connect_to_endpoint(
        (endpoint,
         SecurityPolicy::None.to_str(),
         MessageSecurityMode::None,
         UserTokenPolicy::anonymous(),
        ), IdentityToken::Anonymous,
        )?;

    //subscribe_to_variables(session.clone(), ns)?;


    let _ = Session::run_async(session.clone());

    // i want to poll the server and print values 
    let ctah_branch_mass_flowrate_node = NodeId::new(ns, "ctah_branch_mass_flowrate");
    let heater_branch_mass_flowrate_node = NodeId::new(ns, "heater_branch_flowrate");
    let calculation_time_node = NodeId::new(ns, "calculation_time");
    let ctah_pump_pressure_node = NodeId::new(ns, "ctah_pump_pressure");
    let bt11_temperature_node = NodeId::new(ns, "bt11_temperature_degC");
    let bt12_temperature_node = NodeId::new(ns, "bt12_temperature_degC");
    let heater_power_node = NodeId::new(ns, "heater_power_kilowatts");

    // i will also need another thread to run the polling loop 

    thread::spawn( move ||{
        loop {

            // this is the reading part
            {
                let session_lock = session.read();
                let results = session_lock
                    .read(&[
                        ctah_branch_mass_flowrate_node.clone().into(),
                        ctah_pump_pressure_node.clone().into(),
                        calculation_time_node.clone().into(),
                        heater_branch_mass_flowrate_node.clone().into(),
                        bt11_temperature_node.clone().into(),
                        heater_power_node.clone().into(),
                        bt12_temperature_node.clone().into(),
                    ], TimestampsToReturn::Both, 1.0)
                    .unwrap();
                //let value = &results[0];

                // now lock the mutex 
                let mut heater_mass_flowrate_to_gui = isothermal_mass_flow_output_ptr.lock().unwrap();

                // obtain the heater_branch_flowrate, which should be 
                // index 3

                let heater_br_flow_data_value = &results[3];

                let heater_branch_flowrate: f32 = 
                    heater_br_flow_data_value.value.clone()
                    .unwrap().as_f64().unwrap()
                    as f32;

                *heater_mass_flowrate_to_gui = heater_branch_flowrate;

                // now for bt12, do the same 
                let mut heater_exit_temp_to_gui = 
                bt12_temp_deg_c_output_ptr.lock().unwrap();

                let bt12_exit_temp_data_val = &results[6];

                let bt12_exit_temp_deg_c: f32 = 
                bt12_exit_temp_data_val.value.clone().unwrap()
                    .as_f64().unwrap() as f32;

                *heater_exit_temp_to_gui = bt12_exit_temp_deg_c;


            }

            // now for the writing part, we take the user input pressure 
            // drop

            {
                // first, get user inputs
                let user_input_pressure_drop: f32 = 
                loop_pressure_drop_input_ptr.lock().unwrap().to_owned();

                let user_input_heater_inlet_temp: f32 = 
                bt11_temp_deg_c_input_ptr.lock().unwrap().to_owned();

                let user_input_heater_power_kilowatts: f32 = 
                heater_power_kilowatts_input_ptr.lock().unwrap().to_owned();

                //dbg!(&user_input_heater_power_kilowatts);


                // next, create the write values
                let ctah_pump_node_write: WriteValue = WriteValue {
                        node_id: ctah_pump_pressure_node.clone(),
                        attribute_id: AttributeId::Value as u32,
                        index_range: UAString::null(),
                        value: Variant::Float(user_input_pressure_drop).into(),
                    };


                let heater_inlet_temp_node_write: WriteValue = WriteValue {
                        node_id: bt11_temperature_node.clone(),
                        attribute_id: AttributeId::Value as u32,
                        index_range: UAString::null(),
                        value: Variant::Float(user_input_heater_inlet_temp).into(),
                    };

                let heater_power_node_write: WriteValue = WriteValue {
                        node_id: heater_power_node.clone(),
                        attribute_id: AttributeId::Value as u32,
                        index_range: UAString::null(),
                        value: Variant::Float(user_input_heater_power_kilowatts).into(),
                    };
                // now mutex lock the session, 
                let session_lock = session.read();
                // put write values into the write session lock

                let _ = session_lock
                    .write(&[
                        ctah_pump_node_write,
                        heater_inlet_temp_node_write,
                        heater_power_node_write,
                    ])
                    .unwrap();
            }

            // tbc, need to understand how the reading works here
            // look into integration tests for an examples of how read and 
            // write syntax are used
            //let value = session_lock.read(
            //    &[2], 
            //    TimestampsToReturn::Both, 
            //    1000.0)?;
            thread::sleep(time::Duration::from_millis(70));
        }

        //let stop_session = false;

        //if stop_session {
        //    // Terminate the session loop
        //    session_tx.send(SessionCommand.stop());
        //};

    });

    Ok(())

}



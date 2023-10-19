use std::{ops::DerefMut, thread, time};

use egui::Ui;

use crate::GuiClient;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use opcua::client::prelude::*;
use opcua::sync::RwLock;
use std::sync::{Arc, Mutex};

impl GuiClient {
    
    pub fn opcua_panel_ui(&mut self, ui: &mut Ui) {

        ui.separator();
        ui.add(egui::Spinner::new());
        // slider changes the user input value
        // and we release the mutex lock immediately
        {
            let mut binding = self.opcua_input.lock().unwrap();
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
            opcua_plots_ptr.lock().unwrap().deref_mut()
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
                    self.opcua_plots_ptr.lock().unwrap().deref_mut().remove(index);
                },
                None => {
                    // do nothing 
                    ()
                },
            };

        }



        opcua_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(PlotPoints::from(
                        time_input_vec.clone()
            )).name("opc-ua user input (loop pressure drop [Pa])"));
        });

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

        opcua_mass_flow_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(PlotPoints::from(
                        time_output_vec
            )).name("mass flowrate kg/s"));
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
    opcua_input_ptr: Arc<Mutex<f32>>,
    opcua_output_ptr: Arc<Mutex<f32>>) -> Result<(),StatusCode>{

    // Make the client configuration
    let mut client = ClientBuilder::new()
        .application_name("Simple Client")
        .application_uri("urn:SimpleClient")
        .product_uri("urn:SimpleClient")
        .trust_server_certs(true)
        .create_sample_keypair(true)
        .session_retry_limit(3)
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
                    ], TimestampsToReturn::Both, 1.0)
                    .unwrap();
                //let value = &results[0];

                // now lock the mutex 
                let mut output_to_gui = opcua_output_ptr.lock().unwrap();

                // obtain the heater_branch_flowrate, which should be 
                // index 3

                let heater_br_flow_data_value = &results[3];

                let heater_branch_flowrate: f32 = 
                    heater_br_flow_data_value.value.clone()
                    .unwrap().as_f64().unwrap()
                    as f32;

                *output_to_gui = heater_branch_flowrate;


                //dbg!(heater_br_flow_data_value);
            }

            // now for the writing part, we take the user input pressure 
            // drop

            {
                let user_input_pressure_drop: f32 = 
                    opcua_input_ptr.lock().unwrap().to_owned();
                // now mutex lock the session, 
                let session_lock = session.read();
                let _ = session_lock
                    .write(&[WriteValue {
                        node_id: ctah_pump_pressure_node.clone(),
                        attribute_id: AttributeId::Value as u32,
                        index_range: UAString::null(),
                        value: Variant::Float(user_input_pressure_drop).into(),
                    }])
                    .unwrap();
            }

            // tbc, need to understand how the reading works here
            // look into integration tests for an examples of how read and 
            // write syntax are used
            //let value = session_lock.read(
            //    &[2], 
            //    TimestampsToReturn::Both, 
            //    1000.0)?;
            thread::sleep(time::Duration::from_millis(100));
        }

        //let stop_session = false;

        //if stop_session {
        //    // Terminate the session loop
        //    session_tx.send(SessionCommand.stop());
        //};

    });

    Ok(())

}



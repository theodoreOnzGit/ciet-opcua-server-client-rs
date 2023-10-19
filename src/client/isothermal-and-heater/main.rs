pub mod app;
pub use app::*;
fn main() -> eframe::Result<()> {

    use core::time;
    use std::{thread, time::SystemTime, ops::DerefMut};
    use uom::si::{f64::Time, time::{millisecond, second}};
    use crate::panels::opcua_panel::try_connect_to_server_and_run_client;
    use crate::first_order_transfer_fn::FirstOrderTransferFn;
    
    

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

    let opcua_input_clone = gui_app.opcua_input.clone();
    let opcua_output_clone = gui_app.opcua_output.clone();
    let opcua_plots_ptr_clone = gui_app.opcua_plots_ptr.clone();
    let opcua_ip_addr_ptr_clone = gui_app.opcua_server_ip_addr.clone();

    // let's make a first order transfer fn 
    let mut g_s = FirstOrderTransferFn::new(
        1.0, 
        Time::new::<second>(1.0), 
        0.0, 
        0.0, 
        Time::new::<second>(4.0)
        );

    // this is the thread for the user input and 
    // first order transfer fn
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

            let model_output = g_s.set_user_input_and_calc_output(
                current_time, user_input as f64);

            //dbg!(&g_s);

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
        let mut connection_result = try_connect_to_server_and_run_client(
            "opc.tcp://10.25.199.152:4840abcde/rust_ciet_opcua_server",
            2,
            opcua_input_clone.clone(),
            opcua_output_clone.clone());

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
                    opcua_input_clone.clone(),
                    opcua_output_clone.clone());

            }

            let time_elapsed_ms = time_now.elapsed().unwrap().as_millis();
            let time_elapsed_s: f64 = time_elapsed_ms as f64 / 1000 as f64;

            let opcua_input: f32 = 
                opcua_input_clone.lock().unwrap().deref_mut().clone();
            let opcua_output: f32 = 
                opcua_output_clone.lock().unwrap().deref_mut().clone();

            opcua_plots_ptr_clone.lock().unwrap().deref_mut()
                .push([time_elapsed_s,opcua_input as f64,
                opcua_output as f64]);
            

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

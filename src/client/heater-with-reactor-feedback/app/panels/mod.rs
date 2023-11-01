use std::ops::DerefMut;

use egui::Ui;

use crate::GuiClient;
use egui_plot::{Legend, Line, Plot, PlotPoints};

pub mod opcua_panel;

impl GuiClient {

    pub fn transfer_fn_input_output_panel_ui(&mut self, ui: &mut Ui) {

        ui.separator();
        ui.add(egui::Spinner::new());

        let mut binding = self.user_input.lock().unwrap();
        let user_input_value = binding.deref_mut();
        ui.add(egui::Slider::new(user_input_value, 0.0..=0.9).
            text("units TBD"));

        drop(binding);

        let mut my_plot = Plot::new("My Plot").legend(Legend::default());

        // sets the aspect for plot 
        my_plot = my_plot.width(800.0);
        my_plot = my_plot.view_aspect(16.0/9.0);
        my_plot = my_plot.data_aspect(2.5);
        my_plot = my_plot.auto_bounds_x();
        my_plot = my_plot.auto_bounds_y();

        // let's create a line in the plot
        let input_output_plot_pts: Vec<[f64;3]> = self.
            input_output_plots_ptr.lock().unwrap().deref_mut()
            .iter().map(|&values|{
                values}
            ).collect();

        let time_vec: Vec<f64> = input_output_plot_pts.iter().map(
            |tuple|{
                let [time,_,_] = *tuple;

                time
            }
        ).collect();

        let user_input_vec: Vec<f64> = input_output_plot_pts.iter().map(
            |tuple|{
                let [_,user_input,_] = *tuple;

                user_input
            }
        ).collect();

        let time_input_vec: Vec<[f64;2]> = input_output_plot_pts.iter().map(
            |tuple|{
                let [time,user_input,_] = *tuple;

                [time, user_input]
            }
        ).collect();

        let time_output_vec: Vec<[f64;2]> = input_output_plot_pts.iter().map(
            |tuple|{
                let [time,_,model_output] = *tuple;

                [time, model_output]
            }
        ).collect();

        let max_time = time_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);
        let max_user_input = user_input_vec.clone().into_iter().fold(f64::NEG_INFINITY, f64::max);

        // include max x and y values 
        my_plot = my_plot.include_x(max_time);
        my_plot = my_plot.include_y(max_user_input);

        // axis labels 
        my_plot = my_plot.x_axis_label(
            "time (seconds), current time (seconds): ".to_owned() 
            + &max_time.to_string());

        // now truncate values that are too old
        // show only last minute 
        opcua_panel::clear_plot_vectors(self, ui, time_vec);



        my_plot.show(ui, |plot_ui| {
            plot_ui.line(Line::new(PlotPoints::from(
                        time_input_vec
            )).name("user input"));
            plot_ui.line(Line::new(PlotPoints::from(
                        time_output_vec
            )).name("model input"));
        });
    }
}


use std::ops::DerefMut;
use std::sync::Mutex;
use std::sync::Arc;

use eframe::egui;
pub mod panels;
pub use panels::first_order_transfer_fn;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize,Clone)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GuiClient {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    pub rad_value_ptr: Arc<Mutex<f32>>,

    // plot values, locked behind an Arc::mutex lock 
    pub plot_points_ptr: Arc<Mutex<Vec<[f64;2]>>>,

    // for input and output of a simple transfer function
    #[serde(skip)] 
    pub user_input: Arc<Mutex<f32>>,
    #[serde(skip)] 
    pub model_output: Arc<Mutex<f32>>,

    pub input_output_plots_ptr: Arc<Mutex<Vec<[f64;3]>>>,

    // for input and output of opcua server and client
    #[serde(skip)] 
    pub opcua_input: Arc<Mutex<f32>>,
    #[serde(skip)] 
    pub opcua_output: Arc<Mutex<f32>>,
    #[serde(skip)] 
    pub opcua_server_ip_addr: Arc<Mutex<String>>,

    pub opcua_plots_ptr: Arc<Mutex<Vec<[f64;3]>>>,
    // selected panel for graph plotting 
    open_panel:  Panel,
}

#[derive(serde::Deserialize, serde::Serialize,PartialEq,Clone)]
enum Panel {
    Simple,
    InputOutput,
    IsothermalCIET,
    HeaterV2BareCIET,
}

impl Default for GuiClient {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Roentgen".to_owned(),
            rad_value_ptr: Arc::new(Mutex::new(3.6)),
            plot_points_ptr: Arc::new(
                Mutex::new(vec![])
            ),
            open_panel: Panel::IsothermalCIET,
            user_input: Arc::new(Mutex::new(0.0)),
            model_output: Arc::new(Mutex::new(0.0)),
            input_output_plots_ptr: Arc::new(
                Mutex::new(vec![])
            ),
            opcua_input: Arc::new(Mutex::new(0.0)),
            opcua_output: Arc::new(Mutex::new(0.0)),
            opcua_plots_ptr: Arc::new(
                Mutex::new(vec![])
            ),
            opcua_server_ip_addr: Arc::new(Mutex::new(
                "127.0.0.1".to_string())),
        }
    }
}

impl GuiClient {
    /// Called once before the first frame.
    pub fn new() -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        //// Load previous app state (if any).
        //// Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        Default::default()
    }

}


impl eframe::App for GuiClient {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading(" 3.6 Roentgen... Not great not terrible");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            let mut binding = self.rad_value_ptr.lock().unwrap();
            let rad_value_ptr_clone = binding.deref_mut();

            ui.add(egui::Slider::new(rad_value_ptr_clone, 0.0..=15000.0).
                text("Roentgen/hr"));
            if ui.button("Increment").clicked() {
                *rad_value_ptr_clone += 1.0;
            }
            // get rid of mutable ref
            drop(binding);
            // separator and select panel
            ui.separator();
            ui.horizontal( 
                |ui| {
                    ui.selectable_value(&mut self.open_panel, Panel::Simple, "Simple User Input"); 
                    ui.selectable_value(&mut self.open_panel, Panel::InputOutput, "Transfer Fn Simulation"); 
                    ui.selectable_value(&mut self.open_panel, Panel::IsothermalCIET, "CIET Isothermal Simulation"); 
                    ui.selectable_value(&mut self.open_panel, Panel::HeaterV2BareCIET, 
                        "CIET Heater v2 Bare Simulation"); 
            }
            );
            ui.separator();

            // just a test widget, shows it's running i guess

            match self.open_panel {
                Panel::Simple => {
                    self.simple_panel_ui(ui);
                },
                Panel::InputOutput => {
                    self.user_input_output_panel_ui(ui);
                },
                Panel::IsothermalCIET => {
                    self.ciet_isothermal_panel_ui(ui);
                }
                Panel::HeaterV2BareCIET => {
                    self.ciet_heater_bare_panel_ui(ui);
                }
            }
            
            ui.separator();

            ui.add(egui::github_link_file!(
                "https://gitlab.com/theodore_ong/eframe_gui_opcua_client",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        ctx.request_repaint();
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}


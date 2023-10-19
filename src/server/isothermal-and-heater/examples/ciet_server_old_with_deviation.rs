
#[warn(missing_docs)]
use opcua::server::prelude::*;
use local_ip_address::local_ip;
use opcua::server::config;

use thermal_hydraulics_rs::prelude::alpha_nightly::*;
use uom::si::power::kilowatt;

use super::ciet_functions_for_deviation_calcs::*;
use std::{time::{Instant, SystemTime}, sync::{Arc, Mutex}};
use crate::heater::{*, struct_supports::StructuralSupport};
//use opcua::server::address_space;

/// In this example, we use the legacy ciet server codes used in maturin
/// to generate the results
pub fn construct_and_run_ciet_server(run_server: bool){

    let mut server = build_standard_server();

    let ns = {
        let address_space = server.address_space();
        let mut address_space = address_space.write();
        address_space
            .register_namespace("urn:simple-server")
            .unwrap()
    };

    // I'll have 4 variables here
    // note that each variable needs a separate node ID
    // this is how the user will interact with ciet: through these
    // flowrates and the pump pressure

    let ctah_branch_mass_flowrate_node = NodeId::new(ns, "ctah_branch_mass_flowrate");
    let heater_branch_mass_flowrate_node = NodeId::new(ns, "heater_branch_flowrate");
    let dhx_branch_mass_flowrate_node = NodeId::new(ns, "dhx_branch_flowrate");
    let ctah_pump_pressure_node = NodeId::new(ns, "ctah_pump_pressure");

    // now for the CIET Heater, I'll a few more nodes. I need 
    // at least the BT-11 temperature (heater inlet),
    // BT-12 temperature (heater outlet),
    // and heater power, 
    //
    // I will not be recording surface temperatures for now
    // nor am I using the csv writer just yet
    // The user should be able to adjust bt11_temperature and 
    // heater power, 
    // and consequently, observe bt12_temperature
    let bt11_temperature_node = NodeId::new(ns, "bt11_temperature_degC");
    let bt12_temperature_node = NodeId::new(ns, "bt12_temperature_degC");
    let heater_power_node = NodeId::new(ns, "heater_power_kilowatts");



    // I'll have another two here to close off the Heater and DHX branch respectively

    let heater_branch_valve_node = NodeId::new(ns, "heater_branch_valve_open");
    let dhx_branch_valve_node = NodeId::new(ns, "dhx_branch_valve_open");
    let ctah_branch_valve_node = NodeId::new(ns, "ctah_branch_valve_open");

    // Here are an additional 3 variables for calculation time
    let calculation_time_node = NodeId::new(ns, "calculation_time");
    let initiation_time_node = NodeId::new(ns, "ciet_obj_construction_time");
    let total_calc_time_node = NodeId::new(ns, "construction_time_plus_calc_time");

    // And then some more variables for 
    // (1) manometer reading error
    // (2) loop pressure drop error due to flowrate error of 2\%
    // (3) fldk error
    // (4) total error (sqrt sum of them)
    
    let manometer_reading_error_pascals_node 
        = NodeId::new(ns, "manometer_reading_error_pascals");
    let loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals_node
        = NodeId::new(ns, "loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals");
    let loop_pressure_drop_error_due_to_fldk_pascals_node
        = NodeId::new(ns, "loop_pressure_drop_error_due_to_fldk_pascals");
    let loop_pressure_drop_error_total_node
        = NodeId::new(ns, "loop_pressure_drop_error_total");



    let address_space = server.address_space();

    // this part is responsible for sensor data
    {
        let mut address_space = address_space.write();

        // Create a sample folder under objects folder
        let sample_folder_id = address_space
            .add_folder("sensor data", "sensor data", &NodeId::objects_folder_id())
            .unwrap();

        // Add some variables to our sample folder. Values will be overwritten by the timer
        let _ = address_space.add_variables(
            vec![
                Variable::new(&ctah_branch_mass_flowrate_node, 
                              "ctah_branch_mass_flowrate_kg_per_s_FM40", 
                              "ctah_branch_mass_flowrate_kg_per_s_FM40", 0 as f64),
                Variable::new(&heater_branch_mass_flowrate_node, 
                              "heater_branch_mass_flowrate_kg_per_s", 
                              "heater_branch_mass_flowrate_kg_per_s", 0 as f64),
                Variable::new(&dhx_branch_mass_flowrate_node, 
                              "dhx_branch_mass_flowrate_kg_per_s_FM20", 
                              "dhx_branch_mass_flowrate_kg_per_s_FM20", 0 as f64),
                Variable::new(&calculation_time_node, 
                              "calculation_time_ms", 
                              "calculation_time_ms", 0 as f64),
                Variable::new(&initiation_time_node, 
                              "ciet_obj_construction_time_ms", 
                              "ciet_obj_construction_time_ms", 0 as f64),
                Variable::new(&total_calc_time_node, 
                              "construction_time_plus_calc_time_ms", 
                              "construction_time_plus_calc_time_ms", 0 as f64),
                Variable::new(&bt12_temperature_node, 
                "bt12_temperature_degC_heater_outlet", 
                "bt12_temperature_degC_heater_outlet", 
                79.12 as f64),
            ],
            &sample_folder_id,
        );
    }

    // this part is responsible for errors of pressure drop
    {
        let mut address_space = address_space.write();

        // Create a sample folder under objects folder
        let sample_folder_id = address_space
            .add_folder("deviation and error", "deviation and error", &NodeId::objects_folder_id())
            .unwrap();

        // Add some variables to our sample folder. Values will be overwritten by the timer
        let _ = address_space.add_variables(
            vec![
                Variable::new(&manometer_reading_error_pascals_node, 
                              "manometer_reading_error_pascals", 
                              "manometer_reading_error_pascals", 0 as f64),
                Variable::new(&loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals_node, 
                              "loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals", 
                              "loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals", 0 as f64),
                Variable::new(&loop_pressure_drop_error_due_to_fldk_pascals_node, 
                              "loop_pressure_drop_error_due_to_fldk_pascals", 
                              "loop_pressure_drop_error_due_to_fldk_pascals", 0 as f64),
                Variable::new(&loop_pressure_drop_error_total_node, 
                              "loop_pressure_drop_error_total", 
                              "loop_pressure_drop_error_total", 0 as f64),
            ],
            &sample_folder_id,
        );
    }

    // this is the piece of code for the writeonly variable
    // we can use booleans or floats
    {
        let mut address_space = address_space.write();
        let folder_id = address_space
            .add_folder("Controller", "Controller", &NodeId::objects_folder_id())
            .unwrap();


        VariableBuilder::new(&ctah_pump_pressure_node, 
                             "ctah_pump_pressure_pa", "ctah_pump_pressure_pa")
            .data_type(DataTypeId::Float)
            .value(0 as f64)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);

        VariableBuilder::new(&heater_branch_valve_node,
                             "heater_branch_valve_open", "heater_branch_valve_open")
            .data_type(DataTypeId::Boolean)
            .value(true as bool)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);

        VariableBuilder::new(&dhx_branch_valve_node,
                             "dhx_branch_valve_open", "dhx_branch_valve_open")
            .data_type(DataTypeId::Boolean)
            .value(true as bool)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);

        VariableBuilder::new(&ctah_branch_valve_node,
                             "ctah_branch_valve_open", "ctah_branch_valve_open")
            .data_type(DataTypeId::Boolean)
            .value(true as bool)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);

        VariableBuilder::new(&bt11_temperature_node, 
            "bt11_temperature_degC_heater_inlet", 
            "bt11_temperature_degC_heater_inlet")
            .data_type(DataTypeId::Float)
            .value(79.12 as f64)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);

        VariableBuilder::new(&heater_power_node, 
            "heater_power_kilowatts", 
            "heater_power_kilowatts")
            .data_type(DataTypeId::Float)
            .value(8.0 as f64)
            .writable()
            .organized_by(&folder_id)
            .insert(&mut address_space);
    }




    // adding functions to ciet's server now...
    //
    // this one prints the endpoint every 5s so the user knows
    // how to connect to ciet

    let print_endpoint_simple = || {
        let ip_add = get_ip_as_str();

        println!("\n opc.tcp://{}:{}{} \n",ip_add,4840,CUSTOM_ENDPOINT_PATH);
    };


    //server.add_polling_action(5000, print_endpoint);
    server.add_polling_action(5000, print_endpoint_simple);


    // we need to prepare transmitters and receivers for the
    // ciet isothermal facility

    //let (tx, rx) = mpsc::channel();


    // now this algorithm is REALLY inefficient, i am instantiating CIET at
    // EVERY timestep in addition to calculation
    //
    // but if it works, it works

    // clone address space for ciet loop
    let address_space_clone = address_space.clone();
    let calculate_flowrate_and_pressure_loss = move || {

        // construct CIET
        let start_of_object_init = Instant::now();
        let initiation_duration = start_of_object_init.elapsed();

        let start_of_calc_time = Instant::now();

        let mut address_space_lock = address_space_clone.write();
        
        // step 1, find the correct node object for 
        // pump pressure and the
        // boolean for valve control open or close
        let ctah_pump_node = ctah_pump_pressure_node.clone();
        let pump_pressure_value = address_space_lock.
            get_variable_value(ctah_pump_node).unwrap();
        let pump_pressure_value: f64 = pump_pressure_value.
            value.unwrap().as_f64().unwrap();

        // now for heater valve, ctah valve and dhx valve
        // control
        let heater_valve_open = address_space_lock.
            get_variable_value(heater_branch_valve_node.clone()).unwrap();
        let heater_valve_open = 
            heater_valve_open.value.unwrap();

        // this is an opcua Variant::Boolean 
        // we can use a match statement to extract true or false values
        // kind of a clunky way but it can work

        fn match_true_false(opcua_bool: Variant) -> bool{

            match opcua_bool {
                Variant::Boolean(true) => return true,
                Variant::Boolean(false) => return false,
                // for all other types, throw an error,
                _ => panic!("value must be true or false"),
            }

        }

        let heater_valve_open: bool =
            match_true_false(heater_valve_open);
        

        let dhx_valve_open = address_space_lock.
            get_variable_value(dhx_branch_valve_node.clone()).unwrap().value.unwrap();
        let dhx_valve_open:bool = match_true_false(dhx_valve_open);

        let ctah_valve_open = address_space_lock.
            get_variable_value(ctah_branch_valve_node.clone()).unwrap().value.unwrap();
        let ctah_valve_open:bool = match_true_false(ctah_valve_open);
        

        let ciet_temp_deg_c: f64 = 20.0;
        // step 2 calculate mass flowrate for ctah,
        // heater and dhx branch
        let (ctah_branch_flowrate,
             ctah_branch_pressure_change) = 
            get_ciet_isothermal_mass_flowrate(
                pump_pressure_value,
                ciet_temp_deg_c,
                dhx_valve_open,
                heater_valve_open,
                ctah_valve_open
                );

        let heater_branch_flowrate = 
            get_heater_branch_mass_flowrate(
                ctah_branch_pressure_change.value,
                ciet_temp_deg_c,
                heater_valve_open);

        let dhx_branch_flowrate = 
            get_dhx_branch_mass_flowrate(
                ctah_branch_pressure_change.value,
                ciet_temp_deg_c,
                dhx_valve_open);

        // step 3, calc time
        let calc_time = start_of_calc_time.elapsed();


        let calc_time_taken_milleseconds: u16 = 
            calc_time.as_millis().try_into().unwrap();

        // step 4, update values into nodes
        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            calculation_time_node.clone(), 
            calc_time_taken_milleseconds as f64,
            &now, 
            &now);

        let initiation_time_taken_millseconds: u16 =
            initiation_duration.as_millis().try_into().unwrap();

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            initiation_time_node.clone(), 
            initiation_time_taken_millseconds as f64,
            &now, 
            &now);
        let total_time_taken: u16 =
            calc_time_taken_milleseconds + initiation_time_taken_millseconds;

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            total_calc_time_node.clone(), 
            total_time_taken as f64,
            &now, 
            &now);

        
        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            ctah_branch_mass_flowrate_node.clone(), 
            ctah_branch_flowrate as f64,
            &now, 
            &now);

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            heater_branch_mass_flowrate_node.clone(), 
            heater_branch_flowrate as f64,
            &now, 
            &now);

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            dhx_branch_mass_flowrate_node.clone(), 
            dhx_branch_flowrate as f64,
            &now, 
            &now);

        // step 5, calculate errors and print

        //(1) 2\% flowrate error
        //let two_percent_flowrate_error_ctah_heater_only_flow = 
        //    get_loop_pressure_drop_error_due_to_flowmeter_ctah_heater(
        //        MassRate::new::<kilogram_per_second>(ctah_branch_flowrate),
        //        Pressure::new::<pascal>(pump_pressure_value),
        //        0.02);

        let two_percent_flowrate_error_ctah_heater_only_flow = 
            parameterically_estimate_ctah_loop_pressure_drop_error_due_to_flowrate(
                MassRate::new::<kilogram_per_second>(ctah_branch_flowrate), 
                Pressure::new::<pascal>(pump_pressure_value), 
                heater_valve_open, 
                dhx_valve_open, 
                ctah_valve_open, 
                20.0, // temperature degrees C
                0.02);


        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            loop_pressure_drop_error_due_to_coriolis_flowmeter_pascals_node.clone(), 
            two_percent_flowrate_error_ctah_heater_only_flow.value as f64,
            &now, 
            &now);

        //(2) 14.7 Pa manometer error
        let manometer_reading_error_pascals = 
            get_manometer_reading_error_pascals();

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            manometer_reading_error_pascals_node.clone(), 
            manometer_reading_error_pascals.value as f64,
            &now, 
            &now);

        //(3) 10\% fldk error

        let mut fldk_error_pascals_squared = 
            get_fldk_error_pascals_ctah_branch(
                MassRate::new::<kilogram_per_second>(ctah_branch_flowrate),
                0.10)
            * get_fldk_error_pascals_ctah_branch(
                MassRate::new::<kilogram_per_second>(ctah_branch_flowrate),
                0.10);

        // if only CTAH and heater branch open add the heater branch error

        if ctah_valve_open && heater_valve_open {
            fldk_error_pascals_squared += get_fldk_error_pascals_heater_branch(
                MassRate::new::<kilogram_per_second>(heater_branch_flowrate),
                0.10)
            * get_fldk_error_pascals_heater_branch(
                MassRate::new::<kilogram_per_second>(heater_branch_flowrate),
                0.10);

        }
        // if and only if ctah and dhx branch valve open,
        // then add the dhx branch errors

        if ctah_valve_open && dhx_valve_open {
            fldk_error_pascals_squared += get_fldk_error_pascals_dhx_branch(
                MassRate::new::<kilogram_per_second>(dhx_branch_flowrate),
                0.10)
            * get_fldk_error_pascals_dhx_branch(
                MassRate::new::<kilogram_per_second>(dhx_branch_flowrate),
                0.10);

        }

        let fldk_error_pascals = 
            fldk_error_pascals_squared.sqrt();

        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            loop_pressure_drop_error_due_to_fldk_pascals_node.clone(), 
            fldk_error_pascals.value as f64,
            &now, 
            &now);

        //(4) total error

        let total_pressure_error_estimate_pascals_squared = 
            two_percent_flowrate_error_ctah_heater_only_flow * 
            two_percent_flowrate_error_ctah_heater_only_flow
            + manometer_reading_error_pascals *
            manometer_reading_error_pascals
            + fldk_error_pascals_squared;

        let total_pressure_error_estimate = 
            total_pressure_error_estimate_pascals_squared.sqrt();


        let now = DateTime::now();
        let _ = address_space_lock.set_variable_value(
            loop_pressure_drop_error_total_node.clone(), 
            total_pressure_error_estimate.value as f64,
            &now, 
            &now);


        // i think we are done!


    };

    server.add_polling_action(500, calculate_flowrate_and_pressure_loss);

    // to check if polling server adds the polling time to
    // the execution time
    // i will get it to run every 2500 ms (2.5s)
    // and sleep will be 2.5s
    //
    // if it adds to the polling time, it will print as often as
    // the endpoint prints (every 5s)
    // otherwise it will print twice as often
    //
    // the second polling action is the ciet heater code

    // first, initial conditions and timestep
    let timestep = Time::new::<uom::si::time::millisecond>(15.0);
    let initial_temperature: ThermodynamicTemperature = 
    ThermodynamicTemperature::new::<degree_celsius>(79.12);
    let inlet_temperature = initial_temperature;
    let ambient_air_temp: ThermodynamicTemperature = 
    ThermodynamicTemperature::new::<degree_celsius>(21.67);

    // next, heater nodalisation
    let number_of_inner_temperature_nodes: usize = 6;


    let heater_v2_bare_shared_ptr = Arc::new(Mutex::new(
        HeaterVersion2Bare::new_dewet_model(
        initial_temperature,
        ambient_air_temp,
        number_of_inner_temperature_nodes
    )));

    let heater_top_head_bare_shared_ptr
    = Arc::new(Mutex::new(
        HeaterTopBottomHead::new_top_head(
            initial_temperature,
            ambient_air_temp)));

    let heater_bottom_head_bare_shared_ptr
    = Arc::new(Mutex::new(
        HeaterTopBottomHead::new_bottom_head(
        initial_temperature,
        ambient_air_temp)));

    // static mixers
    let static_mixer_mx_10_object_shared_ptr
    = Arc::new(Mutex::new(
        StaticMixerMX10::new_static_mixer(
        initial_temperature,
        ambient_air_temp)));

    let static_mixer_mx_10_pipe_shared_ptr
    = Arc::new(Mutex::new(StaticMixerMX10::new_static_mixer_pipe(
        initial_temperature,
        ambient_air_temp)));

    // structural support
    let struct_support_equiv_diameter: Length = Length::new::<inch>(0.5);
    let struc_support_equiv_length: Length = Length::new::<uom::si::length::foot>(1.0);


    let structural_support_heater_top_head_shared_ptr = 
    Arc::new(Mutex::new(
    StructuralSupport::new_steel_support_cylinder(
        struc_support_equiv_length,
        struct_support_equiv_diameter,
        initial_temperature,
        ambient_air_temp)));

    let structural_support_heater_bottom_head = 
    structural_support_heater_top_head_shared_ptr.lock().unwrap().clone();

    let structural_support_heater_bottom_head_shared_ptr = 
    Arc::new(Mutex::new(structural_support_heater_bottom_head));

    let structural_support_mx_10 = 
    structural_support_heater_top_head_shared_ptr.lock().unwrap().clone();

    let structural_support_mx_10_shared_ptr = 
    Arc::new(Mutex::new(structural_support_mx_10));

    let inlet_bc_shared_ptr: Arc<Mutex<HeatTransferEntity>> 
    = Arc::new(Mutex::new(BCType::new_const_temperature( 
        inlet_temperature).into()));

    let outlet_bc_shared_ptr: Arc<Mutex<HeatTransferEntity>> = 
    Arc::new(Mutex::new(
        BCType::new_adiabatic_bc().into()));

    let approx_support_conductance: ThermalConductance = 
    structural_support_heater_top_head_shared_ptr.lock().unwrap()
        .get_axial_node_to_bc_conductance();

    let ambient_air_temp_bc_shared_ptr: 
    Arc<Mutex<HeatTransferEntity>> = Arc::new(Mutex::new(
        inlet_bc_shared_ptr.lock().unwrap().clone()
    ));
    // struct support conductance assumed constant
    // kind of negligible so doesn't matter
    let support_conductance_interaction = HeatTransferInteractionType::
        UserSpecifiedThermalConductance(approx_support_conductance);

    // mass flowrate constant
    let mass_flowrate = MassRate::new::<kilogram_per_second>(0.18);
    // main loop for ciet heater

    let loop_time = SystemTime::now();
    let ciet_heater_loop = move || {
        // timer start 
        let loop_time_start = loop_time.elapsed().unwrap();

        // bcs 


        // create interactions 


        // let's get heater temperatures for post processing
        // as well as the interaction
        // for simplicity, i use the boussineseq approximation,
        // which assumes that heat transfer is governed by 
        // average density (which doesn't change much for liquid 
        // anyway)


        // this is needed for heated section 
        // bulk outlet temperature
        let mut therminol_array_clone: FluidArray 
        = heater_v2_bare_shared_ptr.lock().unwrap().
            therminol_array.clone().try_into().unwrap();


        let heater_fluid_bulk_temp: ThermodynamicTemperature = 
        therminol_array_clone.try_get_bulk_temperature().unwrap();

        // this is needed for heater surface temperatures
        let _heater_surface_array_clone: SolidColumn 
        = heater_v2_bare_shared_ptr.lock() 
        .unwrap().steel_shell.clone().try_into().unwrap();


        // heater top head exit temperature for comparison
        let heater_top_head_bare_therminol_clone: FluidArray = 
        heater_top_head_bare_shared_ptr.lock().unwrap()
            .therminol_array.clone().try_into().unwrap();

        let _heater_top_head_exit_temperature: ThermodynamicTemperature = 
        heater_top_head_bare_therminol_clone.get_temperature_vector()
            .unwrap().into_iter().last().unwrap();

        // BT-12: static mixer outlet temperature

        let static_mixer_therminol_clone: FluidArray = 
        static_mixer_mx_10_object_shared_ptr.
            lock().unwrap().therminol_array.clone().try_into().unwrap();

        let _static_mixer_exit_temperature: ThermodynamicTemperature
        = static_mixer_therminol_clone.get_temperature_vector().unwrap()
            .into_iter().last().unwrap();

        let static_mixer_pipe_therminol_clone: FluidArray = 
        static_mixer_mx_10_pipe_shared_ptr.lock().unwrap()
            .therminol_array.clone().try_into().unwrap();


        // for advection interactions, because I assume boussineseq 
        // approximations, I'll just take the average density 
        // and use it for enthalpy transfer calculations
        let heater_therminol_avg_density: MassDensity = 
        LiquidMaterial::TherminolVP1.density(
            heater_fluid_bulk_temp).unwrap();

        let generic_advection_interaction = 
        HeatTransferInteractionType::new_advection_interaction(
            mass_flowrate,
            heater_therminol_avg_density,
            heater_therminol_avg_density,
        );
        // calculation steps, read bt11_temperature_degC and 
        // heater power from opc-ua input
        let heater_power: Power;
        let heater_inlet_temp: ThermodynamicTemperature;
        {
            let address_space_lock = address_space.write();
            let bt11_user_input_value_deg_c = address_space_lock.
                get_variable_value(
                    bt11_temperature_node.clone())
                .unwrap().value.unwrap()
                .as_f64().unwrap();

            heater_inlet_temp = ThermodynamicTemperature::new::
                <degree_celsius>(bt11_user_input_value_deg_c);

            let user_set_inlet_bc: HeatTransferEntity = 
            BCType::new_const_temperature( 
                heater_inlet_temp).into();
            // change inlet bc ptr 

            inlet_bc_shared_ptr.lock().unwrap().set(
                user_set_inlet_bc).unwrap();


            let heater_user_input_value_kilowatts = address_space_lock.
                get_variable_value(
                    heater_power_node.clone())
                .unwrap().value.unwrap()
                .as_f64().unwrap();

            heater_power = Power::new::<kilowatt>(
                heater_user_input_value_kilowatts);
        }

        // postprocessing, print out temperature sensors
        {
            let bt_12_temperature: ThermodynamicTemperature = 
            static_mixer_pipe_therminol_clone.get_temperature_vector().unwrap() 
                .into_iter().last().unwrap();

            // get bt_12_temperature in degrees c rounded to 1
            // decimal place
            let bt12_temperature_deg_c: f64 = 
            (bt_12_temperature.get::<degree_celsius>()*10.0)
            .round()
            /10.0;

            // set bt12 temperature node
            let mut address_space_lock = address_space.write();
            let now = DateTime::now();
            let _ = address_space_lock.set_variable_value(
                bt12_temperature_node.clone(), 
                bt12_temperature_deg_c as f64,
                &now, 
                &now);
        }

        // make axial connections to BCs 
        
        heater_bottom_head_bare_shared_ptr.lock().unwrap()
            .therminol_array.link_to_back(
            &mut inlet_bc_shared_ptr.lock().unwrap(),
            generic_advection_interaction
        ).unwrap();

        heater_v2_bare_shared_ptr.lock().unwrap().therminol_array.link_to_back(
            &mut heater_bottom_head_bare_shared_ptr.lock().unwrap().therminol_array,
            generic_advection_interaction
        ).unwrap();

        heater_v2_bare_shared_ptr.lock().unwrap().therminol_array.link_to_front(
            &mut heater_top_head_bare_shared_ptr.lock().unwrap().therminol_array,
            generic_advection_interaction
        ).unwrap();


        heater_top_head_bare_shared_ptr.lock().unwrap().therminol_array.link_to_front(
            &mut static_mixer_mx_10_object_shared_ptr.lock().unwrap().therminol_array,
            generic_advection_interaction
        ).unwrap();

        static_mixer_mx_10_object_shared_ptr.lock().unwrap().therminol_array.link_to_front(
            &mut static_mixer_mx_10_pipe_shared_ptr.lock().unwrap().therminol_array,
            generic_advection_interaction
        ).unwrap();

        static_mixer_mx_10_pipe_shared_ptr.lock().unwrap().therminol_array.link_to_front(
            &mut outlet_bc_shared_ptr.lock().unwrap(),
            generic_advection_interaction
        ).unwrap();

        // lateral connections without thread spawning 

        heater_v2_bare_shared_ptr.lock().unwrap().
            lateral_and_miscellaneous_connections(
                mass_flowrate,
                heater_power);

        heater_bottom_head_bare_shared_ptr.lock().unwrap(). 
            lateral_and_miscellaneous_connections(
                mass_flowrate);

        heater_top_head_bare_shared_ptr.lock().unwrap()
            .lateral_and_miscellaneous_connections(
                mass_flowrate);


        static_mixer_mx_10_object_shared_ptr.lock().unwrap().
            lateral_and_miscellaneous_connections(
                mass_flowrate);

        static_mixer_mx_10_pipe_shared_ptr.lock().unwrap().
            lateral_and_miscellaneous_connections(
            mass_flowrate);


        // link struct supports to ambient air
        // axially 
        structural_support_heater_bottom_head_shared_ptr.lock().unwrap(). 
            support_array.link_to_front(
                &mut ambient_air_temp_bc_shared_ptr.lock().unwrap(),
                support_conductance_interaction
            ).unwrap();

        structural_support_heater_top_head_shared_ptr.lock().unwrap(). 
            support_array.link_to_front(
                &mut ambient_air_temp_bc_shared_ptr.lock().unwrap(),
                support_conductance_interaction
            ).unwrap();

        structural_support_mx_10_shared_ptr.lock().unwrap()
            .support_array.link_to_front(
                &mut ambient_air_temp_bc_shared_ptr.lock().unwrap(),
            support_conductance_interaction
        ).unwrap();

        // link struct supports to heater top/bottom heads
        structural_support_heater_top_head_shared_ptr.lock().unwrap().
            support_array.link_to_back(
                &mut heater_top_head_bare_shared_ptr.lock().unwrap().steel_shell,
                support_conductance_interaction
            ).unwrap();
        structural_support_heater_bottom_head_shared_ptr.lock().unwrap(). 
            support_array.link_to_back(
                &mut heater_bottom_head_bare_shared_ptr.lock().unwrap().steel_shell,
                support_conductance_interaction
            ).unwrap();

        structural_support_mx_10_shared_ptr.lock().unwrap().support_array.link_to_back(
            &mut static_mixer_mx_10_pipe_shared_ptr.lock().unwrap().steel_shell,
            support_conductance_interaction
        ).unwrap();

        // note, the heater top and bottom head area changed 
        // during course of this interaction, so should be okay


        // i will also connect heater shell to the structural support 
        // via the head as in ciet 

        heater_v2_bare_shared_ptr.lock().unwrap().steel_shell.link_to_back(
            &mut heater_bottom_head_bare_shared_ptr.lock().unwrap().steel_shell,
            support_conductance_interaction
        ).unwrap();

        heater_v2_bare_shared_ptr.lock().unwrap().steel_shell.link_to_front(
            &mut heater_top_head_bare_shared_ptr.lock().unwrap().steel_shell,
            support_conductance_interaction
        ).unwrap();

        // probably edit this to include twisted tape conductance
        heater_v2_bare_shared_ptr.lock().unwrap().twisted_tape_interior.link_to_back(
            &mut heater_bottom_head_bare_shared_ptr.lock().unwrap().twisted_tape_interior,
            support_conductance_interaction
        ).unwrap();

        heater_v2_bare_shared_ptr.lock().unwrap().twisted_tape_interior.link_to_front(
            &mut heater_top_head_bare_shared_ptr.lock().unwrap().twisted_tape_interior,
            support_conductance_interaction
        ).unwrap();

        // now link it laterally to ambient temperatures
        structural_support_heater_top_head_shared_ptr.lock().unwrap().
            lateral_and_miscellaneous_connections();
        structural_support_heater_bottom_head_shared_ptr.lock().unwrap(). 
            lateral_and_miscellaneous_connections();

        structural_support_mx_10_shared_ptr.lock().unwrap().
            lateral_and_miscellaneous_connections();

        // advance timesteps
        heater_v2_bare_shared_ptr.lock().unwrap().
            advance_timestep(timestep);

        heater_bottom_head_bare_shared_ptr.lock().unwrap(). 
            advance_timestep(
                timestep);

        heater_top_head_bare_shared_ptr.lock().unwrap()
            .advance_timestep(timestep);

        static_mixer_mx_10_object_shared_ptr.lock().unwrap()
            .advance_timestep(timestep);

        static_mixer_mx_10_pipe_shared_ptr.lock().unwrap().advance_timestep(
            timestep);


        structural_support_heater_bottom_head_shared_ptr.lock().unwrap().
            advance_timestep(timestep);
        structural_support_heater_top_head_shared_ptr.lock().unwrap().
            advance_timestep(timestep);

        structural_support_mx_10_shared_ptr.lock().unwrap().advance_timestep(
            timestep);

        // that's it!


        let _time_taken_for_calculation_loop = loop_time.elapsed().unwrap()
        - loop_time_start;
        // probably want to add a node for heater loop time taken

        //dbg!(time_taken_for_calculation_loop);
    };

    
    server.add_polling_action(
        timestep.get::<uom::si::time::millisecond>().round() as u64, 
        ciet_heater_loop);


    if run_server { server.run(); }

}

const CUSTOM_ENDPOINT_PATH: &str = "/rust_ciet_opcua_server";
fn build_standard_server() -> Server {

    let server_builder = ServerBuilder::new();

    let server_builder = 
        server_builder.application_name("test server_builder");

    let server_builder =
        server_builder.application_uri("urn:OPC UA Sample Server");




    let ip_address = get_ip_as_str();

    let server_builder = 
        server_builder.host_and_port(&ip_address, 4840);


    let server_builder =
        server_builder.discovery_urls(
            vec![
            CUSTOM_ENDPOINT_PATH.into(),
            ]);


    // username and password is just anonymous

    let user_id_anonymous = config::ANONYMOUS_USER_TOKEN_ID;


    let user_id_vector = 
        vec![user_id_anonymous]
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<String>>();




    let path = CUSTOM_ENDPOINT_PATH;


    let my_endpoints = vec![
        ("custom_path", ServerEndpoint::new_none(path,&user_id_vector)),
    ];


    let server_builder = 
        server_builder.endpoints(my_endpoints);

    // then we build the server

    let server = server_builder.server().unwrap();
    return server;

}

fn get_ip_as_str() -> String {

    let my_local_ip = local_ip().unwrap();

    // i can convert it to a string

    let ip_add_string : String = my_local_ip.to_string();

    return ip_add_string;

}




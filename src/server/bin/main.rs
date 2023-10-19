pub mod ciet_libraries;
pub use ciet_libraries::*;


pub mod examples;
pub use examples::ciet_server_old_with_deviation;

fn main() {
    let run_server = true;
    ciet_server_old_with_deviation::construct_and_run_ciet_server(run_server);
}


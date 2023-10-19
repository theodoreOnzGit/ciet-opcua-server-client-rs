pub mod ciet_libraries;
pub use ciet_libraries::*;


pub mod examples;
pub use examples::ciet_server_old_with_deviation;

/// The server code here is constructed using the thermal hydraulics 
/// library
/// The thermal_hydraulics_rs library was constructed with CIET 
/// in mind.
/// This is the compact integral effects test from the UC Berkeley 
/// Thermal Hydraulics Lab. It is a Library which contains useful 
/// traits and methods for thermal hydraulics calculations.
/// This crate has heavy reliance on units of measure (uom) released under 
/// Apache 2.0 license. So you'll need to get used to unit safe calculations
/// with uom as well.
///
///
/// This ciet-opcua-server-client-rs binary and 
/// thermal_hydraulics_rs library was initially developed for 
/// use in my PhD thesis under supervision 
/// of Professor Per F. Peterson. thermal_hydraulics_rs is 
/// a thermal hydraulics
/// library in Rust that is released under the GNU General Public License
/// v 3.0. This is partly due to the fact that some of the libraries 
/// inherit from GeN-Foam and OpenFOAM, both licensed under GNU General
/// Public License v3.0.
/// As such, the entire library and all binaries (server and client) 
/// is released under GNU GPL v3.0. It is a strong 
/// copyleft license which means you cannot use it in proprietary software 
/// unless you release the source code under GNU GPL v3.0.
///
///
/// License
///
///    This is a thermal hydraulics server demonstration written 
///    in rust meant to help with the
///    fluid mechanics and heat transfer aspects of the calculations
///    for the Compact Integral Effects Tests (CIET) and hopefully 
///    Gen IV Reactors such as the Fluoride Salt cooled High Temperature 
///    Reactor (FHR). 
///     
///    Copyright (C) 2022-2023  Theodore Kay Chen Ong, Singapore Nuclear
///    Research and Safety Initiative, Per F. Peterson, University of 
///    California, Berkeley Thermal Hydraulics Laboratory
///
///    ciet-opcua-server-client-rs is free software; you can 
///    redistribute it and/or modify it
///    under the terms of the GNU General Public License as published by the
///    Free Software Foundation; either version 2 of the License, or (at your
///    option) any later version.
///
///    ciet-opcua-server-client-rs is distributed in the hope 
///    that it will be useful, but WITHOUT
///    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
///    FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
///    for more details.
///
///    This thermal hydraulics library and ciet-opcua-server-client-rs 
///    binaries
///    contains some code copied from GeN-Foam, and OpenFOAM derivative.
///    This offering is not approved or endorsed by the OpenFOAM Foundation nor
///    OpenCFD Limited, producer and distributor of the OpenFOAM(R)software via
///    www.openfoam.com, and owner of the OPENFOAM(R) and OpenCFD(R) trademarks.
///    Nor is it endorsed by the authors and owners of GeN-Foam.
///
///    You should have received a copy of the GNU General Public License
///    along with this program.  If not, see <http://www.gnu.org/licenses/>.
///
/// © All rights reserved. Theodore Kay Chen Ong,
/// Singapore Nuclear Research and Safety Initiative,
/// Per F. Peterson,
/// University of California, Berkeley Thermal Hydraulics Laboratory
///
/// Main author of the code: Theodore Kay Chen Ong, supervised by
/// Professor Per F. Peterson
///
/// Btw, I no affiliation with the Rust Foundation. 
fn main() {
    let run_server = true;
    ciet_server_old_with_deviation::construct_and_run_ciet_server(run_server);
}


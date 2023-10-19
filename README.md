# ciet-opcua-server-client-rs
Compact Integral Effects Test (CIET) OPC-UA Server and Client 

## basic running

You will run either a server or client

For server code, you will need OPC-UA, so have the prerequisites installed 
like openssl and so on...

```bash
cargo run --bin server
```

For client, you will need all libraries for eframe, egui and OPC-UA
```bash
cargo run --bin client
```
## prerequisites

For the server, on the Linux end, you will need openssl and openblas.

For Arch Linux based systems:
```bash
sudo pacman -S openblas
```

For Ubuntu based systems:
```bash
sudo apt install libopenblas-dev
```

For the client, on the windows end, you will also need openssl.
Openssl does not release binaries on its github page. However, 
some people have precompiled binaries online, download at your own risk.




## licenses

There are several crates I've used, including thermal hydraulics rs,
uom and many others. These depend on libraries such as OpenBLAS, 
intel-mkl and other things. All these libraries are open sourced 
and have licenses such as GNU GPL v3, Apache 2.0, MIT, Mozilla Public 
License and so on. The license can be found in the license files folder


I developed this server and client as part of my PhD thesis and used
many free and open source libraries including but not limited to:

1. Units of measure (uom)
2. Peroxide
3. Roots
4. GeN-Foam and OpenFOAM
5. ndarray-linalg and ndarray (which depends on intel-mkl for Windows/MacOS or OpenBLAS 
for Linux machines)
6. egui, egui_plot and eframe
7. local-ip-address 
8. env_logger and log 
9. csv
10. serde

Additionally, to save myself from some pain in creating a GUI, 
I took reference from Andrei Litvin's rs value plotter, and 
Emil Ernerfeldt's eframe template:

1. https://github.com/andy31415/rs-value-plotter
2. https://github.com/emilk/eframe_template



They are usually released under Apache 2.0 and MIT (uom and peroxide)
and roots/OpenBLAS 
is released under BSD 2 clause. The licensing notices
is provided in the licensing file.

This app is released uses a thermal hydraulics
library which has some code taken from GeN-Foam, an OpenFOAM
derivative.
OpenFOAM and GeN-Foam are released under GNU GPL v3.0. 
As I am reliant on these libraries under the GNU GPL v3.0
license, this software is also released under GNU GPL v3.0.





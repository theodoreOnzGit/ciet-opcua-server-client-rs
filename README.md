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
License and so on. The license file is pending.

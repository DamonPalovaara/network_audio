// This file is a modified version of the example project found at:
// github.com/RustAudio/rust-jack/blob/master/examples/playback_capture.rs
// This file is used for live demoing the server/client

extern crate jack;
use std::io;
use std::thread;

mod server;

use server::Server;

fn main() {
    // Create client
    let (client, _status) = jack::Client::new("Network Audio", jack::ClientOptions::NO_START_SERVER).unwrap();

    // Register ports
    let in_0 = client.register_port("net_in_0", jack::AudioIn::default()).unwrap();

    let out_0 = client.register_port("net_out_0", jack::AudioOut::default()).unwrap();
    
    // Create server
    let jack_buf_size  = 1024;
    let sample_rate    = 44100;
    let num_channels   = 2;
    let network_mtu    = 1500;
    let server_address = "192.168.8.13:8000";
    let send_address   = "192.168.8.13:9001";
    let mut network_server = Server::new(
        jack_buf_size, 
        sample_rate, 
        num_channels,
        network_mtu,
        server_address,
        send_address
    );

    let nines = f32::from_be_bytes([9, 9, 9, 9]);
    let jack_buffer = vec![nines; jack_buf_size];
    network_server.fill_buffer(&jack_buffer, 0); 
    network_server.fill_buffer(&jack_buffer, 1);

    // Jack function which is executed in async
    let process_callback_server = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {

        network_server.fill_buffer(in_0.as_slice(ps), 0);
        network_server.send_packets();

        jack::Control::Continue
    };

    // Activate the client, which starts the processing.
    let process = jack::ClosureProcessHandler::new(process_callback_server);    
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn buffer_size(&mut self, _: &jack::Client, sz: jack::Frames) -> jack::Control {
        println!("JACK: buffer size changed to {}", sz);
        jack::Control::Continue
    }

    fn sample_rate(&mut self, _: &jack::Client, sample_rate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", sample_rate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }

    fn latency(&mut self, _: &jack::Client, mode: jack::LatencyType) {
        println!(
            "JACK: {} latency has changed",
            match mode {
                jack::LatencyType::Capture => "capture",
                jack::LatencyType::Playback => "playback",
            }
        );
    }
}
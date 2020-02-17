extern crate network_disco;

use std::thread;
use network_disco::client::Client;
use network_disco::server::Server;

#[cfg(test)]
static TEST_ADDRESS: &str = "127.0.0.1";

#[test]
fn test_client_receive_buffer() {
    
    // Create client
    let server_address = TEST_ADDRESS.to_owned() + ":9002";
    let mut client = Client::new(&server_address);

    // Create server on separate thread
    thread::spawn(|| {

        // Create server
        let jack_buf_size  = 1024;
        let _sample_rate   = 44100;
        let _num_channels  = 2;
        let network_mtu    = 1500;
        let server_address = TEST_ADDRESS.to_owned() + ":8003";
        let send_address   = TEST_ADDRESS.to_owned() + ":9002";
        let mut server = Server::new(
            jack_buf_size, 
            _sample_rate, 
            _num_channels,
            network_mtu,
            &server_address,
            &send_address
        );

        // Fill server buffers
        let nines = f32::from_be_bytes([9, 9, 9, 9]);
        let jack_buffer = vec![nines; jack_buf_size]; 
        server.fill_buffer(&jack_buffer, 0);
        server.fill_buffer(&jack_buffer, 1);

        // Send packets
        loop { server.send_packets(); }
    });   

    // client._prime();

    // // Test that Client is primed correctly
    // assert_eq!(client.get_sample_rate(), 0);
    // assert_eq!(client.get_audio_buf_size(), 1024);
}
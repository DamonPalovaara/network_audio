#[cfg(test)]
static TEST_ADDRESS: &str = "127.0.0.1";

//extern crate jack;
use std::net::UdpSocket;

pub struct Server {
    _sample_rate:     usize,        // Sample rate of JACK server
    _num_channels:    usize,        // Number of channels to use
    udp_payload_size: usize,        // Size of allocated space for each udp buffer
    buffers:          Vec<Vec<u8>>, // Where the udp buffers reside
    num_bufs:         usize,        // Number of udp buffers each channel splits into
    send_address:     String,       // IP address and port in "xxx.xxx.xxx.xxx:xxxx" format
    udp_socket:       UdpSocket,    // UDP socket that sends packets
    count:            Vec<usize>,
    key:              usize,
}

impl Server {

    pub fn new(
        jack_buf_size:  usize, 
        _sample_rate:   usize, 
        _num_channels:  usize, 
        network_mtu:    usize,
        server_address: &str,
        send_address:   &str,
    ) -> Server {
        let num_bufs         = ((jack_buf_size * 4) as f64 / (network_mtu - 100) as f64).ceil() as usize;
        let udp_payload_size = ((jack_buf_size * 4) as f64 / num_bufs as f64).ceil() as usize + 4;
        let raw_buf_size     = (num_bufs * 4) + (jack_buf_size * 4);
        let buffers          = vec![vec![0; raw_buf_size]; _num_channels];
        let udp_socket       = UdpSocket::bind(server_address).unwrap();
        let send_address     = send_address.to_string();
        let count            = vec![0; _num_channels];
        let key              = (256 / num_bufs) * num_bufs;

        Server {
            _sample_rate,
            _num_channels,
            udp_payload_size,
            buffers,
            num_bufs,
            send_address,
            udp_socket,
            count,
            key
        }
    }

    pub fn fill_buffer(&mut self, jack_buf: &[f32], channel_idx: usize) {

        // Convert jack buffer into bytes
        let jack_buf_u8 = jack_buf_to_bytes(&jack_buf);        
        let header = self.generate_header(channel_idx);

        // Turn the buffers into chunk iterators
        let udp_chunks  = self.buffers[channel_idx][..].chunks_mut(self.udp_payload_size);
        let mut jack_chunks = jack_buf_u8.chunks(self.udp_payload_size - 4);        

        // Copy jack buffer into server buffers
        udp_chunks.for_each(|udp_chunk| {            
            udp_chunk[..4].copy_from_slice(&header);
            udp_chunk[4..].copy_from_slice(jack_chunks.next().unwrap());
        });
    }

    // Try to make this function async so Jack can update while this is sending
    pub fn send_packets(&self) {
        for i in 0..self.buffers.len() {
            self.buffers[i].chunks(self.udp_payload_size).for_each(|udp_payload| {
                self.udp_socket.send_to(udp_payload, &self.send_address).unwrap();
            });
        }
    }

    fn generate_header(&mut self, channel_idx: usize) -> [u8; 4] {
        let channel_num = channel_idx as u8;
        let encoded_idx = (self.count[channel_idx] % self.key) as u8;
        let sample_rate = 0u8;
        let num_chunks  = self.num_bufs as u8;

        self.count[channel_idx] += 1;
        
        [channel_num, encoded_idx, sample_rate, num_chunks]
    }

}

struct Client {
    channel_number: usize, // Channel_number | Encoded index | Sample rate | number of chunks
    encoded_index:  usize,
    sample_rate:    usize,
    num_chunks:     usize,
    server_address: String,
    server_socket:  UdpSocket,
}

impl Client {
    fn new(
        server_address: &str,
    ) -> Client { 
        let channel_number = 0;
        let encoded_index = 0;
        let sample_rate = 0;
        let num_chunks = 0;
        let server_address = server_address.to_string();
        let server_socket = UdpSocket::bind(&server_address).unwrap();        

        Client {
            channel_number,
            encoded_index,
            sample_rate,
            num_chunks,
            server_address,
            server_socket,
        }
    }

    fn read_packet(&mut self) {
        let mut buf = vec![0; 2000];
        self.server_socket.recv_from(&mut buf).unwrap();
    }
}

// Casts a f32 slice as a u8 slice
fn jack_buf_to_bytes(jack_buf: &[f32]) -> &[u8] {
    unsafe { 
        std::slice::from_raw_parts(jack_buf.as_ptr() as *const u8, jack_buf.len() * 4)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_server_new() {
        // Create server
        let jack_buf_size  = 1024;
        let _sample_rate   = 44100;
        let _num_channels  = 2;
        let network_mtu    = 1500;
        let server_address = TEST_ADDRESS.to_owned() + ":8000";
        let send_address   = TEST_ADDRESS.to_owned() + ":9001";
        let server = Server::new(
            jack_buf_size, 
            _sample_rate, 
            _num_channels,
            network_mtu,
            &server_address,
            &send_address
        );

        assert_eq!(server.udp_payload_size, 1370);
        assert_eq!(server.buffers[0].len(), 4108);
        assert_eq!(server.key, 255);
    }

    #[test]
    fn test_server_fill_buffer() {
        // Create server
        let jack_buf_size  = 1024;
        let _sample_rate   = 44100;
        let _num_channels  = 2;
        let network_mtu    = 1500;
        let server_address = TEST_ADDRESS.to_owned() + ":8001";
        let send_address   = TEST_ADDRESS.to_owned() + ":9001";
        let mut server = Server::new(
            jack_buf_size, 
            _sample_rate, 
            _num_channels,
            network_mtu,
            &server_address,
            &send_address
        );

        // Fill buffers
        let one = f32::from_be_bytes([1, 1, 1, 1]);
        let jack_buffer = vec![one; jack_buf_size];
        server.fill_buffer(&jack_buffer, 0); 
        server.fill_buffer(&jack_buffer, 1);

        // Test channel 0               
        assert_eq!(0, server.buffers[0][0]);
        assert_eq!(0, server.buffers[0][3]);
        assert_eq!(1, server.buffers[0][4]);
        assert_eq!(1, server.buffers[0][1369]);
        assert_eq!(0, server.buffers[0][1370]);
        assert_eq!(1, server.buffers[0][1374]);
        assert_eq!(1, server.buffers[0][2739]);
        assert_eq!(0, server.buffers[0][2740]);
        assert_eq!(1, server.buffers[0][2744]);
        assert_eq!(1, server.buffers[0][4107]);

        // Test channel 1        
        assert_eq!(0, server.buffers[1][0]);
        assert_eq!(0, server.buffers[1][3]);
        assert_eq!(1, server.buffers[1][4]);
        assert_eq!(1, server.buffers[1][1369]);
        assert_eq!(0, server.buffers[1][1370]);
        assert_eq!(1, server.buffers[1][1374]);
        assert_eq!(1, server.buffers[1][2739]);
        assert_eq!(0, server.buffers[1][2740]);
        assert_eq!(1, server.buffers[1][2744]);
        assert_eq!(1, server.buffers[1][4107]);
    }
    
    // TODO: Test that packets are being received correctly
    #[test]
    fn test_server_send_packets() {
        // Create server
        let jack_buf_size  = 1024;
        let _sample_rate   = 44100;
        let _num_channels  = 2;
        let network_mtu    = 1500;
        let server_address = TEST_ADDRESS.to_owned() + ":8002";
        let send_address   = TEST_ADDRESS.to_owned() + ":9001";
        let mut server = Server::new(
            jack_buf_size, 
            _sample_rate, 
            _num_channels,
            network_mtu,
            &server_address,
            &send_address
        );

        // Fill buffers
        let one = f32::from_be_bytes([1, 1, 1, 1]);
        let jack_buffer = vec![one; jack_buf_size]; 
        server.fill_buffer(&jack_buffer, 0);
        server.fill_buffer(&jack_buffer, 1);

        // Send packets
        server.send_packets();
    }

    #[test]
    fn test_client_new() {
        let client_address = TEST_ADDRESS.to_owned() + ":9001";
        let client = Client::new(&client_address);
    }

    #[test]
    fn test_server_client_handshake() {
        // Create client
        let server_address = TEST_ADDRESS.to_owned() + ":9002";
        let mut client = Client::new(&server_address);

        // Create server
        let jack_buf_size  = 1024;
        let _sample_rate   = 44100;
        let _num_channels  = 2;
        let network_mtu    = 8000;
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
        let ones = f32::from_be_bytes([1, 1, 1, 1]);
        let jack_buffer = vec![ones; jack_buf_size]; 
        server.fill_buffer(&jack_buffer, 0);
        server.fill_buffer(&jack_buffer, 1);

        // Send packets
        server.send_packets();
        
        // Receive packet
        client.read_packet();
    }
}
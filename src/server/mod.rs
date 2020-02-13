use std::net::UdpSocket;

#[cfg(test)]
static TEST_ADDRESS: &str = "127.0.0.1";

pub struct Server {
    _sample_rate:  usize,        // Sample rate of JACK server
    _num_channels: usize,        // Number of channels to use
    payload_size:  usize,        // Size of allocated space for each udp buffer
    buffers:       Vec<Vec<u8>>, // Where the udp buffers reside
    num_bufs:      usize,        // Number of udp buffers each channel splits into
    send_address:  String,       // IP address and port in "xxx.xxx.xxx.xxx:xxxx" format
    udp_socket:    UdpSocket,    // UDP socket that sends packets
    count:         Vec<usize>,   // Count used for generating header
    key:           usize,        // Key used for generating header
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
        let num_bufs     = ((jack_buf_size * 4) as f64 / (network_mtu - 100) as f64).ceil() as usize;
        let payload_size = ((jack_buf_size * 4) as f64 / num_bufs as f64).ceil() as usize + 4;
        let raw_buf_size = (num_bufs * 4) + (jack_buf_size * 4);
        let buffers      = vec![vec![0; raw_buf_size]; _num_channels];
        let udp_socket   = UdpSocket::bind(server_address).unwrap();
        let send_address = send_address.to_string();
        let count        = vec![0; _num_channels];
        let key          = (256 / num_bufs) * num_bufs;

        Server {
            _sample_rate,
            _num_channels,
            payload_size,
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
        let jack_buf_u8 = f32_slice_to_bytes(&jack_buf);

        // Turn the buffers into chunk iterators
        let headers: Vec<[u8; 4]> = (0..self.num_bufs).map(|_| self.generate_header(channel_idx)).collect();
        let mut udp_chunks        = self.buffers[channel_idx][..].chunks_mut(self.payload_size);
        let mut jack_chunks       = jack_buf_u8.chunks(self.payload_size - 4);

        // Copy a header and jack chunk into each udp chunk
        for i in 0..self.num_bufs {
            let udp_chunk = udp_chunks.next().unwrap();
            udp_chunk[..4].copy_from_slice(&headers[i]);
            udp_chunk[4..].copy_from_slice(jack_chunks.next().unwrap());
        }
    }

    // Try to make this function async so Jack can update while this is sending
    pub fn send_packets(&self) {
        for i in 0..self.buffers.len() {
            self.buffers[i].chunks(self.payload_size).for_each(|udp_payload| {
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

    #[cfg(test)]
    pub fn _read_buffer(&self, channel_idx: usize) -> &[u8] {
        &self.buffers[channel_idx]
    }
}

// Casts a f32 slice as a u8 slice
fn f32_slice_to_bytes(jack_buf: &[f32]) -> &[u8] {
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

        assert_eq!(server.payload_size, 1370);
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
        let nines = f32::from_be_bytes([9, 9, 9, 9]);
        let jack_buffer = vec![nines; jack_buf_size];
        server.fill_buffer(&jack_buffer, 0); 
        server.fill_buffer(&jack_buffer, 1);

        // Test channel 0                 
        assert_eq!(0, server._read_buffer(0)[0   ]); // 1st header 
        assert_eq!(0, server._read_buffer(0)[1   ]);
        assert_eq!(0, server._read_buffer(0)[2   ]);
        assert_eq!(3, server._read_buffer(0)[3   ]);        
        assert_eq!(9, server._read_buffer(0)[4   ]); // 1st payload 
        assert_eq!(9, server._read_buffer(0)[1369]);       
        assert_eq!(0, server._read_buffer(0)[1370]); // 2nd header 
        assert_eq!(1, server._read_buffer(0)[1371]);
        assert_eq!(0, server._read_buffer(0)[1372]);
        assert_eq!(3, server._read_buffer(0)[1373]);        
        assert_eq!(9, server._read_buffer(0)[1374]); // 2nd payload
        assert_eq!(9, server._read_buffer(0)[2739]);        
        assert_eq!(0, server._read_buffer(0)[2740]); // 3rd header
        assert_eq!(2, server._read_buffer(0)[2741]);
        assert_eq!(0, server._read_buffer(0)[2742]);
        assert_eq!(3, server._read_buffer(0)[2743]);        
        assert_eq!(9, server._read_buffer(0)[2744]); // 3rd payload
        assert_eq!(9, server._read_buffer(0)[4107]);

        // Test channel 1              
        assert_eq!(1, server._read_buffer(1)[0   ]); // 1st header 
        assert_eq!(0, server._read_buffer(1)[1   ]);
        assert_eq!(0, server._read_buffer(1)[2   ]);
        assert_eq!(3, server._read_buffer(1)[3   ]);        
        assert_eq!(9, server._read_buffer(1)[4   ]); // 1st payload 
        assert_eq!(9, server._read_buffer(1)[1369]);       
        assert_eq!(1, server._read_buffer(1)[1370]); // 2nd header 
        assert_eq!(1, server._read_buffer(1)[1371]);
        assert_eq!(0, server._read_buffer(1)[1372]);
        assert_eq!(3, server._read_buffer(1)[1373]);        
        assert_eq!(9, server._read_buffer(1)[1374]); // 2nd payload
        assert_eq!(9, server._read_buffer(1)[2739]);        
        assert_eq!(1, server._read_buffer(1)[2740]); // 3rd header
        assert_eq!(2, server._read_buffer(1)[2741]);
        assert_eq!(0, server._read_buffer(1)[2742]);
        assert_eq!(3, server._read_buffer(1)[2743]);        
        assert_eq!(9, server._read_buffer(1)[2744]); // 3rd payload
        assert_eq!(9, server._read_buffer(1)[4107]);
    }
}
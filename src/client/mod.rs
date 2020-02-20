use std::net::UdpSocket;
use std::thread;
use std::slice;

#[cfg(test)]
static TEST_ADDRESS: &str = "127.0.0.1";

static MAX_PAYLOAD: usize = 5000;

pub struct Client {
    sample_rate:    usize,
    key:            usize,
    rows:           usize,
    audio_buf_size: usize,
    payload_size:   usize,
    server_address: String,
    buffer:         Vec<Vec<u8>>,
    row_count:      Vec<usize>,
}

impl Client {
    pub fn new(server_address: &str) -> Client { 
        let sample_rate    = 0;
        let key            = 0;
        let rows           = 0;
        let audio_buf_size = 0;
        let payload_size   = 0;
        let server_address = server_address.to_string();
        let buffer         = Vec::new();
        let row_count      = Vec::new();

        Client {
            sample_rate,
            key,
            rows,
            audio_buf_size,
            payload_size,
            server_address,
            buffer,
            row_count
        }
    }

    // pub fn read_packet(&mut self) {
    //     let read_len    = self.server_socket.recv_from(&mut self.packet_buffer).unwrap().0;
    //     let encoded_idx = self.packet_buffer[1] as usize;
    //     let num_chunks  = self.packet_buffer[3] as usize;
    //     let row         = encoded_idx / num_chunks;
    //     let row_idx     = encoded_idx % num_chunks;
    //     let buf_idx     = (row * self.audio_buf_size * 4) + (self.payload_size * row_idx);
    //     self.buffer[0][buf_idx..buf_idx + read_len].copy_from_slice(&self.packet_buffer[..read_len]);
    // }

    pub fn get_next_row(&mut self, channel_num: usize) -> &[u8] {
        let start_idx = self.audio_buf_size * self.row_count[channel_num];
        let end_idx   = start_idx + (self.audio_buf_size * 4);
        self.row_count[channel_num] = self.row_count[channel_num] % self.rows;
        &self.buffer[0][start_idx..end_idx]
    }

    // To be removed
    pub fn fetch_packet_info(&self) -> (usize, usize, u8) {
        let socket = UdpSocket::bind(&self.server_address).unwrap();
        let mut buf = vec![0; MAX_PAYLOAD];
        let mut read_len;
        loop {
            read_len = socket.recv_from(&mut buf).unwrap().0;
            // If not last packet (unless only one packet per buffer)
            if (buf[1] % buf[3]) != buf[3] - 1 || buf[3] == 1 {
                break;
            }
        }
        (read_len - 4, buf[3] as usize, buf[2])
    }

    pub fn prime(&mut self, payload_size: usize, num_chunks: usize, sample_rate: u8) {
        self.sample_rate = sample_rate as usize;
        self.key = num_chunks * 2usize.pow(8 - (num_chunks as f64).log2().ceil() as u32);
        self.rows = self.key / num_chunks;
        self.audio_buf_size = 2usize.pow(((payload_size * num_chunks) as f64).log2().floor() as u32 - 2);
        self.payload_size = payload_size;
        let buffer_size = (self.key / num_chunks) * (self.audio_buf_size * 4);
        self.buffer = vec![vec![0; buffer_size]; 1];
        self.row_count = vec![0; 1];
    }

    pub fn spawn_packet_reader(&mut self) {
        let socket            = UdpSocket::bind(&self.server_address).unwrap();
        let mut packet_buffer = vec![0; self.payload_size + 4];
        let buffer_pointer    = &mut self.buffer[0][0] as *mut u8;
        let audio_buf_bytes   = self.audio_buf_size * 4;
        let payload_bytes     = self.payload_size;
        let mut raw_buffer    = unsafe { slice::from_raw_parts_mut(&mut *buffer_pointer, self.buffer[0].len()) };
        
        thread::spawn(move||{
            let mut read_len = 0;
            let mut row      = 0;
            let mut column   = 0;
            let mut index    = 0;
            let mut range    = 0..0;
            while true {
                let read_len = socket.recv_from(&mut packet_buffer).unwrap().0;
                row          = (packet_buffer[1] / packet_buffer[3]) as usize;
                column       = (packet_buffer[1] % packet_buffer[3]) as usize;
                index        = (row * audio_buf_bytes) + (column * payload_bytes);
                range        = index..(index + read_len);
                raw_buffer[range].copy_from_slice(&packet_buffer[0..read_len]);
            }
        });
    }

    pub fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn get_audio_buf_size(&self) -> usize {
        self.audio_buf_size
    }

    pub fn get_server_address(&self) -> String {
        self.server_address.clone()
    }
} 

#[cfg(test)]
mod tests {
    use super::*;   

    #[test]
    fn test_client_prime() {
        let client_address = TEST_ADDRESS.to_owned() + ":9001";
        let mut client = Client::new(&client_address);
        client.prime(1366, 3, 0);
        assert_eq!(client.key, 192);
        assert_eq!(client.rows, 64);
        assert_eq!(client.audio_buf_size, 1024);
        assert_eq!(client.payload_size, 1366);
        assert_eq!(client.buffer[0].len(), 262144);
    }
}
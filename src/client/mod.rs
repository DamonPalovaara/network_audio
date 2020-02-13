use std::net::UdpSocket;

#[cfg(test)]
static TEST_ADDRESS: &str = "127.0.0.1";

pub struct Client {
    sample_rate:    usize,
    key:            usize,
    jack_buf_size:  usize,
    payload_size:   usize,
    server_address: String,
    server_socket:  UdpSocket,
}

impl Client {
    pub fn new(server_address: &str) -> Client { 
        let sample_rate    = 0;
        let key            = 0;
        let jack_buf_size  = 0;
        let payload_size   = 0;
        let server_address = server_address.to_string();
        let server_socket  = UdpSocket::bind(&server_address).unwrap();        

        Client {
            sample_rate,
            key,
            jack_buf_size,
            payload_size,
            server_address,
            server_socket,
        }
    }

    pub fn _read_packet(&mut self) {
        let mut buf = vec![0; 2000];
        self.server_socket.recv_from(&mut buf).unwrap();
    }

    // This should be called first time read_packets gets called
    pub fn prime(&mut self) {
        let mut buf = vec![0; 2000];
        let mut read_len;
        loop {
            read_len = self.server_socket.recv_from(&mut buf).unwrap().0;
            if (buf[1] % buf[3]) != buf[3] - 1 || buf[3] == 1 {
                break;
            }
        }
        self.sample_rate   = buf[2] as usize;
        self.key           = (256usize / buf[3] as usize) * buf[3] as usize;
        self.jack_buf_size = 2usize.pow((( ((read_len - 4) * buf[3] as usize) / 4) as f64).log2().floor() as u32);
        self.payload_size  = read_len - 4;
    }

    pub fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn get_key(&self) -> usize {
        self.key
    }

    pub fn get_jack_buf_size(&self) -> usize {
        self.jack_buf_size
    }

    pub fn get_payload_size(&self) -> usize {
        self.payload_size
    }

    pub fn get_server_address(&self) -> String {
        self.server_address.clone()
    }
} 

#[cfg(test)]
mod tests {
    use super::*;   

    #[test]
    fn test_client_new() {
        let client_address = TEST_ADDRESS.to_owned() + ":9001";
        let _client = Client::new(&client_address);
    }
}
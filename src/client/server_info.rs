// Contains all information about the server
struct ServerInfo {
    channels: Vec<ChannelInfo>,
}

impl ServerInfo {

}

// Contains information about a channel like port number, number of audio channels, etc
struct ChannelInfo {
    name:           String,
    description:    String,
    server_address: String,
    num_channels:   usize,
}

impl ChannelInfo {

}
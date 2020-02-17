Goal:
- I decided to turn this project into a general purpose crate for sending audio over the network
- Needs support for different APIs (ALSA, Windows, ASIO, etc)
- Support different Client Server interactions (TCP, UDP, Multicast, etc)

TODO:
- Finish making this file
- Create a Jack/Server/Client integration test
- Finish implementing new key algorithm

Thoughts:
- Having the server send metadata via a separate stream would allow the header of the audio streams to only be 2 bytes. Would also allow for other information too large for a 4 byte header to be sent
- Each stream (1 or more channels) needs to have it's own port so that the client doesn't read packets it doesn't care about
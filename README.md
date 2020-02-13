Goal:
- Be able to capture audio on a server and multi-cast it to multiple clients
- I'm designing this to be used at 'silent' parties (people listen to the same audio stream on headphones)

TODO:
- Finish making this file
- Separate Client, Server, and the unit tests into separate files

Thoughts:
- Having the server send metadata via a separate stream would allow the header of the audio streams to only be 2 bytes. Would also allow for other information too large for a 4 byte header to be sent
- Each stream (1 or more channels) needs to have it's own port so that the client doesn't read packets it doesn't care about
- Change key generation to num_chunks * 2^x where x is the largest integer that keeps the result <= 256.
    - Would allow Client to be more flexible with buffer size without a complex algorithm 
    - Works out to be num_chunks * 2^floor(8 - log2(num_chunks))
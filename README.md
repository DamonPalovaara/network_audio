TODO:
- Finish making this file
- Separate Client, Server, and the unit tests into separate files

Issues:
- No way to handle jack buffer < udp_payload_size (if using small buffer or jumbo-frames)
- Consider refactoring so that all udp packets are equal length

# Lesson 09
[back to main](../README.md)

https://robot-dreams-rust.mag.wiki/9-network-io/index.html#homework

## Task

Implement client - server message transfer

## Iplementation

- Client can start when server is running or not
- Client will reconnect after server is restarted (try sending some more messages)
- All messages send by a client are distributed to all connected clients
- Images will be converted to .png in client after message received

### Commands:
- any text
- .image file.jpg file2.jpg
- .file file.dat file2.dat
- .quit

### Env Variables:

Server and Client:
- HOSTNAME
  - default localhost
- PORT
  - default 11111

Client only:
- USERNAME 
  - randomly generated if not provided



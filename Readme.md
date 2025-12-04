# Run bin crate

`cargo run --bin <crate name or bin file name>`


## Connecting localhost chat swarm

In 1st terminal run: `cargo run --bin cryptochain -- <port#>
Note the following:

```
Your ID: 12D3KooWJKrPgXDJf9KcqBfUCkLw7XLbShDHtHKz6VLd742XRqY7
* Listening on: /ip4/127.0.0.1/tcp/44595

```

Open second terminal and run this command replacing address and your ID as based on outputs.
`cargo run --bin localp2p -- /ip4/127.0.0.1/tcp/<PortNumber>/p2p/<YourId>`

From here should get:
`Connected to: <PeedID> (1 total peers)` output in both terminals.
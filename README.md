# The Autonomi Network (previously Safe Network)

[Autonomi.com](https://autonomi.com/)

Own your data. Share your disk space. Get paid for doing so.<br>
The Data on the Autonomi Network is Decentralised, Autonomous, and built atop of Kademlia and
Libp2p.<br>

## Table of Contents

- [For Users](#for-users)
- [For Developers](#for-developers)
- [For the Technical](#for-the-technical)
- [Using a Local Network](#using-a-local-network)
- [Metrics Dashboard](#metrics-dashboard)

### For Users

- [CLI](https://github.com/maidsafe/safe_network/blob/main/autonomi-cli/README.md) The Command Line
  Interface, allowing users to interact with the network from their terminal.
- [Node](https://github.com/maidsafe//safe_network/blob/main/sn_node/README.md) The backbone of the
  safe network. Nodes can be run on commodity hardware and provide storage space and validation of
  transactions to the network.
- Web App: Coming Soon!

#### Building the Node from Source

If you wish to build a version of `safenode` from source, some special consideration must be given
if you want it to connect to the current beta network.

You should build from the `stable` branch, as follows:

```
git checkout stable
export FOUNDATION_PK=b20c916c7a28707018292f06dfdb66ab88ebcbad9c78d18135e843a91b1d66b875b24d2c27d8d1ad4637c2d5811896fe
export GENESIS_PK=93f7355906fa8c1a639bac80f4619dbb4cf5f71c47827d1ff2c30f0d133f6b841859662cbf7e0bbceca0eb0f521f6ebc
export NETWORK_ROYALTIES_PK=af451aa34a0d16c50eb217b91ab6b2ca75ef43b9c20449384ff1e90dbf8477351499cca985828e33b208805dadc80c63
export PAYMENT_FORWARD_PK=adc6401588af49c60af6717a60546207abddb4e150014b4ab6c407ef6d7b3d3899b8892a91ab23042378b7b285e655fc
cargo build --release --features=network-contacts --bin safenode
```

#### Running the Node

To run a node and receive rewards, you need to specify your Ethereum address as a parameter. Rewards are paid to the specified address.

```
cargo run --release --bin safenode --features=network-contacts -- --rewards-address <YOUR_ETHEREUM_ADDRESS_TO_RECEIVE_REWARDS>
```

More options about EVM Network below.

### For Developers

#### Build

You should also build `safe` with the `network-contacts` and `distribution` features enabled:

```
cargo build --release --features="network-contacts,distribution" --bin safe
```

For `safenode`, only the `network-contacts` feature should be required:

```
cargo build --release --features=network-contacts --bin safenode
```

#### Main Crates

- [Autonomi API](https://github.com/maidsafe/safe_network/blob/main/autonomi/README.md) The client APIs
  allowing use of the Autonomi Network to users and developers.
- [Autonomi CLI](https://github.com/maidsafe/safe_network/blob/main/autonomi-cli/README.md) The Command Line
  Interface, allowing users to interact with the network from their terminal.
- [Node](https://github.com/maidsafe/safe_network/blob/main/sn_node/README.md) The backbone of the
  autonomi network. Nodes can be run on commodity hardware and run the Network.
- [Node Manager](https://github.com/maidsafe/safe_network/blob/main/sn_node_manager/README.md) Use
  to create a local network for development and testing.
- [Node RPC](https://github.com/maidsafe/safe_network/blob/main/sn_node_rpc_client/README.md) The
  RPC server used by the nodes to expose API calls to the outside world.

#### Transport Protocols and Architectures

The Safe Network uses `quic` as the default transport protocol.

The `websockets` feature is available for the `sn_networking` crate, and above, and will allow for
tcp over websockets.

If building for `wasm32` then `websockets` are enabled by default as this is the only method
available to communicate with a network as things stand. (And that network must have `websockets`
enabled.)

#### Building for wasm32

WASM support for the autonomi API is currently under active development. More docs coming soon.

### For the Technical

- [Logging](https://github.com/maidsafe/safe_network/blob/main/sn_logging/README.md) The
  generalised logging crate used by the safe network (backed by the tracing crate).
- [Metrics](https://github.com/maidsafe/safe_network/blob/main/metrics/README.md) The metrics crate
  used by the safe network.
- [Networking](https://github.com/maidsafe/safe_network/blob/main/sn_networking/README.md) The
  networking layer, built atop libp2p which allows nodes and clients to communicate.
- [Protocol](https://github.com/maidsafe/safe_network/blob/main/sn_protocol/README.md) The protocol
  used by the safe network.
- [Transfers](https://github.com/maidsafe/safe_network/blob/main/sn_transfers/README.md) The
  transfers crate, used to send and receive tokens Native to the network.
- [Registers](https://github.com/maidsafe/safe_network/blob/main/sn_registers/README.md) The
  registers crate, used for the Register CRDT data type on the network.
- [Peers Acquisition](https://github.com/maidsafe/safe_network/blob/main/sn_peers_acquisition/README.md)
  The peers acquisition crate, or: how the network layer discovers bootstrap peers.
- [Build Info](https://github.com/maidsafe/safe_network/blob/main/sn_build_info/README.md) Small
  helper used to get the build/commit versioning info for debug purposes.

### Using a Local Network

We can explore the network's features by using multiple node processes to form a local network. e also need to run a
local EVM network for our nodes and client to connect to.

Follow these steps to create a local network:

##### 1. Prerequisites

The latest version of [Rust](https://www.rust-lang.org/learn/get-started) should be installed. If you already have an installation, use `rustup update` to get the latest version.

Run all the commands from the root of this repository.

If you haven't already, install Foundry. We need to have access to Anvil, which is packaged with Foundry, to run an EVM node: https://book.getfoundry.sh/getting-started/installation

To collect rewards for you nodes, you will need an EVM address, you can create one using [metamask](https://metamask.io/).

##### 2. Run a local EVM node

```sh
cargo run --bin evm_testnet
```

This creates a CSV file with the EVM network params in your data directory.

##### 3. Create the test network and pass the EVM params
   `--rewards-address` _is the address where you will receive your node earnings on._

```bash
cargo run --bin=safenode-manager --features=local -- local run --build --clean --rewards-address <YOUR_ETHEREUM_ADDRESS>
```

The EVM Network parameters are loaded from the CSV file in your data directory automatically when the `local` feature flag is enabled (`--features=local`).

##### 4. Verify node status

```bash
cargo run --bin safenode-manager --features local -- status
```

The node manager's `run` command starts the node processes. The `status` command should show twenty-five
running nodes.

##### 5. Uploading and Downloading Data

To upload a file or a directory, you need to set the `SECRET_KEY` environment variable to your EVM secret key:

> When running a local network, you can use the `SECRET_KEY` printed by the `evm_testnet` command [step 2](#2-run-a-local-evm-node) as it has all the money.

```bash
SECRET_KEY=<YOUR_EVM_SECRET_KEY> cargo run --bin autonomi --features local -- file upload <path>
```

The output will print out the address at which the content was uploaded.

Now to download the files again:

```bash
cargo run --bin autonomi --features local -- file download <addr> <dest_path>
```

### Registers

Registers are one of the network's data types. The workspace here has an example app demonstrating
their use by two users to exchange text messages in a crude chat application.

In the first terminal, using the registers example, Alice creates a register:

```
cargo run --example registers --features=local -- --user alice --reg-nickname myregister
```

Alice can now write a message to the register and see anything written by anyone else. For example
she might enter the text "Hello, who's there?" which is written to the register and then shown as
the "Latest value", in her terminal:

```
Register address: "50f4c9d55aa1f4fc19149a86e023cd189e509519788b4ad8625a1ce62932d1938cf4242e029cada768e7af0123a98c25973804d84ad397ca65cb89d6580d04ff07e5b196ea86f882b925be6ade06fc8d"
Register owned by: PublicKey(0cf4..08a5)
Register permissions: Permissions { anyone_can_write: true, writers: {PublicKey(0cf4..08a5)} }

Current total number of items in Register: 0
Latest value (more than one if concurrent writes were made):
--------------
--------------

Enter a blank line to receive updates, or some text to be written.
Hello, who's there?
Writing msg (offline) to Register: 'Hello, who's there?'
Syncing with SAFE in 2s...
synced!

Current total number of items in Register: 1
Latest value (more than one if concurrent writes were made):
--------------
[Alice]: Hello, who's there?
--------------

Enter a blank line to receive updates, or some text to be written.

```

For anyone else to write to the same register they need to know its xor address, so to communicate
with her friend Bob, Alice needs to find a way to send it to Bob. In her terminal, this is the
value starting "50f4..." in the output above. This value will be different each time you run the
example to create a register.

Having received the xor address, in another terminal Bob can access the same register to see the
message Alice has written, and he can write back by running this command with the address received
from Alice. (Note that the command should all be on one line):

```
cargo run --example registers --features=local -- --user bob --reg-address 50f4c9d55aa1f4fc19149a86e023cd189e509519788b4ad8625a1ce62932d1938cf4242e029cada768e7af0123a98c25973804d84ad397ca65cb89d6580d04ff07e5b196ea86f882b925be6ade06fc8d
```

After retrieving the register and displaying the message from Alice, Bob can reply and at any time,
Alice or Bob can send another message and see any new messages which have been written, or enter a
blank line to poll for updates.

Here's Bob writing from his terminal:

```
Latest value (more than one if concurrent writes were made):
--------------
[Alice]: Hello, who's there?
--------------

Enter a blank line to receive updates, or some text to be written.
hi Alice, this is Bob!
```

Alice will see Bob's message when she either enters a blank line or writes another message herself.

### Inspect a Register

A second example, `register_inspect` allows you to view its structure and content. To use this with
the above example you again provide the address of the register. For example:

```
cargo run --example register_inspect --features=local -- --reg-address 50f4c9d55aa1f4fc19149a86e023cd189e509519788b4ad8625a1ce62932d1938cf4242e029cada768e7af0123a98c25973804d84ad397ca65cb89d6580d04ff07e5b196ea86f882b925be6ade06fc8d
```

After printing a summary of the register, this example will display
the structure of the register each time you press Enter, including the following:

```
Enter a blank line to print the latest register structure (or 'Q' <Enter> to quit)

Syncing with SAFE...
synced!
======================
Root (Latest) Node(s):
[ 0] Node("4eadd9"..) Entry("[alice]: this is alice 3")
[ 3] Node("f05112"..) Entry("[bob]: this is bob 3")
======================
Register Structure:
(In general, earlier nodes are more indented)
[ 0] Node("4eadd9"..) Entry("[alice]: this is alice 3")
  [ 1] Node("f5afb2"..) Entry("[alice]: this is alice 2")
    [ 2] Node("7693eb"..) Entry("[alice]: hello this is alice")
[ 3] Node("f05112"..) Entry("[bob]: this is bob 3")
  [ 4] Node("8c3cce"..) Entry("[bob]: this is bob 2")
    [ 5] Node("c7f9fc"..) Entry("[bob]: this is bob 1")
    [ 1] Node("f5afb2"..) Entry("[alice]: this is alice 2")
      [ 2] Node("7693eb"..) Entry("[alice]: hello this is alice")
======================
```

Each increase in indentation shows the children of the node above.
The numbers in square brackets are just to make it easier to see
where a node occurs more than once.

### RPC

The node manager launches each node process with a remote procedure call (RPC) service. The
workspace has a client binary that can be used to run commands against these services.

Run the `status` command with the `--details` flag to get the RPC port for each node:

```
$ cargo run --bin safenode-manager -- status --details
...
===================================
safenode-local25 - RUNNING
===================================
Version: 0.103.21
Peer ID: 12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8
Port: 38835
RPC Port: 34416
Multiaddr: /ip4/127.0.0.1/udp/38835/quic-v1/p2p/12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8
PID: 62369
Data path: /home/<<user_directory>>/.local/share/safe/node/12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8
Log path: /home/<<user_directory>>/.local/share/safe/node/12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8/logs
Bin path: target/release/safenode
Connected peers: 24
```

Now you can run RPC commands against any node.

The `info` command will retrieve basic information about the node:

```
$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 info
Node info:
==========
RPC endpoint: https://127.0.0.1:34416
Peer Id: 12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8
Logs dir: /home/<<user_directory>>/.local/share/safe/node/12D3KooWJ4Yp8CjrbuUyeLDsAgMfCb3GAYMoBvJCRp1axjHr9cf8/logs
PID: 62369
Binary version: 0.103.21
Time since last restart: 1614s
```

The `netinfo` command will return connected peers and listeners:

```
$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 netinfo
Node's connections to the Network:

Connected peers:
Peer: 12D3KooWJkD2pB2WdczBJWt4ZSAWfFFMa8FHe6w9sKvH2mZ6RKdm
Peer: 12D3KooWRNCqFYX8dJKcSTAgxcy5CLMcEoM87ZSzeF43kCVCCFnc
Peer: 12D3KooWLDUFPR2jCZ88pyYCNMZNa4PruweMsZDJXUvVeg1sSMtN
Peer: 12D3KooWC8GR5NQeJwTsvn9SKChRZqJU8XS8ZzKPwwgBi63FHdUQ
Peer: 12D3KooWJGERJnGd5N814V295zq1CioxUUWKgNZy4zJmBLodAPEj
Peer: 12D3KooWJ9KHPwwiRpgxwhwsjCiHecvkr2w3JsUQ1MF8q9gzWV6U
Peer: 12D3KooWSBafke1pzz3KUXbH875GYcMLVqVht5aaXNSRtbie6G9g
Peer: 12D3KooWJtKc4C7SRkei3VURDpnsegLUuQuyKxzRpCtsJGhakYfX
Peer: 12D3KooWKg8HsTQ2XmBVCeGxk7jHTxuyv4wWCWE2pLPkrhFHkwXQ
Peer: 12D3KooWQshef5sJy4rEhrtq2cHGagdNLCvcvMn9VXwMiLnqjPFA
Peer: 12D3KooWLfXHapVy4VV1DxWndCt3PmqkSRjFAigsSAaEnKzrtukD

Node's listeners:
Listener: /ip4/127.0.0.1/udp/38835/quic-v1
Listener: /ip4/192.168.1.86/udp/38835/quic-v1
Listener: /ip4/172.17.0.1/udp/38835/quic-v1
Listener: /ip4/172.18.0.1/udp/38835/quic-v1
Listener: /ip4/172.20.0.1/udp/38835/quic-v1
```

Node control commands:

```
$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 restart 5000
Node successfully received the request to restart in 5s

$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 stop 6000
Node successfully received the request to stop in 6s

$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 update 7000
Node successfully received the request to try to update in 7s
```

NOTE: it is preferable to use the node manager to control the node rather than RPC commands.

Listening to royalty payment events:

```
$ cargo run --bin safenode_rpc_client -- 127.0.0.1:34416 transfers
Listening to transfer notifications... (press Ctrl+C to exit)

New transfer notification received for PublicKey(0c54..5952), containing 1 cash note/s.
CashNote received with UniquePubkey(PublicKey(19ee..1580)), value: 0.000000001

New transfer notification received for PublicKey(0c54..5952), containing 1 cash note/s.
CashNote received with UniquePubkey(PublicKey(19ee..1580)), value: 0.000000001
```

The `transfers` command can provide a path for royalty payment cash notes:

```
$ cargo run --release --bin=safenode_rpc_client -- 127.0.0.1:34416 transfers ./royalties-cash-notes
Listening to transfer notifications... (press Ctrl+C to exit)
Writing cash notes to: ./royalties-cash-notes
```

Each received cash note is written to a file in the directory above, under another directory
corresponding to the public address of the recipient.

### Tear Down

When you're finished experimenting, tear down the network:

```bash
cargo run --bin safenode-manager -- local kill
```

## Metrics Dashboard

Use the `open-metrics` feature flag on the node / client to start
an [OpenMetrics](https://github.com/OpenObservability/OpenMetrics/) exporter. The metrics are
served via a webserver started at a random port. Check the log file / stdout to find the webserver
URL, `Metrics server on http://127.0.0.1:xxxx/metrics`

The metrics can then be collected using a collector (for e.g. Prometheus) and the data can then be
imported into any visualization tool (for e.g., Grafana) to be further analyzed. Refer to
this [Guide](./metrics/README.md) to easily setup a dockerized Grafana dashboard to visualize the
metrics.

## Contributing

Feel free to clone and modify this project. Pull requests are welcome.<br>You can also
visit \* \*[The MaidSafe Forum](https://safenetforum.org/)\*\* for discussion or if you would like to join our
online community.

### Pull Request Process

1. Please direct all pull requests to the `alpha` branch instead of the `main` branch.
1. Ensure that your commit messages clearly describe the changes you have made and use
   the [Conventional Commits](https://www.conventionalcommits.org/) specification.

## License

This Safe Network repository is licensed under the General Public License (GPL), version
3 ([LICENSE](http://www.gnu.org/licenses/gpl-3.0.en.html)).

# Networking (Joining The Lightning Network)

So far, we've implemented quite a lot of important functionality for our node - sourcing blockchain data, publishing transactions, estimating fees, etc. That's all good fun, but Lightning is a team sport. There is no "I" in team, right? So, how do we join the party?

<p align="center" style="width: 50%; max-width: 300px;">
  <img src="./tutorial_images/node_setup/networking.png" alt="networking" width="100%" height="auto">
</p>

To conenct to peers, we're going to need to implement the ability to conduct high performance I/O operations. For those of you who do not come from a computer networking background, let's take a moment to unpack "high performance I/O operations", as it's helpful to understanding both how Lightning nodes and LDK work under the hood.

First, let's discuss an **Input/Output** (**I/O**) operation. In the context of our Lightning node, an I/O operation refers to the process of exchanging data between our program and external systems (other nodes or servers on the network). This is how we'll be able to communicate vital tasks for our node, such as gathering network gossip, opening channels, and routing payments. 

Now, we don't want our operations (there will be many!) to be slow or get in the way of eachother. In other words, we're going to want "high performance" operations. This means that our operations should have properties like being asynchronous and non-blocking. An asynchronous operation will initiate the task and then immediately continue to executing other tasks. On the other hand, synchronous operations will wait until the current task has finised until moving on to the next one. In other words, it will "block" other tasks from completing while it's waiting.

#### Question: Why is it imperative that our node performs asyncronous I/O instead of synchronous? This may seem like a trivial question, but it's worth pondering if you don't come from a system background!


<details>
  <summary>Answer</summary>

This isn't a trick question! It's just meant to spark further thought.

In short, our Lightning node will be performing many actions at once. For example, we'll be processing new gossip messages, opening channels, routing payments, monitoring the blockchain, etc. The list goes on and on.

If we had to wait for any given task to complete before moving on to another task, we wouldn't be able to run an working node.

</details>

## TCP (Transmission Control Protocol)

To connect to our peers, we'll rely on a TCP/IP connection. This includes protocols such as IPv4, IPv6, and Tor. These protocols are used because they satisfy properties that Lightning requires. For example, consider this sentence from [BOLT #1](https://github.com/lightning/bolts/blob/master/01-messaging.md), which describes Lightning's Base Protocol: **This protocol assumes an underlying authenticated and ordered transport mechanism that takes care of framing individual messages**.

#### Question: Why is ordered transport required by Lightning?


<details>
  <summary>Answer</summary>

First, let's review what "ordered transport" is.

**Ordered transport**, unsurprisingly, means that messages will arrive in the same order that they were sent. To see why this is critical, let's take a brief detour to [BOLT #2: Peer Protocol for Channel Management
](https://github.com/lightning/bolts/blob/master/02-peer-protocol.md). BOLT #2 describes the message types that peers will send eachother to update their channel states. For instance, consider the following common message types:

<p align="center" style="width: 50%; max-width: 300px;">
  <img src="./tutorial_images/node_setup/alice_bob_tcp_0.png" alt="alice_bob_tcp_0" width="50%" height="auto">
</p>

- `update_add_htlc`: Node A will send this message to node B (or vice versa) to indicate that they would like to add an HTLC to their commitment transactions.
- `commitment_signed`: Node A will send this message to node B (or vice versa) to provide the signature(s) for the current commitment transaction, effectively advancing channel state.

Let's imagine that Alice wants to add two HTLCs to her channel with Bob. She can do that by sending two `update_add_htlc` messages to Bob and then sending a `commitment_signed` message with the appropriate signatures.

<p align="center" style="width: 50%; max-width: 300px;">
  <img src="./tutorial_images/node_setup/alice_bob_tcp_1.png" alt="alice_bob_tcp_1" width="70%" height="auto">
</p>

Notice that the `commitment_signed` does not explitly mention which HTLCs the signatures are for. Instead, it does this implicitly. Since Lightning messages are assumed to be ordered and reliable, the protocol assumes that messages sent will always arrive, and they will arrive in the order they are sent. This way, Alice can rest assured that, if Bob gets the `commitment_signed`, he also got the `update_add_htlc` messages in the correct order.

For a great in-depth blog, discussing how to operate a Lightning channel, please see [Normal operation and closure of a pre-taproot LN channel](https://ellemouton.com/posts/normal-operation-pre-taproot/) by Elle Mouton.

</details>

TCP achieves reliable message delivery by assigning a *sequence number* to each byte of data transmitted to the receiver. To ensure that the data was successfully received, TCP requires the reciever to send a positive acknowledgment (ACK) back to the sender. If the ACK message is not received before the time-out interval, the data will be sent again.

Additionally, since the data is transmitted with sequence numbers, the receiver can correctly order the data they reciever, ensuring that it is processed and handled in the right order. Cool stuff, eh!

## Connecting to Peers

To connect to peers, we'll need to identify them by their **TCP Address**. A TCP address is composed of an **IP Address** and a **Port Number** in the following format: `IP:Port`.

An **IP Address** is used to identify a specific host on a network and functions like a postal address, allowing data to be delivered to the correct location. There are two types of IP Addresses:
- **IPv4**: A 32-bit address
- **IPv6**: A 128-bit address

A **Port Number** identifies the specifi capplication running on the device. It can be a 16-bit integer, effectively between 0-65,535. Lightning's default port is 9735.

Nodes that wish to prioritize privacy and anonymity, may opt to run or leverage a **Tor (The Onion Router)** server, thus concealing their IP Address. The Tor client will then route any incoming or outgoing connections through the Tor network's encrypted relays, thus preserving your privacy. Users who leverage Tor will have an **onion address**, which reads `<username>.onion:<port>`.

## LDK lightning_net_tokio Crate

LDK provides a `lightning_net_tokio` Rust crate which provides a socket handling library for who wish to use rust-lightning with native a `TcpStream`.

LDK has made implementing this create quite easy, with just a few steps to complete.

### Define our TCP Listener

First, we'll need to inform LDK of where our node should be listening for incoming network traffic from. We can do this by defining a `TcpListener` and then passing our listener and a `PeerManager` as inputs to the `setup_inbound` function made available via `lightning_net_tokio`.

```rust
use lightning_net_tokio; // use LDK's sample networking module

let listen_port = 9735;
let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", listen_port))
    .await.unwrap()
loop {
    let tcp_stream = listener.accept().await.unwrap().0;
    tokio::spawn(async move {
        // Use LDK's supplied networking battery to facilitate inbound
        // connections.
        lightning_net_tokio::setup_inbound(
            &peer_manager,
            tcp_stream.into_std().unwrap(),
        )
        .await;
    });
}
```

Let's dig in a little to see what's happening in each step above.

#### 1. Define our TCP Listener
First we create a listener socket, which is bound to the IP address that we provide. 

#### 2. Listen for Connection
Next, we create a loop which will continously check for incoming connections from peers. Specifically, `lister.accept()` will wait asyncronously for an incoming connection. 

#### 3. Spawn a Task to Handle Conneciton
When we have a new conection, we'll spawn an asychronous task to handle the connection. Since each connection is processed with it's own task, we are able to handle multiple connections concurrently.

#### 4. Call `set_inbound` Function
Finally, we'll provide the new connection and a `peer_manager` to the `setup_inbound` function. This will initialize the new connection within the framework of LDK.
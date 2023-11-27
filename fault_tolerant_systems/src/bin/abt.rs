use std::sync::mpsc;
use std::time::Duration;
use std::{fmt, thread};

#[derive(PartialEq, Clone)]
enum ZeroOrOne {
    Zero,
    One,
}

impl ZeroOrOne {
    fn reverse(&mut self) {
        match self {
            ZeroOrOne::Zero => *self = ZeroOrOne::One,
            ZeroOrOne::One => *self = ZeroOrOne::Zero,
        }
    }

    fn get_reverse(&self) -> ZeroOrOne {
        match self {
            ZeroOrOne::Zero => ZeroOrOne::One,
            ZeroOrOne::One => ZeroOrOne::Zero,
        }
    }
}

impl fmt::Display for ZeroOrOne {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZeroOrOne::Zero => write!(f, "0"),
            ZeroOrOne::One => write!(f, "1"),
        }
    }
}

type MsgAndId = (&'static str, ZeroOrOne);
const ACK_MSG: &str = "ack";
const TIMEOUT: Duration = Duration::from_millis(10);

fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel::<MsgAndId>();
    let (tx_to_sender, rx_from_receiver) = mpsc::channel::<MsgAndId>();

    // sender
    let sender = thread::spawn(move || {
        let messages = ["a", "b", "c", "d", "e", "f", "g", "h"];

        let mut next_i: usize = 0;
        let mut sent = ZeroOrOne::One;
        let mut b = ZeroOrOne::Zero;
        while next_i < messages.len() {
            if sent != b {
                println!("[sender] Sending message ({}, {})", messages[next_i], b);
                tx_to_receiver.send((messages[next_i], b.clone())).unwrap();
                next_i += 1;
                sent.reverse();
            }

            match rx_from_receiver.recv_timeout(TIMEOUT) {
                Ok((_ack, receive_i)) => {
                    println!("[sender] Received ack");
                    if receive_i == b {
                        println!("[sender] Index is correct, sending next message");
                        b.reverse();
                    } else {
                        println!("[sender] Index is incorrect, skipping");
                    }
                }
                Err(_) => {
                    println!("[sender] Timeout, resending message");
                    sent.reverse();
                    next_i -= 1;
                }
            }
        }
    });

    // receiver
    let receiver = thread::spawn(move || {
        let mut expect_i = ZeroOrOne::Zero;
        while let Ok((message, receive_i)) = rx_from_sender.recv() {
            println!("[receiver] Received message: ({}, {})", message, receive_i);
            if receive_i == expect_i {
                println!("[receiver] Index is correct, sending ack");
                tx_to_sender.send((ACK_MSG, expect_i.clone())).unwrap();
                expect_i.reverse();
            } else {
                println!("[receiver] Index is incorrect, sending ack");
                tx_to_sender
                    .send((ACK_MSG, expect_i.get_reverse()))
                    .unwrap();
            }
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();
}

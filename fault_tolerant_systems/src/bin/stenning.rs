use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type MsgAndId = (&'static str, usize);
const TIMEOUT: Duration = Duration::from_millis(10);
const ACK_MSG: &str = "ack";

fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel::<MsgAndId>();
    let (tx_to_sender, rx_from_receiver) = mpsc::channel::<MsgAndId>();

    // sender
    let sender = thread::spawn(move || {
        let messages = ["a", "b", "c", "d", "e", "f", "g", "h"];

        let mut send_flag = true;
        let mut next_i: usize = 0;
        while next_i < messages.len() {
            if send_flag {
                println!(
                    "[sender] Sending message: ({}, {})",
                    messages[next_i], next_i
                );
                tx_to_receiver.send((messages[next_i], next_i)).unwrap();
                send_flag = false;
            }

            match rx_from_receiver.recv_timeout(TIMEOUT) {
                Ok((_ack, receive_i)) => {
                    println!("[sender] Received ack");
                    if receive_i == next_i {
                        println!("[sender] Index is correct, sending next message");
                        send_flag = true;
                        next_i += 1;
                    } else {
                        println!("[sender] Index is incorrect, skipping");
                        continue;
                    }
                }
                Err(_) => {
                    println!("[sender] Timeout, resending message");
                    send_flag = true;
                }
            }
        }
    });

    // receiver
    let receiver = thread::spawn(move || {
        let mut expect_i = 0;
        while let Ok((message, receive_i)) = rx_from_sender.recv() {
            println!("[receiver] Received message: ({}, {})", message, receive_i);
            if receive_i == expect_i {
                println!("[receiver] Index is correct, sending ack");
                tx_to_sender.send((ACK_MSG, expect_i)).unwrap();
                expect_i += 1;
            } else {
                println!("[receiver] Index is incorrect, sending ack");
                tx_to_sender.send((ACK_MSG, expect_i - 1)).unwrap();
            }
        }
    });

    sender.join().unwrap();
    receiver.join().unwrap();
}

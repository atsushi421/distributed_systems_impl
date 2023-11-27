#![feature(deadline_api)]
use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

type MsgAndId = (&'static str, usize);
const ACK_MSG: &str = "ack";
const TIMEOUT: Duration = Duration::from_millis(10);
const WINDOW_SIZE: i32 = 3;

fn main() {
    let (tx_to_receiver, rx_from_sender) = mpsc::channel::<MsgAndId>();
    let (tx_to_sender, rx_from_receiver) = mpsc::channel::<MsgAndId>();

    // sender
    let sender = thread::spawn(move || {
        let messages = ["a", "b", "c", "d", "e", "f", "g", "h"];

        let mut next_i: usize = 0;
        let mut last_ack_i = -1;
        let mut sending_times = VecDeque::new();
        while next_i < messages.len() {
            while (last_ack_i + 1) as usize <= next_i
                && next_i <= (last_ack_i + WINDOW_SIZE) as usize
            {
                println!(
                    "[sender] Sending message ({}, {})",
                    messages[next_i], next_i
                );
                tx_to_receiver.send((messages[next_i], next_i)).unwrap();
                sending_times.push_back(std::time::Instant::now());
                next_i += 1;
            }

            match rx_from_receiver.recv_deadline(*sending_times.front().unwrap() + TIMEOUT) {
                Ok((_ack, receive_i)) => {
                    println!("[sender] Received ack");
                    if receive_i as i32 == last_ack_i + 1 {
                        println!("[sender] Index is correct, moving window");
                        last_ack_i = receive_i as i32;
                        sending_times.pop_front();
                    } else {
                        println!("[sender] Index is incorrect, skipping");
                    }
                }
                Err(_) => {
                    println!("[sender] Timeout, resending window");
                    next_i = (last_ack_i + 1) as usize;
                }
            }

            thread::sleep(Duration::from_nanos(10));
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

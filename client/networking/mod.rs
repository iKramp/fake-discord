use std::io::{Read, Write,BufReader,BufRead};
use std::net::TcpStream;
use std::str::from_utf8;

use super::user_interface::{FromNetworkingEvent, ToNetworkingEvent};
use crate::user_interface::FromNetworkingEvent::SenderInitialized;
use crate::user_interface::{Message, MessageId};
use iced::futures::channel::mpsc::Sender;
use iced::futures::SinkExt;
use iced::Result;
use tokio::sync::mpsc;


pub struct ClientNetworking{
    stream : Option<TcpStream>
}

impl ClientNetworking {
    pub fn new() -> Self {
        Self {
            stream : None
        }
    }

    pub async fn manage_connection(&mut self, mut event_sender: Sender<FromNetworkingEvent>) -> Result {
            let (to_event_sender, mut to_event_receiver) = mpsc::unbounded_channel::<ToNetworkingEvent>();
        event_sender
            .send(SenderInitialized(to_event_sender))
            .await
            .unwrap();

        self.stream = Some(TcpStream::connect("localhost:8080").expect("Couldnt connect to server!"));

        println!("Established connection");

        loop {
            while let Some(message) = to_event_receiver.recv().await {
                match message {
                    ToNetworkingEvent::MessageSent(msg) => {
                        //send message to yourself
                        event_sender
                            .send(FromNetworkingEvent::Message(
                                MessageId::new(0),
                                Message {
                                    contents: msg.clone(),
                                    sender: "You".to_owned(),
                                },
                            ))
                            .await
                            .unwrap();
                        
                        //send to server
                        self.stream.as_ref().unwrap().write(&msg.as_bytes());   
                        println!("Sending message to server,awaiting reply...");
                        //await reply
                        let mut buf = [0;512];
                        let bytes_read = self.stream.as_ref().unwrap().read(&mut buf).unwrap();
                        if bytes_read == 0 { return Ok(());}
                        println!("Got it");
                            event_sender.send(FromNetworkingEvent::Message(
                            MessageId::new(0),
                            Message {
                                contents: std::str::from_utf8(&buf[..bytes_read]).unwrap().to_string(),
                                sender: "Other guy".to_owned(),
                            },
                            ))
                            .await
                            .unwrap();                            
                        tokio::time::sleep(core::time::Duration::from_millis(10)).await;
                    },
                    _ => (),
                }
            }
        }
        //println!("Terminated.");
    }


    pub fn pollServer(){



    }

    pub fn send_message(){




    }
}
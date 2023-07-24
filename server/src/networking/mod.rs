use crate::database::database_commands::save_message;

use super::database::database_actions::DbManager;
use super::database::{data_types::User, database_actions::QerryReturnType};
use alloc::str;
use alloc::sync::Arc;
use anyhow::Result;
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

pub struct ServerNetworking {
    // channels: HashMap<u64, Vec<TcpStream>>,
    _clients: Vec<TcpStream>,
}

struct Request {
    task_type_id: u8,
    task: Box<tokio::task::JoinHandle<Result<QerryReturnType>>>,
}

impl ServerNetworking {
    pub const fn new() -> Self {
        Self {
            _clients: Vec::new(),
        }
    }

    pub async fn handle_client(
        mut stream: TcpStream,
        db_manager: Arc<DbManager>,
        client_id: u64,
    ) -> Result<()> {
        println!("Incoming connection from: {}", stream.peer_addr()?);
        let mut stream_manager = network_manager::NetworkManager::new(stream).await;
        println!("stream manager ready");
        let mut querries_vec: Vec<Request> = Vec::new(); //when a request is sent from the client, spawn a task, save it here and loop through this and return the data when a task finishes
        //let mut _user: Option<User> = None; //I leave this here to remind you that as soon as the initial connection is made, packets containing the public keys should be sent.
        //This also implies user authentication and thus we can be sure which user is on this connection. For all future networking the
        //connection will bi encrypted so having the user (and his public key) in memory is beneficial

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));//so we don't hog the resources or something idk
            let data = stream_manager.get_message().unwrap_or(Vec::new());
            if !stream_manager.connected {
                println!("client disconnected");
                //clear the querries vec
                return Ok(());
            }
            if data.len() > 0 {
                let data = kvptree::from_string(data)?;
                println!("Received message from client");
                let request_type_id = data.get_str("request_type_id")?.parse::<u64>()?;

                //stream.write_all(buf.get(..bytes_read).ok_or(anyhow::anyhow!("err"))?)?;
                //println!("Echoed");

                //decide what to do depending on the client request
                // 1 - client requests its id
                // 2 - client sends a message
                // 3 - client wants new messages ig
                if request_type_id == 1 {
                    println!("returning id");
                    let data = kvptree::ValueType::LIST(HashMap::from([
                        (
                            "answer_type_id".to_owned(),
                        kvptree::ValueType::STRING("1".to_owned()),
                    ),
                        (
                            "answer".to_owned(),
                        kvptree::ValueType::LIST(HashMap::from([(
                            "client_id".to_owned(),
                            kvptree::ValueType::STRING(client_id.to_string()),
                        )])),
                    ),
                        ]));
                    stream_manager.send_message(kvptree::to_string(data));
                } else if request_type_id == 2 {
                    //TODO: refactor this mess and separate it more
                    let data = data.get_node("request")?;
                    println!("saving message");
                    let msg = crate::database::data_types::Message {
                        id: 1,
                        user_id: client_id,
                        channel_id: 1,
                        text: data.get_str("message")?,
                        date_created: 1,
                    };
                    let tman = db_manager.clone();
                    let handle = tokio::spawn(async move { tman.save_message(&msg).await });
                    querries_vec.push(Request {
                        task_type_id: 2,
                        task: Box::new(handle),
                    });
                } else if request_type_id == 3 {
                    let data = data.get_node("request")?;
                    println!("client wants recent messages");
                    let tman = db_manager.clone();
                    let handle = tokio::spawn(async move {
                        tman.get_new_messages(
                            data.get_str("request.channel_id")?.parse::<u64>()?,
                        data.get_str("request.last_message_id")?.parse::<u64>()?,
                    )
                    .await //actually read these numbers lol
                    });
                    querries_vec.push(Request {
                        task_type_id: 3,
                        task: Box::new(handle),
                    });
                }
            }

            for (id, request) in querries_vec.iter_mut().enumerate() {
                if request.task.is_finished() {
                    let (res,) = tokio::join!(&mut request.task); //use res to return a value
                    if request.task_type_id == 3 {
                        //return messages
                        let returned_data = res??;
                        if let QerryReturnType::Messages(vec) = returned_data {
                            let mut buf = Vec::new();
                            buf.push(3);
                            for message in vec {
                                let mut temp_buf: Vec<u8> = Vec::new();
                                temp_buf.append(&mut message.id.to_be_bytes().to_vec());
                                temp_buf.append(&mut message.channel_id.to_be_bytes().to_vec());
                                temp_buf.append(&mut message.text.as_bytes().to_vec());

                                buf.append(&mut temp_buf.len().to_be_bytes().to_vec());
                                buf.append(&mut temp_buf);
                            }

                            stream_manager.send_message(buf);
                        } else {
                            println!("error");
                        }
                    }
                    querries_vec.remove(id);
                    break; //we break so we have no borrow conflicts. returning 1 result per loop is sufficient anyway
                }
            }
        }
    }

    pub async fn listen_for_client(&mut self, db_manager: DbManager) {
        let db_manager = Arc::new(db_manager);
        //listen on port 8080
        let listener = TcpListener::bind("localhost:8080").unwrap();
        println!("Server listening on port 8080");

        let mut client_cnt = 0;
        // spawn a new thread for each connection
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    client_cnt += 1;
                    println!("New connection: {} ", stream.peer_addr().unwrap());
                    //self.clients.push(stream.try_clone().unwrap());
                    let temp = db_manager.clone();
                    let handle =
                        tokio::spawn(
                            async move { Self::handle_client(stream, temp, client_cnt).await },
                        );
                    let res = handle.await;
                    if let Err(e) = res {
                        println!("{e}");
                    }
                }
                Err(e) => {
                    println!("Error: {e}");
                }
            }
        }
        //close socket server
        println!("Stopping listening");
        drop(listener);
    }
}
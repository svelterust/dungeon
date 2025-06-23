// Imports
use argh::FromArgs;
use dungeon::Payload;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
struct PlayerState {
    x: f32,
    y: f32,
}

#[derive(FromArgs)]
/// Dungeon multiplayer server
struct Args {
    #[argh(option, short = 'p', default = "9000")]
    /// port to listen on
    port: u16,
}

struct Server {
    clients: Arc<Mutex<HashMap<u32, TcpStream>>>,
    client_counter: Arc<Mutex<u32>>,
    player_positions: Arc<Mutex<HashMap<u32, PlayerState>>>,
}

impl Server {
    fn new() -> Self {
        Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_counter: Arc::new(Mutex::new(0)),
            player_positions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn start(&self, address: &str) -> std::io::Result<()> {
        // Start server on address
        let listener = TcpListener::bind(address)?;
        println!("Server listening on {address}");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let clients = Arc::clone(&self.clients);
                    let counter = Arc::clone(&self.client_counter);
                    let positions = Arc::clone(&self.player_positions);
                    thread::spawn(move || {
                        Self::handle_client(stream, clients, counter, positions);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {e}");
                }
            }
        }
        Ok(())
    }

    fn handle_client(
        mut stream: TcpStream,
        clients: Arc<Mutex<HashMap<u32, TcpStream>>>,
        counter: Arc<Mutex<u32>>,
        positions: Arc<Mutex<HashMap<u32, PlayerState>>>,
    ) {
        let client_id = {
            let mut counter = counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        println!("New client connected: {client_id}");

        stream.set_nodelay(true).expect("Failed to set nodelay");
        let stream_clone = stream.try_clone().expect("Failed to clone stream");
        {
            let mut clients = clients.lock().unwrap();
            clients.insert(client_id, stream_clone);
        }

        let mut buffer = [0; 1024];
        let mut message_buffer = Vec::new();

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client {client_id} disconnected");
                    Self::handle_disconnect(&clients, &positions, client_id);
                    break;
                }
                Ok(n) => {
                    message_buffer.extend_from_slice(&buffer[..n]);

                    // Process all complete messages
                    while message_buffer.len() >= 4 {
                        let len = u32::from_le_bytes([
                            message_buffer[0],
                            message_buffer[1],
                            message_buffer[2],
                            message_buffer[3],
                        ]) as usize;

                        if message_buffer.len() >= 4 + len {
                            if let Ok(payload) =
                                bincode::deserialize::<Payload>(&message_buffer[4..4 + len])
                            {
                                Self::handle_payload(
                                    &clients,
                                    &positions,
                                    client_id,
                                    payload,
                                    &message_buffer[..4 + len],
                                );
                            }
                            message_buffer.drain(..4 + len);
                        } else {
                            break;
                        }
                    }
                }
                Err(_) => {
                    Self::handle_disconnect(&clients, &positions, client_id);
                    break;
                }
            }
        }
    }

    fn handle_payload(
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        positions: &Arc<Mutex<HashMap<u32, PlayerState>>>,
        sender_id: u32,
        payload: Payload,
        message: &[u8],
    ) {
        match payload {
            Payload::Move(player_id, x, y) => {
                // Update player position
                {
                    let mut positions = positions.lock().unwrap();
                    positions.insert(player_id, PlayerState { x, y });
                }
                // Broadcast move to others
                Self::broadcast_to_others(clients, sender_id, message);
            }
            Payload::Join(_) => {
                // Send current state to new player
                Self::send_current_state(clients, positions, sender_id);
                // Broadcast join to others
                Self::broadcast_to_others(clients, sender_id, message);
            }
            Payload::Leave(_) => {
                // Remove from positions and broadcast
                {
                    let mut positions = positions.lock().unwrap();
                    positions.remove(&sender_id);
                }
                Self::broadcast_to_others(clients, sender_id, message);
            }
            Payload::Shoot(_, _, _, _, _) => {
                // Broadcast bullet to all other clients
                Self::broadcast_to_others(clients, sender_id, message);
            }
            Payload::BossShoot(_, _, _, _) => {
                // Broadcast boss bullet to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::PlayerHit(_, _) => {
                // Broadcast player hit to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossHit(_) => {
                // Broadcast boss hit to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossSpawn(_, _) => {
                // Broadcast boss spawn to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossDead => {
                // Broadcast boss death to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossMultiShoot(_, _, _) => {
                // Broadcast boss multi-shot to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossDash(_, _) => {
                // Broadcast boss dash to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossAreaAttack(_, _) => {
                // Broadcast boss area attack to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::BossShield(_) => {
                // Broadcast boss shield to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::PlayerRespawn(_, _, _) => {
                // Broadcast player respawn to all clients
                Self::broadcast_to_all(clients, message);
            }
            Payload::PlayerDirection(_, _, _) => {
                // Broadcast player direction to other clients
                Self::broadcast_to_others(clients, sender_id, message);
            }
            Payload::PlayerKill(_, _) => {
                // Broadcast player kill to all clients
                Self::broadcast_to_all(clients, message);
            }
        }
    }

    fn send_current_state(
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        positions: &Arc<Mutex<HashMap<u32, PlayerState>>>,
        new_player_id: u32,
    ) {
        let positions = positions.lock().unwrap();
        let mut clients = clients.lock().unwrap();

        if let Some(stream) = clients.get_mut(&new_player_id) {
            for (&player_id, state) in positions.iter() {
                if player_id != new_player_id {
                    // Send Join message for existing player
                    if let Ok(join_data) = bincode::serialize(&Payload::Join(player_id)) {
                        let len = (join_data.len() as u32).to_le_bytes();
                        let _ = stream.write_all(&len);
                        let _ = stream.write_all(&join_data);
                    }

                    // Send Move message for existing player's position
                    if let Ok(move_data) =
                        bincode::serialize(&Payload::Move(player_id, state.x, state.y))
                    {
                        let len = (move_data.len() as u32).to_le_bytes();
                        let _ = stream.write_all(&len);
                        let _ = stream.write_all(&move_data);
                    }
                }
            }
            let _ = stream.flush();
        }
    }

    fn handle_disconnect(
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        positions: &Arc<Mutex<HashMap<u32, PlayerState>>>,
        client_id: u32,
    ) {
        // Remove from clients and positions
        {
            let mut clients = clients.lock().unwrap();
            clients.remove(&client_id);
        }
        {
            let mut positions = positions.lock().unwrap();
            positions.remove(&client_id);
        }

        // Broadcast leave message to remaining clients
        if let Ok(leave_data) = bincode::serialize(&Payload::Leave(client_id)) {
            let len = (leave_data.len() as u32).to_le_bytes();
            let mut message = Vec::new();
            message.extend_from_slice(&len);
            message.extend_from_slice(&leave_data);
            Self::broadcast_to_all(clients, &message);
        }
    }

    fn broadcast_to_others(
        clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
        sender_id: u32,
        message: &[u8],
    ) {
        let mut clients = clients.lock().unwrap();
        let mut disconnected_clients = Vec::new();

        for (&client_id, stream) in clients.iter_mut() {
            if client_id != sender_id
                && (stream.write_all(message).is_err() || stream.flush().is_err())
            {
                disconnected_clients.push(client_id);
            }
        }

        for client_id in disconnected_clients {
            clients.remove(&client_id);
        }
    }

    fn broadcast_to_all(clients: &Arc<Mutex<HashMap<u32, TcpStream>>>, message: &[u8]) {
        let mut clients = clients.lock().unwrap();
        let mut disconnected_clients = Vec::new();

        for (&client_id, stream) in clients.iter_mut() {
            if stream.write_all(message).is_err() || stream.flush().is_err() {
                disconnected_clients.push(client_id);
            }
        }

        for client_id in disconnected_clients {
            clients.remove(&client_id);
        }
    }
}

fn main() -> std::io::Result<()> {
    // Start the server
    let Args { port } = argh::from_env::<Args>();
    Server::new().start(&format!("0.0.0.0:{port}"))
}

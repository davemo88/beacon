
use rand;
use bincode;
use chrono;
use std::{fs, io};
use std::io::{Read, Write};
use ed25519_dalek::{Keypair, Signature, PublicKey};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
use serde::{Serialize, Deserialize};

struct Message {
    beacon_state: bool,
    broadcast_time: chrono::DateTime<chrono::Utc>,
    pubkey: PublicKey,
    sig: Signature,
}

#[derive(Serialize, Deserialize)]
struct Beacon {
    name: String,
    keypair: Keypair,
}

fn new_beacon(name: &String) -> Beacon {
    println!("creating new beacon {}", name);

    let mut csprng = rand::rngs::OsRng{};

    let b = Beacon {
        name: name.to_string(),
        keypair: Keypair::generate(&mut csprng),
    };

    println!("new beacon created");
    println!("saving new beacon");

    save_beacon(&b).unwrap();

    println!("new beacon saved");

    b
}

fn save_beacon(b: &Beacon) -> io::Result<()> {

    let encoded_beacon: Vec<u8> = bincode::serialize(&b).unwrap();
    let mut buffer = io::BufWriter::new(fs::File::create(format!("{}.bcn", b.name))?);
    buffer.write_all(&encoded_beacon)?;
    buffer.flush()?;
    Ok(())
}

//fn load_beacon(name: String) -> Result<Beacon, io::Error> {
fn load_beacon(name: &String) -> Beacon {
    
    println!("loading beacon {}", name);
    let mut f = fs::File::open(format!("{}.bcn", name)).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer);

    let b: Beacon = bincode::deserialize(&buffer).unwrap();

    println!("beacon loaded");
    b
}

fn delete_beacon(name: &String) -> io::Result<()> {
    fs::remove_file(format!("{}.bcn", name))?;
    Ok(())
}

fn create_message(beacon: &Beacon, beacon_state: bool) -> Message 
{
    let broadcast_time = chrono::Utc::now();

    let mut msg = Message {
        beacon_state: beacon_state,
        broadcast_time: broadcast_time, 
        pubkey: beacon.keypair.public,
//        sig: sig,
        sig: beacon.keypair.sign(b"bogus"),
    };

    msg.sig = beacon.keypair.sign(&get_bytes_to_sign(&msg));

    msg
}

fn get_bytes_to_sign(message: &Message) -> Vec<u8> {
    let mut for_signing: Vec<u8> = bincode::serialize(&message.broadcast_time.timestamp()).unwrap(); 
    for_signing.push(message.beacon_state as u8);
    for_signing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon() {

        let name = String::from("test");
    
        let b = new_beacon(&name);
    
        let lb = load_beacon(&name);
    
        let msg = create_message(&b, true);

        assert!(lb.keypair.public.verify(&get_bytes_to_sign(&msg), &msg.sig).is_ok());

        delete_beacon(&b.name);
    }
}

fn main() {

}

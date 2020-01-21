use rand;
use bincode;
use chrono;
use std::{fs, io};
use std::io::{Read, Write};
use ed25519_dalek::{Keypair, Signature, PublicKey};
use serde::{Serialize, Deserialize};

pub const CLI_TCP_ADDRESS: &'static str = "0.0.0.0:3333";

#[derive(Serialize, Deserialize)]
pub enum Command {
    Create(String),
    Delete(String),
    Broadcast(String,bool),
    Subscribe { name: String, pubkey: String },
    Unsubscribe(String),
    ListBeacons(),
    ListSubs(),
}

#[derive(Serialize, Deserialize)]
pub struct Beacon {
    pub name: String,
    pub keypair: Keypair,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub beacon_state: bool,
    pub broadcast_time: i64,
    pub pubkey: PublicKey,
    pub sig: Signature,
}

#[derive(Serialize, Deserialize)]
pub struct BeaconSub {
    pub name: String,
    pub pubkey: PublicKey,
    pub last_message: Option<Message>,
}

impl Beacon {

    pub fn new(name: &String) -> Beacon {
        println!("creating new beacon {}", name);
    
        let mut csprng = rand::rngs::OsRng{};
    
        let b = Beacon {
            name: name.to_string(),
            keypair: Keypair::generate(&mut csprng),
        };
    
        println!("new beacon created");
    
        b
    }

    pub fn save(&self) -> io::Result<()> {

        println!("saving beacon");

        let encoded_beacon: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut buffer = io::BufWriter::new(fs::File::create(format!("{}.bcn", self.name))?);
        buffer.write_all(&encoded_beacon)?;
        buffer.flush()?;
    
        println!("beacon saved");
    
        Ok(())
    }

//fn load_beacon(name: String) -> Result<Beacon, io::Error> {
    pub fn load(name: &String) -> Beacon {
        
        println!("loading beacon {}", name);

        let mut f = fs::File::open(format!("{}.bcn", name)).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
    
        let b: Beacon = bincode::deserialize(&buffer).unwrap();
    
        println!("beacon loaded");

        b
    }

    pub fn delete(name: &String) -> io::Result<()> {
        fs::remove_file(format!("{}.bcn", name))?;
        Ok(())
    }

    pub fn create_message(&self, beacon_state: bool) -> Message 
    {
        let broadcast_time = chrono::Utc::now().timestamp();
    
        let mut msg = Message {
            beacon_state: beacon_state,
            broadcast_time: broadcast_time, 
            pubkey: self.keypair.public,
            sig: self.keypair.sign(b"bogus"),
        };
    
        msg.sig = self.keypair.sign(&msg.get_bytes_to_sign());
    
        msg
    }

}

impl Message {
    pub fn get_bytes_to_sign(&self) -> Vec<u8> {
        let mut bytes_to_sign: Vec<u8> = bincode::serialize(&self.broadcast_time).unwrap(); 
        bytes_to_sign.push(self.beacon_state as u8);
        bytes_to_sign
    }
}

impl BeaconSub {
    pub fn save(&self) -> io::Result<()> {

        println!("saving beacon sub");

        let encoded_beacon_sub: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut buffer = io::BufWriter::new(fs::File::create(format!("{}.bcs", self.name))?);
        buffer.write_all(&encoded_beacon_sub)?;
        buffer.flush()?;
    
        println!("beacon sub saved");
    
        Ok(())
    }

//fn load_beacon(name: String) -> Result<Beacon, io::Error> {
    pub fn load(name: &String) -> BeaconSub {
        
        println!("loading beacon sub {}", name);

        let mut f = fs::File::open(format!("{}.bcs", name)).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
    
        let bs: BeaconSub = bincode::deserialize(&buffer).unwrap();
    
        println!("beacon loaded");

        bs
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon() {

        let name = String::from("test");
    
        let b = Beacon::new(&name);

        b.save().unwrap();
    
        let lb = Beacon::load(&name);
    
        let msg = b.create_message(true);

        assert!(lb.keypair.public.verify(&msg.get_bytes_to_sign(), &msg.sig).is_ok());

        Beacon::delete(&b.name).unwrap();
    }
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Request {}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Response {}


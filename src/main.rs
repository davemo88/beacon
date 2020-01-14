
use rand;
use bincode;
use chrono;
use std::{fs, io};
use std::io::{Read, Write};
use ed25519_dalek::{Keypair, Signature, PublicKey};
//use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
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

impl Beacon {

    fn new(name: &String) -> Beacon {
        println!("creating new beacon {}", name);
    
        let mut csprng = rand::rngs::OsRng{};
    
        let b = Beacon {
            name: name.to_string(),
            keypair: Keypair::generate(&mut csprng),
        };
    
        println!("new beacon created");
    
        b
    }

    fn save(&self) -> io::Result<()> {

        println!("saving beacon");

        let encoded_beacon: Vec<u8> = bincode::serialize(&self).unwrap();
        let mut buffer = io::BufWriter::new(fs::File::create(format!("{}.bcn", self.name))?);
        buffer.write_all(&encoded_beacon)?;
        buffer.flush()?;
    
        println!("beacon saved");
    
        Ok(())
    }

//fn load_beacon(name: String) -> Result<Beacon, io::Error> {
    fn load(name: &String) -> Beacon {
        
        println!("loading beacon {}", name);

        let mut f = fs::File::open(format!("{}.bcn", name)).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer);
    
        let b: Beacon = bincode::deserialize(&buffer).unwrap();
    
        println!("beacon loaded");

        b
    }

    fn delete(name: &String) -> io::Result<()> {
        fs::remove_file(format!("{}.bcn", name))?;
        Ok(())
    }

    fn create_message(&self, beacon_state: bool) -> Message 
    {
        let broadcast_time = chrono::Utc::now();
    
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
        let mut bytes_to_sign: Vec<u8> = bincode::serialize(&self.broadcast_time.timestamp()).unwrap(); 
        bytes_to_sign.push(self.beacon_state as u8);
        bytes_to_sign
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beacon() {

        let name = String::from("test");
    
        let b = Beacon::new(&name);

        b.save();
    
        let lb = Beacon::load(&name);
    
        let msg = b.create_message(true);

        assert!(lb.keypair.public.verify(&msg.get_bytes_to_sign(), &msg.sig).is_ok());

        Beacon::delete(&b.name);
    }
}

fn main() {

}

use std::iter::FromIterator;
use std::net;
use std::io::prelude::*;

#[path = "beacon.rs"] mod beacon;

use crate::beacon::Command;

struct BeaconCli {
//    pub args: std::env::Args,
    args: Vec<String>,
}

impl BeaconCli {

    fn get_command(&mut self) -> Command {
        match &self.args[1][..] {
            "create" => self.create(),
            "delete" => self.delete(),
            "broadcast" => self.broadcast(),
            "subscribe" => self.subscribe(),
            "unsubscribe" => self.unsubscribe(),
            _ => panic!("invalid command"),
        }
    }

    fn create(&mut self) -> Command {
        let name = &self.args[2];
        println!("creating beacon {}", name);
        Command::Create(name.to_string())
    }

    fn delete(&mut self) -> Command {
        let name = &self.args[2];
        println!("deleting beacon {}", name);
        Command::Delete(name.to_string())
    }

    fn broadcast(&mut self) -> Command {
        let name = &self.args[2]; 
        let state: bool  = self.args[3].parse().unwrap();
        println!("broadcasting status {} for beacon {}", state, name);
        Command::Broadcast(name.to_string(), state)
    }

    fn subscribe(&mut self) -> Command {
        let name = &self.args[2];
        let pubkey = &self.args[3];

        println!("subscribing to beacon {} with pubkey {}", name, pubkey);
        Command::Subscribe {
            name: name.to_string(),
            pubkey: pubkey.to_string(),
        }
    }

    fn unsubscribe(&mut self) -> Command {
        let name = &self.args[2];
        println!("unsubscribing from beacon {}", name);
        Command::Unsubscribe(name.to_string())
    }

}

// broadcast if a beacon name is passed
pub fn main()
{
    println!("running cli test");

    let mut bcli = BeaconCli {
        args: Vec::from_iter(std::env::args()),
    };

    println!("{:?}", bcli.args);

    let c: Command = bcli.get_command();
    
    let mut stream = net::TcpStream::connect(beacon::TCP_ADDRESS).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{}", response);

}

use pgp::composed::{key, signed_key};
use pgp::ser::Serialize;
use std::io::Write;

// should I give them a body, e.g. a name for new beacons?
enum MessageType {
    BeaconOn,
    BeaconOff,
}

struct Message {
    message_type: MessageType,
    broadcast_time: String,
    pubkey: String,
    sig: String,
}

struct Beacon {
    name: String,
    signed_key: signed_key::SignedSecretKey,
}

//fn broadcast_message(msg: &Message) -> ()
//{
////TODO: verify signature before broadcasting
//
//}
//
//fn sign_message(msg: &mut Message) -> ()
//{
//
//}

fn new_beacon(name: String) {
    println!("creating new beacon {}", name);
    let key_params = key::SecretKeyParamsBuilder::default()
        .key_type(key::KeyType::EdDSA)
        .can_sign(true)
        .primary_user_id(String::from("this_user"))
        .build()
        .unwrap();

// https://github.com/rpgp/rpgp/blob/348b7c62bb09a188274d3bf659db20b861748d88/pgp-ffi/src/secret_key.rs#L159
    let signed_key = key_params.generate().unwrap().sign(|| "".into()).unwrap();

    println!("new beacon created");
    println!("saving new beacon");

    let b = Beacon {
        name: name,
        signed_key: signed_key,
    };

    save_beacon(&b);

    println!("new beacon saved");
}

fn load_beacon(name: String) {

}

fn save_beacon(b: &Beacon) -> std::io::Result<()> {

    let mut buffer = std::io::BufWriter::new(std::fs::File::create("test.bcn")?);
    b.signed_key.to_writer(&mut buffer);
    buffer.flush()?;
    Ok(())

}

//    let new = Message {
//        message_type: MessageType::BeaconOff,
//        broadcast_time: String::from("Now"),
//        pubkey: String::from("pubkey"),
//        sig: String::from("sig"),
//    };

fn main() {

    let new = new_beacon(String::from("new beacon"));

}

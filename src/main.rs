mod beacon;
mod p2p;
mod cli;

fn main() {
    p2p::beacon_p2p().unwrap();
//    cli::main();
}

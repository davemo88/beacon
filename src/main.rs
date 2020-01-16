mod beacon;
mod p2p;
mod cli;

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        println!("test_main");
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    p2p::beacon_p2p()
}

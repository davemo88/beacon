mod beacon;
mod p2p;

#[cfg(test)]
mod tests {

    #[test]
    fn test_main() {
        println!("test_main");
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    p2p::chat_example()
}

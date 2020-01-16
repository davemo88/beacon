use crate::beacon;
use beacon::Command;

struct BeaconCli {
    socket_address: String,
}

impl BeaconCli {

    fn create(args: std::env::Args) -> beacon::Command {

        if let Some(name) = args.nth(2) {
            Ok(Command::Create(name.parse()));
        }
    }
}

// broadcast if a beacon name is passed
fn cli_main()
{
    let args = std::env::args();
    if let Some(command) = args.nth(1) {
        let command: String = command.parse();
//        match command:
//            "create" => println("{:?}", args);
    }

}


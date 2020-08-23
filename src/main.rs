use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};

fn main() {
    let server_arg = Arg::with_name("server")
        .short("s")
        .long("server")
        .value_name("SERVER")
        .help("The server to be used")
        .takes_value(true);

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("version")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Get the current version of the synapse server")
                .arg(&server_arg),
        )
        .subcommand(
            SubCommand::with_name("user")
                .author(crate_authors!())
                .version(crate_version!())
                .about("modify the users of a given synapse server")
                .arg(&server_arg),
        )
        .get_matches();

    println!("{:?}", matches);

    if let Some(matches) = matches.subcommand_matches("version") {
        match matrixapi::get_server_version(matches.value_of("server")) {
            Ok(_) => (),
            Err(e) => eprintln!("Something went wrong: {}", e),
        };
    }

    if let Some(matches) = matches.subcommand_matches("user") {
        match matrixapi::get_user_list(matches.value_of("server")) {
            Ok(_) => (),
            Err(e) => eprintln!("Something went wrong: {}", e),
        };
    }
}

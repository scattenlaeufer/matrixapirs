use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};

fn exit_program(result: Result<(), matrixapi::MatrixAPIError>) {
    match result {
        Ok(_) => std::process::exit(exitcode::OK),
        Err(e) => {
            eprintln!("Something went wrong: {}", e);
            std::process::exit(exitcode::USAGE);
        }
    };
}

fn main() {
    let debug_arg = Arg::with_name("debug")
        .short("d")
        .long("debug")
        .help("Print debug output")
        .takes_value(false);

    let server_arg = Arg::with_name("server")
        .short("s")
        .long("server")
        .value_name("SERVER")
        .help("The server to be used")
        .takes_value(true);

    let json_arg = Arg::with_name("json")
        .short("j")
        .long("json")
        .help("Return data as json instead of a table")
        .takes_value(false);

    let user_name_arg = Arg::with_name("user_name")
        .value_name("USER_NAME")
        .required(true)
        .help("Name of a specific user");

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(&debug_arg)
        .subcommand(
            SubCommand::with_name("version")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Get the current version of the synapse server")
                .arg(&server_arg)
                .arg(&json_arg),
        )
        .subcommand(
            SubCommand::with_name("user")
                .author(crate_authors!())
                .version(crate_version!())
                .about("Interact the users of a given synapse server")
                .subcommand(
                    SubCommand::with_name("list")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Get a list of all users on a given synapse server")
                        .arg(&server_arg)
                        .arg(&json_arg),
                )
                .subcommand(
                    SubCommand::with_name("detail")
                        .author(crate_authors!())
                        .version(crate_version!())
                        .about("Get a list of all users on a given synapse server")
                        .arg(&server_arg)
                        .arg(&user_name_arg)
                        .arg(&json_arg),
                ),
        )
        .get_matches();

    if matches.is_present("debug") {
        println!("{:?}", matches);
    }

    if let Some(matches) = matches.subcommand_matches("version") {
        exit_program(matrixapi::get_server_version(
            matches.value_of("server"),
            matches.is_present("json"),
        ));
    }

    if let Some(matches) = matches.subcommand_matches("user") {
        if let Some(matches) = matches.subcommand_matches("list") {
            exit_program(matrixapi::get_user_list(
                matches.value_of("server"),
                matches.is_present("json"),
            ));
        };
        if let Some(matches) = matches.subcommand_matches("detail") {
            exit_program(matrixapi::get_user_detail(
                matches.value_of("server"),
                matches.value_of("user_name").unwrap(),
                matches.is_present("json"),
            ));
        };
    }
}

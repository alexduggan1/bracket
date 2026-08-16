use std::{fs::File, io::{BufReader, Read, Write}};
use chrono::Weekday;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Commands
    #[command(subcommand)]
    command: Option<Commands>,

    /// Other
    other: Vec<Option<String>>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a bracket
    Add(AddArgs),
    
    /// All brackets
    All,

    /// Brackets from today
    Today,
}

#[derive(Args)]
struct AddArgs {
    /// The name to type in to access the bracket
    name: String,

    /// The days of the week for this bracket
    #[arg(short, long, value_parser = clap::value_parser!(Weekday))]
    days: Option<Vec<Weekday>>,

    /// Link to the bracket on start.gg
    startgg: String,

    /// Links to the streams for this bracket
    streams: Vec<String>,
}

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if cli.command.is_some() {
        match &cli.command.unwrap() {
            Commands::Add(args) => {
                println!("'myapp add' was used, name is: {:?}", args.name);

                match store_config(args) {
                    Ok(_) => println!("added {:?}", args.name),
                    Err(_) => eprintln!("failed to add {:?}", args.name),
                }
            }
            Commands::All => todo!(),
            Commands::Today => todo!(),
        }
    }
    else {
        if cli.other.len() != 0 {
            let mut i = 0;
            for arg in cli.other {
                if arg.is_some() {
                    let other = arg.unwrap();

                    println!("other: {:?}", other);
                }
                i += 1;
            }
        }
        else {
            println!("not work");
        }
    }
}

fn store_config(to_add: &AddArgs) -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");

    let mut new_content = String::from("");
    new_content.push_str(&to_add.name);
    if to_add.days.is_some() {
            for day in &to_add.days.clone().unwrap() {
                println!("day: {:?}", day);
                new_content.push_str(" -d=");
                new_content.push_str(&day.to_string());
            }
        }
    new_content.push(' ');
    new_content.push_str(&to_add.startgg);
    for stream in &to_add.streams {
        new_content.push(' ');
        new_content.push_str(&stream);
    }

    println!("new content: {:?}", new_content);

    let config_path: std::path::PathBuf;
    let config_path_option = xdg_dirs.find_config_file("brackets.txt");
    if config_path_option.is_some() {
        // there is already a config path
        config_path = config_path_option.unwrap();
        let mut contents = String::new();
        {
            let file = File::open(&config_path)?;
            let mut buf_reader = BufReader::new(file);
            buf_reader.read_to_string(&mut contents)?;
        }
        println!("contents: {:?}", contents);

        contents.push_str("\n");
        contents.push_str(&new_content);


        let mut config_file = File::create(config_path)?;
        config_file.write_all(contents.as_bytes())?;
    }
    else {
        // there is not already a config path

        config_path = xdg_dirs
            .place_config_file("brackets.txt")
            .expect("cannot create configuration directory");
        let mut config_file = File::create(config_path)?;
        config_file.write_all(new_content.as_bytes())?;
    }
    
    
    Ok(())
}


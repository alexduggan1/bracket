use std::{fs::File, io::{BufReader, Read, Write}, str::FromStr};
use chrono::{Weekday, Datelike};
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Commands
    #[command(subcommand)]
    command: Option<Commands>,

    /// Retrieve info about specific bracket(s)
    other: Vec<Option<String>>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a bracket
    Add(Bracket),

    /// Remove a bracket
    Remove(RemoveArgs),
    
    /// Edit a bracket
    Edit(EditArgs),

    /// Retrieve info about all brackets
    All,

    /// Retrieve info about brackets from today
    Today,

    /// Retrieve info about brackets from a specific day
    Day(DayArgs),
}

#[derive(Args, Clone)]
struct Bracket {
    /// The name to type in to access the bracket
    name: String,

    /// The days of the week for this bracket (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
    #[arg(short, long, value_parser = clap::value_parser!(Weekday))]
    days: Option<Vec<Weekday>>,

    /// Link to the bracket
    link: String,

    /// Links to the streams for this bracket
    streams: Vec<String>,
}

impl Bracket {
    pub fn new(name: String, days: Option<Vec<Weekday>>, link: String, streams:Vec<String>) -> Self {
        Self {
            name,
            days,
            link,
            streams,
        }
    }
}

#[derive(Args)]
struct DayArgs {
    /// The day to search for (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
    day: Weekday,
}

#[derive(Args)]
struct RemoveArgs {
    /// The name of the bracket
    name: String,
}

#[derive(Args, Clone)]
struct EditArgs {
    /// The name of the bracket
    name: String,

    /// Commands
    #[command(subcommand)]
    command: EditCommands,
}

#[derive(Subcommand, Clone)]
enum EditCommands {
    /// Rename the bracket
    Rename(EditRenameArgs),

    /// Change link
    ChangeLink(EditChangeLinkArgs),

    /// Add days
    AddDays(EditAddDaysArgs),
    
    /// Remove days
    RemoveDays(EditRemoveDaysArgs),
    
    /// Clear days
    ClearDays,

    /// Add streams
    AddStreams(EditAddStreamsArgs),

    /// Remove streams
    RemoveStreams(EditRemoveStreamsArgs),

    /// Clear streams
    ClearStreams,
}

#[derive(Args, Clone)]
struct EditRenameArgs {
    /// The name of the bracket
    name: String,
}

#[derive(Args, Clone)]
struct EditChangeLinkArgs {
    /// Link to the bracket
    link: String,
}

#[derive(Args, Clone)]
struct EditAddDaysArgs {
    /// Days to add
    days: Vec<Weekday>,
}

#[derive(Args, Clone)]
struct EditRemoveDaysArgs {
    /// Days to remove
    days: Vec<Weekday>,
}

#[derive(Args, Clone)]
struct EditAddStreamsArgs {
    /// Streams to add
    streams: Vec<String>,
}

#[derive(Args, Clone)]
struct EditRemoveStreamsArgs {
    /// Streams to remove
    streams: Vec<String>,
}



fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if cli.command.is_some() {
        match &cli.command.unwrap() {
            Commands::Add(args) => {
                println!("'add' was used, name is: {:?}", args.name);

                match add_bracket(args) {
                    Ok(_) => println!("added {:?}", args.name),
                    Err(_) => eprintln!("failed to add {:?}", args.name),
                }
            },
            Commands::Remove(args) => {
                //println!("'remove' was used, name is: {:?}", args.name);

                match remove_bracket(args.name.clone()) {
                    Ok(_) => println!("removed {:?}", args.name),
                    Err(_) => eprintln!("failed to remove {:?}", args.name),
                }
            },
            Commands::Edit(args) => {
                //println!("'edit' was used, name is: {:?}", args.name);

                match edit_bracket(args.clone()) {
                    Ok(_) => println!(""),
                    Err(_) => eprintln!("failed to edit bracket"),
                }
            },
            Commands::All => {
                //println!("'all' was used");

                let retrieved_brackets = retrieve_brackets();
                match retrieved_brackets {
                    Ok(brackets) => display_brackets(brackets, Commands::All, None),
                    Err(_) => eprintln!("failed to retrieve brackets"),
                }
            },
            Commands::Today => {
                //println!("'today' was used");

                let current_day = chrono::offset::Local::now().date_naive().weekday();

                let retrieved_brackets = retrieve_brackets();
                match retrieved_brackets {
                    Ok(brackets) => display_brackets(brackets, Commands::Today, Some(current_day)),
                    Err(_) => eprintln!("failed to retrieve brackets"),
                }
            },
            Commands::Day(args) => {
                let retrieved_brackets = retrieve_brackets();
                match retrieved_brackets {
                    Ok(brackets) => display_brackets(brackets, Commands::Today, Some(args.day)),
                    Err(_) => eprintln!("failed to retrieve brackets"),
                }
            },
        }
    }
    else {
        if cli.other.len() != 0 {
            let retrieved_brackets = retrieve_brackets();
            match retrieved_brackets {
                Ok(brackets) => {
                    for arg in cli.other {
                        if arg.is_some() {
                            let name_to_search = arg.unwrap();

                            let bracket = brackets
                            .iter()
                            .find(|a| a.name.to_lowercase() == name_to_search.to_lowercase());
                            
                            match bracket {
                                Some(_) => display_bracket(bracket.unwrap().clone()),
                                None => println!("could not find bracket \"{}\"", name_to_search),
                            }
                        }
                    }
                },
                Err(_) => eprintln!("failed to retrieve brackets"),
            }
        }
        else {
            println!("Run bracket -h for help");
        }
    }
}

fn add_bracket(to_add: &Bracket) -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");

    let mut new_content = String::from("");
    new_content.push_str(&to_add.name);
    if to_add.days.is_some() {
            for day in &to_add.days.clone().unwrap() {
                new_content.push_str(" -d=");
                new_content.push_str(&day.to_string());
            }
        }
    new_content.push(' ');
    new_content.push_str(&to_add.link);
    for stream in &to_add.streams {
        new_content.push(' ');
        new_content.push_str(&stream);
    }

    //println!("new content: {:?}", new_content);

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
        //println!("contents: {:?}", contents);

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

fn remove_bracket(to_remove: String) -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");


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

        let mut new_contents: String = String::new();
        let check = to_remove.clone() + " ";
        let lines = contents.lines();
        for line in lines {
            if !line.starts_with(&check) {
                new_contents.push_str(line);
                new_contents.push_str("\n");
            }
        }
        new_contents.pop();

        let mut config_file = File::create(config_path)?;
        config_file.write_all(new_contents.as_bytes())?;
    }
    else {
        // there is not already a config path

        config_path = xdg_dirs
            .place_config_file("brackets.txt")
            .expect("cannot create configuration directory");
        File::create(config_path)?;
    }


    Ok(())
}

fn edit_bracket(args: EditArgs) -> std::io::Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");

    let to_remove = args.name;

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
        
        let mut new_contents: String = String::new();
        let mut old_contents: String = String::from("");
        let check = to_remove.clone() + " ";
        let lines = contents.lines();
        for line in lines {
            if !line.starts_with(&check) {
                new_contents.push_str(line);
                new_contents.push_str("\n");
            }
            else {
                old_contents.push_str(line);
            }
        }
        
        if old_contents == "" {
            println!("bracket \"{}\" not found", to_remove);
            return Ok(());
        }
        // construct bracket from old_contents
        let mut bracket = Bracket::new("".to_string(), Some(Vec::new()), "".to_string(), Vec::new());
        let mut i = 0;
        for word in old_contents.split_whitespace() {
            if word.starts_with("-d=") {
                let day_word = word.strip_prefix("-d=").unwrap();
                let day_result = chrono::Weekday::from_str(day_word);
                
                if let Some(ref mut vec) = bracket.days {
                    vec.push(day_result.unwrap());
                }
            }
            else {
                match i {
                    0 => { bracket.name = word.to_string(); },
                    1 => { bracket.link = word.to_string(); },
                    _ => { bracket.streams.push(word.to_string()); },
                }

                i += 1;
            }
        }


        match args.command {
            EditCommands::Rename(edit_rename_args) => {
                bracket.name = edit_rename_args.name;
            },
            EditCommands::ChangeLink(edit_change_link_args) => {
                bracket.name = edit_change_link_args.link;
            },
            EditCommands::AddDays(edit_add_days_args) => {
                if bracket.days.is_some() {
                    let mut days = bracket.days.clone().unwrap();

                    for day in edit_add_days_args.days {
                        if !days.contains(&day) {
                            days.push(day);
                        }
                    }

                    bracket.days = Some(days);
                }
                else {
                    bracket.days = Some(edit_add_days_args.days);
                }
            },
            EditCommands::RemoveDays(edit_remove_days_args) => {
                if bracket.days.is_some() {
                    let mut days = bracket.days.clone().unwrap();

                    days.retain(|a| !edit_remove_days_args.days.contains(a));

                    bracket.days = Some(days);
                }
            },
            EditCommands::ClearDays => {
                bracket.days = None;
            },
            EditCommands::AddStreams(edit_add_streams_args) => {
                for stream in edit_add_streams_args.streams {
                    if !bracket.streams.contains(&stream) {
                        bracket.streams.push(stream);
                    }
                }
            },
            EditCommands::RemoveStreams(edit_remove_streams_args) => {
                bracket.streams.retain(|a| !edit_remove_streams_args.streams.contains(a));
            },
            EditCommands::ClearStreams => {
                bracket.streams.clear();
            },
        }

        // form new string from bracket
        let mut edited_contents: String = String::new();

        edited_contents.push_str(&bracket.name);
        if bracket.days.is_some() {
                for day in &bracket.days.clone().unwrap() {
                    edited_contents.push_str(" -d=");
                    edited_contents.push_str(&day.to_string());
                }
            }
        edited_contents.push(' ');
        edited_contents.push_str(&bracket.link);
        for stream in &bracket.streams {
            edited_contents.push(' ');
            edited_contents.push_str(&stream);
        }

        //println!("edited contents: {:?}", edited_contents);

        new_contents.push_str(&edited_contents);

        let mut config_file = File::create(config_path)?;
        config_file.write_all(new_contents.as_bytes())?;
    }
    else {
        // there is not already a config path

        config_path = xdg_dirs
            .place_config_file("brackets.txt")
            .expect("cannot create configuration directory");
        File::create(config_path)?;
    }


    Ok(())
}

fn retrieve_brackets() -> std::io::Result<Vec<Bracket>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("bracket");

    let mut brackets: Vec<Bracket> = Vec::new();

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
        //println!("contents: {:?}", contents);
        
        for line in contents.lines() {
            let mut bracket = Bracket::new("".to_string(), Some(Vec::new()), "".to_string(), Vec::new());
            //bracket.days = Some(Vec::new());
            //bracket.streams = Vec::new();

            let mut i = 0;
            for word in line.split_whitespace() {
                if word.starts_with("-d=") {
                    let day_word = word.strip_prefix("-d=").unwrap();
                    let day_result = chrono::Weekday::from_str(day_word);
                    
                    if let Some(ref mut vec) = bracket.days {
                        vec.push(day_result.unwrap());
                    }
                }
                else {
                    match i {
                        0 => { bracket.name = word.to_string(); },
                        1 => { bracket.link = word.to_string(); },
                        _ => { bracket.streams.push(word.to_string()); },
                    }

                    i += 1;
                }
            }

            brackets.push(bracket);
        }

    }

    let mut sorted_brackets = brackets.clone();
    sorted_brackets.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(sorted_brackets)
}

fn display_brackets(brackets: Vec<Bracket>, command: Commands, current_day: Option<Weekday>) {
    match command {
        Commands::All => {
            for bracket in brackets {
                display_bracket(bracket);
            }
        },
        Commands::Today => {
            let filtered_brackets: Vec<Bracket> = brackets.clone()
            .into_iter()
            .filter(|a| a.days.clone().unwrap().contains(&current_day.unwrap()))
            .collect();

            for bracket in filtered_brackets {
                display_bracket(bracket);
            }
        },
        _ => eprintln!("invalid display command"),
    }
}

fn display_bracket(bracket: Bracket) {
    print!("{}", bracket.name);
    if bracket.days.is_some() {
        print!(" (");
        let days_clone = bracket.days.clone().unwrap();
        let mut days = days_clone.iter().peekable();
        while let Some(day) = days.next()  {
            if days.peek().is_none() {
                // last one
                print!("{}", 
                    match day {
                        Weekday::Mon => "Monday",
                        Weekday::Tue => "Tuesday",
                        Weekday::Wed => "Wednesday",
                        Weekday::Thu => "Thursday",
                        Weekday::Fri => "Friday",
                        Weekday::Sat => "Saturday",
                        Weekday::Sun => "Sunday",
                    }.to_string()
                );
            }
            else {
                print!("{}, ", 
                    match day {
                        Weekday::Mon => "Monday",
                        Weekday::Tue => "Tuesday",
                        Weekday::Wed => "Wednesday",
                        Weekday::Thu => "Thursday",
                        Weekday::Fri => "Friday",
                        Weekday::Sat => "Saturday",
                        Weekday::Sun => "Sunday",
                    }.to_string()
                );
            }
        }
        print!(")\n");
    }
    println!("           link\t{}", bracket.link);
    
    if bracket.streams.len() != 0 {
        let mut streams = bracket.streams.iter().peekable();
        print!("        streams\t");
        while let Some(stream) = streams.next()  {
            if streams.peek().is_none() {
                // last one
                print!("{}\n", stream);
            }
            else {
                print!("{}\n\t\t", stream);
            }
        }
    }
    println!("");
}

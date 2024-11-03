mod util;

use std::{env, fs, path::Path, process};
use util::{config::{self, new_config, Config, ConfigError}, init::{self, InitParams, OperationType, ProjectSetup}};

fn main() {
    let args: Vec<String> = env::args().collect();
    let config : Config;
    let old_config : Option<Config> = if Path::new("config.toml").exists() {
        let config_result = Config::read_config("config.toml");
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                eprintln!("Line {}: Problem opening the file: {}", line!(), error);
                std::process::exit(2);
            }
        };
        Some(config)
    } else {
        None
    };
    let operation_type;

    if &args.len() == &1usize {
        println!("TODO: Help DOCUMENTATION");
            process::exit(0);
    }

    config = match &args[1][..] {
        "new" => {
            operation_type = OperationType::New;
            config::parse_to_config(args, false)
        },
        "update" => {
            operation_type = OperationType::Update;
            config::parse_to_config(args, true)
        },
        _ => {
            println!("TODO: Help DOCUMENTATION");
            process::exit(0);
        },
    };

    // dbg!(&config);
    setup(old_config, config, operation_type);
}

fn setup(old_config_option: Option<Config>, config: Config, op_type: OperationType){
    let mut old_config = new_config();
    let mut old_config_exists = false;
    match old_config_option {
        Some(config) => {old_config = config; old_config_exists = true},
        None => {},
    };

    let old_setup = &old_config.setup;
    let setup = &config.setup;

    {
        let main_folder_result : std::result::Result<(), ConfigError> = match &setup.deadname {
            Some(deadname) => initialize_main_folder_deadname(&deadname, &setup),
            None => initialize_main_folder(&old_setup, &setup, &op_type, old_config_exists),
        };

        match main_folder_result {
            Ok(()) => {
                old_config.setup.name = setup.name.clone();
                // Regardless of whether old config exists or not (if it didn't it would get default values), edit the name attribute and then write the config to file.
                // UPDATE NAME VALUE IN CONFIG
                let write_config_result = Config::write_config(&old_config, "config.toml");
                match write_config_result {
                    Ok(file) => file,
                    Err(error) => {
                        eprintln!("Line {}: Problem opening the file: {}", line!(), error);
                        std::process::exit(1);
                    },
                };
            },
            Err(error) => {
                eprintln!("Line {}: Problem creating/editing files: {}", line!(), error);
                std::process::exit(3);
            },
        };
    }
    
    let mut paths: Vec<String> = Vec::new();
    
    {
        paths.push("/".to_string());
        for v in &config.file_structure.folders_list {
            let mut paths_to_append: Vec<String> = Vec::new();
            let next_path = String::from("/");
            paths_to_append.push(next_path);
            let mut current_parent_folder = v;
            let mut iterations = 0;
            let max_iterations = 100;
            // MAX ITERATIONS set to 100 (can be changed)
            while current_parent_folder.parent != 0 && iterations < max_iterations {
                // println!("{x}", x = current_parent_folder.name);
                // println!("{x}", x = current_parent_folder.parent);
                match current_parent_folder.name.as_str() {
                    "%days" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.days + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{}_DAY{}", padded_number, padded_number);
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    "%cams" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.cameras + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{x}_{y}_CAM", x = padded_number, y = num_to_char(i).expect("Soft limit for cameras is 26!").to_string());
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    "%soundsources" => {
                        let mut new_paths_vector: Vec<String> = Vec::new();
                        for i in 1..setup.sound_sources + 1 {
                            let padded_number = format!("{:0>2}", i);
                            let folder_name = format!("/{x}_{y}_REC", x = padded_number, y = num_to_char(i).expect("Soft limit for sound recorders is 26!").to_string());
                            for path in &mut paths_to_append {
                                let mut new_path = path.clone();
                                new_path.insert_str(0, &folder_name);
                                new_paths_vector.push(new_path);
                            }
                        }
                        paths_to_append = new_paths_vector;
                    },
                    _ => {
                        for path in &mut paths_to_append {
                            let current_folder_name = &current_parent_folder.name;
                            path.insert_str(0, &format!("/{}", current_folder_name));
                        }
                    },
                };
                current_parent_folder = &config.file_structure.folders_list.get(current_parent_folder.parent - 1).expect("Parent does not exist!");
                iterations += 1;
            }
            if max_iterations == iterations {
                panic!("Looped past max iterations!");
            }
            for path in &mut paths_to_append {
                path.insert_str(0, &setup.name);
            }
            paths.append(&mut paths_to_append);
        }
    }

    for path in paths {
        if !Path::new(&path).exists() {
            match fs::create_dir(&path).map_err(|e| ConfigError::IoError(e)) {
                Ok(()) => {},
                Err(error) => {
                    eprintln!("Line {}: Problem creating file: {}", line!(), error);
                    std::process::exit(3);
                },
            }
            ;
        }
    }

    let write_config_result = Config::write_config(&config, "config.toml");
    match write_config_result {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Line {}: Problem opening the file: {}", line!(), error);
            std::process::exit(1);
        },
    };
    
}

// initializes the main folder, optionally renaming an older folder given the correct conditions.
fn initialize_main_folder(old_setup : &ProjectSetup, setup : &ProjectSetup, op_type: &OperationType, old_config_exists : bool) -> std::result::Result<(), ConfigError>{
    if old_config_exists && op_type == &OperationType::Update && Path::new(&old_setup.name).exists() && &old_setup.name != &setup.name {
        fs::rename(&old_setup.name, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        if !Path::new(&setup.name).exists(){
            fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
        }
    }
    Ok(())
}

// initializes the main folder, renaming an older folder using its name.
fn initialize_main_folder_deadname(deadname: &String, setup: &ProjectSetup) -> std::result::Result<(), ConfigError>{
    if Path::new(&deadname).exists() {
        fs::rename(&deadname, &setup.name).map_err(|e| ConfigError::IoError(e))?;
    } else {
        if !Path::new(&setup.name).exists(){
            fs::create_dir(&setup.name).map_err(|e| ConfigError::IoError(e))?;
        }
    }
    Ok(())
}

fn num_to_char(num: usize) -> Option<char> {
    if num >= 1 && num <= 26 {
        Some((num as u8 + b'A' - 1) as char)
    } else {
        None // Return None if the number is out of range
    }
}
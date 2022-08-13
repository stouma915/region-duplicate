use std::{fs, io};
use std::io::Write;
use std::path::Path;
use std::process::exit;

use clap::{App, Arg};

fn main() {
    let matches = App::new("chunk-duplicate")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"))
        .arg(
            Arg::new("SOURCE")
                .help("Path of Minecraft world data region folder.")
        )
        .get_matches();

    let folder =
        if let Some(p) = matches.value_of("SOURCE") {
            String::from(p)
        } else {
            print!("Path of region folder is: ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();

            if cfg!(windows) {
                input.replace("\r\n", "")
            } else {
                input.replace("\n", "")
            }
        };
    let folder_path = Path::new(&folder);

    if !folder_path.exists() || !folder_path.is_dir() {
        println!("The specified directory '{}' does not exist or isn't a directory.", &folder);
        exit(2);
    }
    if let Some(name) = folder_path.file_name() {
        let name_str = name.to_os_string().into_string().unwrap();
        if name_str != "region" {
            println!("The directory name must be 'region', but it was '{}'.", name_str);
            exit(2);
        }
    } else {
        println!("The specified directory is Root Directory.");
        exit(2);
    }

    let x = request_num("X of source region is: ");
    let y = request_num("Y of source region is: ");

    let src_region_name = format!("r.{}.{}.mca", x, y);
    let src_region_path = folder_path.join(Path::new(&src_region_name));

    if !src_region_path.exists() || !src_region_path.is_file() {
        println!("The region file '{}' does not exist or isn't a file.", &src_region_name);
        exit(2);
    }

    println!();
    println!("WARNING: World data will be destroyed.");

    let mut done_confirm = false;
    let mut confirm_result = false;

    while !done_confirm {
        print!("Continue? (y/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        let replaced = if cfg!(windows) {
            input.replace("\r\n", "")
        } else {
            input.replace("\n", "")
        }.to_lowercase();

        if replaced == "y" || replaced == "n" {
            done_confirm = true;
            confirm_result = replaced == "y";
        }
    }

    if !confirm_result {
        println!("Aborting.");
        exit(1);
    }

    let region_names = fs::read_dir(&folder_path).unwrap().filter_map(|e| {
        let entry = e.ok()?;
        if entry.file_type().ok()?.is_file() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if &file_name != &src_region_name {
                Some(file_name)
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<String>>();

    let mut success = 0;
    let mut failure = 0;

    for region in &region_names {
        let region_path = folder_path.join(Path::new(&region));

        print!("Removing '{}' ...", &region);
        io::stdout().flush().unwrap();
        match fs::remove_file(&region_path) {
            Ok(_) => {
                println!("done");

                print!("Recreating '{}' ...", &region);
                io::stdout().flush().unwrap();
                match fs::copy(&src_region_path, &region_path) {
                    Ok(_) => {
                        println!("done");

                        success += 1;
                    },
                    Err(err) => {
                        println!("failure");
                        println!("Failed to recreate {}: {:?}", &region, err);
                        println!("Skipping.");

                        failure += 1;
                    }
                }
            },
            Err(err) => {
                println!("failure");
                println!("Failed to remove {}: {:?}", &region, err);
                println!("Skipping.");

                failure += 1;
            }
        }
    }

    println!();
    println!("Success {}, Failure {}.", success, failure);
    println!("Complete!");
}

fn request_num(msg: &str) -> i32 {
    let mut done = false;
    let mut tmp = 0;

    while !done {
        print!("{}", msg);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        let replaced = if cfg!(windows) {
            input.replace("\r\n", "")
        } else {
            input.replace("\n", "")
        };

        match replaced.parse::<i32>() {
            Ok(n) => {
                tmp = n;
                done = true;
            },
            Err(_) => println!("Please enter a number.")
        }
    }

    tmp
}

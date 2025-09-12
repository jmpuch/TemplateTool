use clap::{Arg, command};
use ftail::Ftail;
use log::LevelFilter;
//use log::{self, Level};
use std::env;
use std::fs::read_to_string;

fn read_lines(filename: &str) -> Vec<String> {
    read_to_string(filename)
        .expect("Failed to read file") // panic on possible file-reading errors
        .lines() // split the string into an iterator of string slices
        .map(String::from) // make each slice into a string
        .collect() // gather them together into a vector
}

fn main() {
    let arg0 = env::args().next();
    let my_app_name = arg0.as_deref().unwrap_or("template-tool");

    /*
       log::trace!("This is a trace message");
       log::debug!("This is a debug message");
       log::info!(target: "foo", "bar");
       log::warn!("This is a warning message");
       log::error!("This is an error message");

       debug!("Debug log test with macro");
       trace!("Trace log test with macro");
       info!("Info log test with macro");
       warn!("Warn log test with macro");
       error!("Error log test with macro");
    */
    // Define command line arguments using clap
    let matches = command!() // requires `cargo` feature
        .version("1.0")
        .about(
            "traiment de fichiers templates en appliquant des remplacements de balises précisées",
        )
        .arg(
            Arg::new("input_file")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Sets input file"),
        )
        .arg(
            Arg::new("log_level")
                .short('l')
                .long("loglevel")
                .value_name("LEVEL")
                .help("Sets log level (error, warn, info, debug, trace)"),
        )
        .get_matches();

    let input_file = matches
        .get_one::<String>("input_file")
        .map(|s| s.as_str())
        .unwrap_or("work-todo.txt");
    let log_level_str = matches
        .get_one::<String>("log_level")
        .map(|s| s.as_str())
        .unwrap_or("Info");
    let my_level: LevelFilter = match log_level_str.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    let _ = match Ftail::new()
        .console(my_level)
        .single_file(std::path::Path::new("./app.log"), true, my_level)
        .init()
    {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Failed to initialize Ftail: {}", err);
            std::process::exit(1);
        }
    };

    log::info!("Warning: This is a pre-release version.");
    log::info!("Log level set to {:?}", my_level);
    log::info!("Start of processing... {}", my_app_name);
    log::info!("Reading input_file {} ...", input_file);
    let lines = read_lines(input_file);
    let mut line_iter = lines.iter();

    // Process all lines of the file
    let mut if_found = false;
    let mut of_found = false;
    let mut r_found = false;
    let mut if_file_name = "";
    let mut of_file_name = "";
    let mut keypairs: Vec<(&str, &str)> = Vec::new();

    loop {
        match line_iter.next() {
            Some(line) => {
                // skip empty lines or lines starting with #
                if line.is_empty() || line.starts_with('#') {
                    continue;
                } else {
                    let parts: Vec<&str> = line.split(';').collect();
                    if Vec::len(&parts) > 1 {
                        match parts[0] {
                            "if" => {
                                log::trace!("   if line found: {}", line);
                                if_file_name = parts[1];
                                if_found = true;
                            }
                            "of" => {
                                log::trace!("   of line found: {}", line);
                                of_file_name = parts[1];
                                of_found = true;
                            }
                            "r" => {
                                log::trace!("   r line found: {}", line);
                                keypairs.push((parts[1], parts[2]));
                                r_found = true;
                            }
                            "do" => {
                                log::trace!("   do line found: {}", line);
                                if if_found == true && of_found == true && r_found == true {
                                    log::trace!(
                                        "   All required lines found. Proceeding with processing."
                                    );
                                    log::trace!(
                                        "      Processing with if: {}, of: {}, r: {}",
                                        if_file_name,
                                        of_file_name,
                                        r_found
                                    );

                                    let template_content = read_to_string(&if_file_name)
                                        .expect("Failed to read template file");

                                    let mut iterator = keypairs.iter();
                                    let mut output_content = template_content;
                                    while let Some(element) = iterator.next() {
                                        log::trace!(
                                            "      replacing {} / {}",
                                            element.0,
                                            element.1
                                        );
                                        output_content =
                                            output_content.replace(element.0, element.1);
                                    }

                                    std::fs::write(&of_file_name, output_content)
                                        .expect("Failed to write output file");
                                    log::trace!(
                                        "      File {} created successfully.",
                                        of_file_name
                                    );
                                    log::trace!("   done\n");
                                    keypairs.clear();
                                } else {
                                    log::error!("   Error: Missing required lines before 'do'.");
                                }
                            }
                            _ => {
                                log::error!("Unknown line type: {}", line);
                            }
                        }
                    }
                }
            }
            None => break,
        }
    }

    log::info!("End of processing.");
}

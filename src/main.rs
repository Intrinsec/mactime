use std::{error::Error, path::Path};
use chrono::{NaiveDate};
use clap::{Command, Arg};

mod bodyfile;
use bodyfile::{BodyFileParser, DateFilter};

const FORMAT : &str = "Date filter format: YYYY-MM-DD..YYYY-MM-DD (time not handled yet)";

fn parse_filter_args(args: &str) -> Result<[NaiveDate;2], String> {
    fn validate_date(date: &str) -> Result<NaiveDate, String> {
        NaiveDate::parse_from_str(date, "%F")
            // .map(|_| ()) // ignore NaiveDate
            .map_err(|_| String::from("Dates must be in the YYYY-MM-DD format")) // Year-month-day format (ISO 8601). Same as %Y-%m-%d
    }

    let dates : Vec<&str> = args.split("..").collect();
    if dates.len() != 2 {
        return Err(FORMAT.into())
    }

    let start = validate_date(dates[0])?; // start
    let end = validate_date(dates[1])?; // end

    Ok([start, end])
}

fn validate_filter_args(args: &str) -> Result<(), String> {
    parse_filter_args(args).map(|_| ()) // clap doesn't want a value!
}

fn main() -> Result<(), Box<dyn Error>> {
    /*
    Inspired from https://github.com/sleuthkit/sleuthkit/blob/master/tools/timeline/mactime.base
    mactime [-b body_file] [-p password_file] [-g group_file] [-i day|hour idx_file] [-d] [-h] [-V] [-y] [-z TIME_ZONE] [DATE]
		-b: Specifies the body file location, else STDIN is used
		-d: Output in comma delimited format
		-h: Display a header with session information
		-i [day | hour] file: Specifies the index file with a summary of results
		-y: Dates are displayed in ISO 8601 format
		-m: Dates have month as number instead of word (does not work with -y)
		-z: Specify the timezone the data came from (in the local system format) (does not work with -y)
		-g: Specifies the group file location, else GIDs are used
		-p: Specifies the password file location, else UIDs are used
		-V: Prints the version to STDOUT
		[DATE]: starting date (yyyy-mm-dd) or range (yyyy-mm-dd..yyyy-mm-dd) 
		[DATE]: date with time (yyyy-mm-ddThh:mm:ss), using with range one or both can have time
    */

    /*
    Handle args, rules are:
    - dates are UTC
    - output in CSV
    - No date filters required by default
    */
    let matches = Command::new("mactime")
        .author("CERT Intrinsec")
        .arg(Arg::new("bodyfile")
            .short('b')
            .long("bodyfile")
            .required(true)
            .takes_value(true))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .required(false)
            .help("CSV output to file (stdout if not specified)")
            .takes_value(true))
        .arg(Arg::new("filter")
            .short('f')
            .long("filter")
            .required(false)
            .takes_value(true)
            .help(FORMAT)
            .validator(validate_filter_args))
        .arg(Arg::new("sort")
            .short('s')
            .long("sort")
            .required(false)
            .help("Sort timeline by datetime")
            .takes_value(false))
        /*.arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .required(false)
            .help("Display verbose information")
            .takes_value(false)*/
        .get_matches();

    let input = matches.value_of("bodyfile").expect("required bodyfile");
    let output = matches.value_of("output").map(Path::new); // map to path if present, None otherwise
    let filter = matches.value_of("filter")
        .map(|d| parse_filter_args(d).unwrap() ) // parse dates (we can unwrap because it has been validated by clap)
        .map(|d| DateFilter::new(d) ); // convert to DateFilter

    // build bodyfile object: parse bodyfile entries & build timeline with datetime entries
    let bodyfile = BodyFileParser::build(Path::new(input), filter, matches.is_present("sort"))?;

    eprintln!("Number of file records read from {input}: {}", bodyfile.file_len());
    eprintln!("Number of datetime records read from {input}: {}", bodyfile.datetime_len());

    // write CSV to output (stdout or file)
    bodyfile.generate_csv(output)?;

    Ok(())
}
use std::{path::Path, error::Error, collections::HashMap, fmt};
use chrono::{DateTime, Utc, NaiveDate};
use csv::{StringRecord};
use serde::Deserialize;
use bitflags::bitflags;

pub struct BodyFileParser;

impl BodyFileParser {
    pub fn build(path: &Path, filter: Option<DateFilter>, sorted: bool) -> Result<BodyFile, Box<dyn Error>> {
        let mut bodyfile = BodyFile::new();

        // open file, read line, parse line, add entry, build timeline, sort
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)             // we create them just after
            .delimiter(b'|')
            .from_path(path)?;

        // MD5|name|inode|mode_as_string|UID|GID|size|atime|mtime|ctime|crtime
        // 0|c:/$MFT|0-128-6|r/rrwxrwxrwx|0|0|1835008|1595291898|1595291898|1595291898|1595291898
        let headers = StringRecord::from(vec!["md5", "name", "inode", "mode_as_string", "uid", "gid", "size", "atime", "mtime", "ctime", "crtime"]);
        reader.set_headers(headers);

        for record in reader.deserialize() {
            if let Err(e) = record {
                println!("Error deserializing record => {e}");
                // println!("Error deserializing record:\n\t- Error: {e}\n\t- Raw record: {}", );
                continue;
            }
            let record : BodyFileEntry = record.unwrap();
            // println!("{record:#?}");
            bodyfile.add_entry(record);
        }

        bodyfile.build_timeline(&filter);

        if sorted {
            bodyfile.sort_timeline();
        }

        Ok(bodyfile)
    }
}

bitflags! {
    struct MACB : u8 {
        const MODIFIED = 0x1;
        const ACCESSED = 0x2;
        const CHANGED  = 0x4;
        const BIRTH    = 0x8;
    }
}

impl fmt::Display for MACB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut macb_str = String::new();
        macb_str.push(if self.contains(MACB::MODIFIED) { 'm' } else { '.' });
        macb_str.push(if self.contains(MACB::ACCESSED) { 'a' } else { '.' });
        macb_str.push(if self.contains(MACB::CHANGED) { 'c' } else { '.' });
        macb_str.push(if self.contains(MACB::BIRTH) { 'b' } else { '.' });
        write!(f, "{macb_str}")
    }
}

#[derive(Debug)]
struct TimestampEntry {
    datetime: DateTime<Utc>,
    macb: MACB,
    meta: String,
    size: u64,
    filename: String
}

impl Ord for TimestampEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl Eq for TimestampEntry {}

impl PartialEq for TimestampEntry {
    fn eq(&self, other: &Self) -> bool {
        self.datetime == other.datetime && self.macb == other.macb && self.meta == other.meta && self.size == other.size && self.filename == other.filename
    }
}

impl PartialOrd for TimestampEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // first compare datetime
        match self.datetime.partial_cmp(&other.datetime) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        // then by filename
        self.filename.partial_cmp(&other.filename)

        /*
        match self.macb.partial_cmp(&other.macb) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.meta.partial_cmp(&other.meta) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.size.partial_cmp(&other.size) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.filename.partial_cmp(&other.filename)
        */
    }
}

pub struct DateFilter {
    start: NaiveDate,
    end: NaiveDate
}

impl DateFilter {
    pub fn new(d: [NaiveDate;2]) -> Self {
        Self {
            start: d[0],
            end: d[1]
        }
    }
}
// pub struct DateRange(NaiveDate, NaiveDate)

#[derive(Debug)]
pub struct BodyFile {
    entries: Vec<BodyFileEntry>,
    timeline: Vec<TimestampEntry>
}

impl BodyFile {
    fn new() -> Self {
        Self {
            entries: vec![],
            timeline: vec![]
        }
    }

    pub fn file_len(&self) -> usize {
        self.entries.len()
    }

    pub fn datetime_len(&self) -> usize {
        self.timeline.len()
    }

    fn add_entry(&mut self, entry: BodyFileEntry) {
        self.entries.push(entry)
    }

    fn sort_timeline(&mut self) {
        self.timeline.sort()
    }

    fn build_timeline(&mut self, filter: &Option<DateFilter>) {
        for entry in self.entries.iter() {
            // for 1 entry, we can have 4 different CSV entries, one for each MACB timestamps
            
            // convert MACB into a HashMap : <timestamp> => <macb_string>
            let mut macb : HashMap<DateTime<Utc>, MACB> = HashMap::new();
            
            let current_macb = macb.entry(entry.mtime).or_insert(MACB::MODIFIED);
            *current_macb |= MACB::MODIFIED;

            let current_macb = macb.entry(entry.atime).or_insert(MACB::ACCESSED);
            *current_macb |= MACB::ACCESSED;

            let current_macb = macb.entry(entry.ctime).or_insert(MACB::CHANGED);
            *current_macb |= MACB::CHANGED;

            let current_macb = macb.entry(entry.crtime).or_insert(MACB::BIRTH);
            *current_macb |= MACB::BIRTH;

            // for each entry, generate a record & push it to the timeline
            for (date, macb) in macb {

                let out_of_range = match filter.as_ref() {
                    Some(date_filter) => {
                        let naive = date.date().naive_utc();
                        !(date_filter.start <= naive && naive <= date_filter.end) // filter out entries not in the date range
                    }
                    None => false // if date filter is unspecified => all dates are in range
                };

                if out_of_range {
                    continue;
                }

                let timestamp_entry = TimestampEntry { // lots of copies here ...
                    datetime: date,
                    macb: macb,
                    meta: entry.meta.clone(),
                    size: entry.size,
                    filename: entry.name.clone()
                };

                self.timeline.push(timestamp_entry);
            }
        }
    }

    /*enum Destination<'a> {
        File(&'a Path),
        StdOut
    }*/

    pub fn generate_csv(&self, output: Option<&Path>) -> Result<(), Box<dyn Error>> {
        // generate CSV from entries

        // build the writer according to `output` => see https://github.com/BurntSushi/rust-csv/issues/196
        let source_writer : Box<dyn std::io::Write> = match output {
            Some(p) => {
                println!("Writing CSV to {}", p.display());
                Box::new(std::fs::File::create(p)?)
            },
            None => Box::new(std::io::stdout()) // write to stdout
        };

        let mut _count = 0;
        let mut writer = csv::Writer::from_writer(source_writer);
        writer.write_record(&["Datetime", "MACB", "Meta", "Size", "FileName"])?; // headers

        for entry in self.timeline.iter() {
            // TODO: serialize TimeStampEntry directly !
            let date_str = format!("{}", entry.datetime.format("%Y-%m-%d %H:%M:%S"));
            let macb_str = format!("{}", entry.macb);
            let size_str = format!("{}", entry.size);
            let result = writer.write_record(&[
                date_str.as_str(),
                macb_str.as_str(),
                entry.meta.as_str(),
                size_str.as_str(),
                entry.filename.as_str()
            ]);

            if let Err(e) = result {
                eprintln!("Error writing CSV result: {e}");
                continue;
            }

            _count += 1;
        }

        //println!("Writing {_count} timestamp records to CSV");
        writer.flush()?;

        Ok(())
    }
}

/* bodyfile format : https://wiki.sleuthkit.org/index.php?title=Body_file */
#[derive(Debug, Deserialize)]
pub struct BodyFileEntry {
    name: String, // c:/$MFT
    #[serde(rename = "inode")]
    meta: String, // 0-128-6
    size: u64, // 1835008
    #[serde(with = "unix_date_format")]
    atime: DateTime<Utc>, // access
    #[serde(with = "unix_date_format")]
    mtime: DateTime<Utc>, // modified
    #[serde(with = "unix_date_format")]
    ctime: DateTime<Utc>, // metadata change
    #[serde(with = "unix_date_format")]
    crtime: DateTime<Utc>, // creation
}

mod unix_date_format {
    use chrono::{DateTime, Utc, NaiveDateTime};
    use serde::{self, Deserialize, Deserializer};

    // ` Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)` does not work on negative numbers => so we parse the value to i64 and then use `from_timestamp`
    // const FORMAT: &'static str = "%s";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
        let timestamp: i64 = s.parse().map_err(serde::de::Error::custom)?;
        let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
        Ok(datetime)

        /*
        let result = Utc.datetime_from_str(&s, FORMAT);
        if let Err(e) = result {
            // println!("Error while deserializing timestamp: {e}");
            return Err(serde::de::Error::custom(e));
        }
        return result.map_err(serde::de::Error::custom);
        */
    }
}

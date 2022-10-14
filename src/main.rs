use serde::{de, Deserialize, Deserializer};
use std::fmt::Debug;
use std::fs::{self};
use std::io::Write;
use std::path::PathBuf;
use std::{io::LineWriter, path::Path};
use time::{macros::format_description, Date};
use time::{Duration, OffsetDateTime};

fn main() {
    let csv_paths = csv_paths(Path::new("input"));

    for csv_path in &csv_paths {
        eprintln!("Processing {:?}", csv_path);
    }

    let parsed_csvs = csv_paths
        .iter()
        .map(|path| parse_csv(path).unwrap())
        .collect::<Vec<_>>();

    let merged_days = merge(&parsed_csvs);

    eprintln!();
    eprintln!("List of days:");
    for feiertag in &merged_days {
        eprintln!("{}: {}", feiertag.date, feiertag.name);
    }
    eprintln!();

    write_ical(&merged_days, LineWriter::new(std::io::stdout())).unwrap();
}

fn csv_paths(directory: &Path) -> Vec<PathBuf> {
    let paths = fs::read_dir(directory).unwrap();
    paths
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|path| {
            path.extension()
                .map_or(false, |extension| extension == "csv")
        })
        .collect()
}

fn write_ical<W: Write>(
    merged_days: &[Feiertag],
    mut writer: LineWriter<W>,
) -> Result<(), anyhow::Error> {
    // write header

    writeln!(writer, "BEGIN:VCALENDAR")?;
    writeln!(
        writer,
        "PRODID:-//Geo Engine GmbH//Geo Engine Feiertage//DE"
    )?;
    writeln!(writer, "VERSION:2.0")?;
    writeln!(writer, "CALSCALE:GREGORIAN")?;
    writeln!(writer, "METHOD:PUBLISH")?;
    writeln!(
        writer,
        "X-WR-CALNAME:Feiertage in Deutschland für die Geo Engine GmbH"
    )?;
    writeln!(writer, "X-WR-TIMEZONE:UTC")?;
    writeln!(
        writer,
        "X-WR-CALDESC:Feiertage in Deutschland für die Geo Engine GmbH"
    )?;

    // write events

    let date_format = format_description!("[year][month][day]");
    let timestamp_format = format_description!("[year][month][day]T[hour][minute][second]Z");

    let today = OffsetDateTime::now_utc();

    for feiertag in merged_days {
        let date = feiertag.date.format(date_format)?;
        let next_date = (feiertag.date + Duration::DAY).format(date_format)?;

        writeln!(writer, "BEGIN:VEVENT")?;
        writeln!(writer, "DTSTART;VALUE=DATE:{date}")?;
        writeln!(writer, "DTEND;VALUE=DATE:{next_date}")?;
        writeln!(writer, "DTSTAMP:{}", today.format(timestamp_format)?)?;
        writeln!(writer, "UID:{date}#feiertage")?;
        writeln!(writer, "CLASS:PUBLIC")?;
        writeln!(writer, "CREATED:{}", today.format(timestamp_format)?)?;
        writeln!(writer, "DESCRIPTION:Gesetzlicher Feiertag")?;
        writeln!(writer, "LAST-MODIFIED:{}", today.format(timestamp_format)?)?;
        writeln!(writer, "SEQUENCE:0")?;
        writeln!(writer, "STATUS:CONFIRMED")?;
        writeln!(writer, "SUMMARY:{}", feiertag.name)?;
        writeln!(writer, "TRANSP:TRANSPARENT")?;
        writeln!(writer, "END:VEVENT")?;
    }

    // end calendar
    writeln!(writer, "END:VCALENDAR")?;

    Ok(())
}

/// Merges multiple inputs into a single one.
/// Assumes the inputs are sorted by date.
fn merge(inputs: &[Vec<Feiertag>]) -> Vec<Feiertag> {
    let mut iterators = inputs
        .iter()
        .map(|input| input.iter().peekable())
        .collect::<Vec<_>>();

    let mut result: Vec<Feiertag> = Vec::new();

    // get nearest date while at least one iterator outputs one
    while let Some(next_feiertag) = iterators
        .iter_mut()
        .filter_map(|it| it.peek())
        .min_by_key(|feiertag| feiertag.date)
        .cloned()
    {
        // advance all iterators with that date
        for it in &mut iterators {
            if let Some(feiertag) = it.peek() {
                if feiertag.date == next_feiertag.date {
                    it.next();
                }
            }
        }

        // put into result
        result.push(next_feiertag.clone());
    }

    result
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Clone)]
struct Feiertag {
    name: String,
    #[serde(deserialize_with = "deserialize_date")]
    date: Date,
}

fn parse_csv(path: &Path) -> Result<Vec<Feiertag>, csv::Error> {
    let mut reader = csv::ReaderBuilder::new().delimiter(b'\t').from_path(path)?;

    reader.deserialize().collect()
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    // ignore three chars, e.g. `Mo, `
    let substring = &s[4..];

    let format = format_description!("[day].[month].[year]");
    Date::parse(substring, &format).map_err(de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    use time::Month;

    // #[test]
    // fn test_deserialize_feiertag() {
    //     assert!(parse_csv(Path::new("feiertage-nrw.csv")));
    // }

    #[test]
    fn test_merge() {
        let v1 = vec![
            Feiertag {
                name: "Neujahr".to_string(),
                date: Date::from_calendar_date(2020, Month::January, 1).unwrap(),
            },
            Feiertag {
                name: "Glückstag".to_string(),
                date: Date::from_calendar_date(2020, Month::February, 3).unwrap(),
            },
            Feiertag {
                name: "1. Mai!!!".to_string(),
                date: Date::from_calendar_date(2020, Month::May, 1).unwrap(),
            },
        ];
        let v2 = vec![
            Feiertag {
                name: "Neujahr".to_string(),
                date: Date::from_calendar_date(2020, Month::January, 1).unwrap(),
            },
            Feiertag {
                name: "Karneval".to_string(),
                date: Date::from_calendar_date(2020, Month::February, 10).unwrap(),
            },
            Feiertag {
                name: "1. Mai!!!".to_string(),
                date: Date::from_calendar_date(2020, Month::May, 1).unwrap(),
            },
        ];

        let merged = merge(&[v1, v2]);

        assert_eq!(
            merged,
            vec![
                Feiertag {
                    name: "Neujahr".to_string(),
                    date: Date::from_calendar_date(2020, Month::January, 1).unwrap(),
                },
                Feiertag {
                    name: "Glückstag".to_string(),
                    date: Date::from_calendar_date(2020, Month::February, 3).unwrap(),
                },
                Feiertag {
                    name: "Karneval".to_string(),
                    date: Date::from_calendar_date(2020, Month::February, 10).unwrap(),
                },
                Feiertag {
                    name: "1. Mai!!!".to_string(),
                    date: Date::from_calendar_date(2020, Month::May, 1).unwrap(),
                },
            ]
        );
    }
}

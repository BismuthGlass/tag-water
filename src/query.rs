use crate::database::{models, Database};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rocket::State;

#[derive(Debug)]
pub struct MetaTag {
    name: String,
    value: Option<String>,
}

#[derive(Debug)]
pub struct QueryData {
    pub tags_included: Vec<String>,
    pub tags_excluded: Vec<String>,
    pub metatags: Vec<MetaTag>,
}

fn date_string_to_timestamp(date_str: &str) -> Result<i64, &'static str> {
    // Split the date and time parts
    let parts: Vec<&str> = date_str.split("_").collect();
    if parts.len() != 2 {
        return Err("Invalid date format");
    }

    let date_part = parts[0];
    let time_part = parts[1];

    // Split date into day, month, year
    let date_parts: Vec<&str> = date_part.split("-").collect();
    if date_parts.len() != 3 {
        return Err("Invalid date format");
    }
    let day: u32 = date_parts[0].parse().map_err(|_| "Failed to parse day")?;
    let month: u32 = date_parts[1].parse().map_err(|_| "Failed to parse month")?;
    let year: i32 = date_parts[2].parse().map_err(|_| "Failed to parse year")?;

    // Split time into hour, minute
    let time_parts: Vec<&str> = time_part.split(":").collect();
    if time_parts.len() != 2 {
        return Err("Invalid time format");
    }
    let hour: u32 = time_parts[0].parse().map_err(|_| "Failed to parse hour")?;
    let minute: u32 = time_parts[1]
        .parse()
        .map_err(|_| "Failed to parse minute")?;

    // Create NaiveDateTime object
    let datetime = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year, month, day).ok_or("Invalid date")?,
        NaiveTime::from_hms_opt(hour, minute, 0).ok_or("Invalid time")?,
    );

    // Convert to Unix timestamp
    Ok(datetime.and_utc().timestamp())
}

pub fn parse_query(query: &[String]) -> QueryData {
    let mut tags_included = Vec::new();
    let mut tags_excluded = Vec::new();
    let mut metatags = Vec::new();
    for part in query {
        if part.len() == 0 {
            continue;
        }
        match part.as_bytes()[0] {
            b'-' => {
                tags_excluded.push(part[1..].to_string());
            }
            b'@' => {
                let base: Vec<&str> = part[1..].split('=').collect();
                if base.len() < 2 {
                    metatags.push(MetaTag {
                        name: base[0].to_string(),
                        value: None,
                    });
                } else {
                    metatags.push(MetaTag {
                        name: base[0].to_string(),
                        value: Some(base[1].to_string()),
                    });
                }
            }
            _ => {
                tags_included.push(part.to_string());
            }
        }
    }
    QueryData {
        tags_included,
        tags_excluded,
        metatags,
    }
}

pub async fn parse_query_string(
    db: &State<Database>,
    query: &[String],
) -> Result<models::EntryQuery, Vec<String>> {
    let query_breakdown = parse_query(query);
    let mut query_data = models::EntryQuery::default();
    let mut log = Vec::new();

    // Checking for every tag
    let mut unknown_tags = Vec::new();
    for t in &query_breakdown.tags_included {
        match db.get_tag(t.clone()).await {
            None => unknown_tags.push(t.clone()),
            Some(id) => query_data.tags_included.push(id),
        }
    }
    for t in &query_breakdown.tags_excluded {
        match db.get_tag(t.clone()).await {
            None => unknown_tags.push(t.clone()),
            Some(id) => query_data.tags_excluded.push(id),
        }
    }
    if unknown_tags.len() > 0 {
        log.push(format!("Tag not found: {}", unknown_tags.join(" "),));
        return Err(log);
    }

    // Handling metatags
    for mt in &query_breakdown.metatags {
        match mt.name.as_str() {
            "created_after" => match &mt.value {
                None => log.push(format!("@{} needs a value (`dd-mm-yyyy_hh:mm`)", mt.name)),
                Some(v) => match date_string_to_timestamp(v) {
                    Ok(time) => query_data.created_after = Some(time),
                    Err(e) => log.push(format!("@{}: {e}", mt.name)),
                },
            },
            "created_before" => match &mt.value {
                None => log.push(format!("@{} needs a value (`dd-mm-yyyy_hh:mm`)", mt.name)),
                Some(v) => match date_string_to_timestamp(v) {
                    Ok(time) => query_data.created_before = Some(time),
                    Err(e) => log.push(format!("@{}: {e}", mt.name)),
                },
            },
            "updated_after" => match &mt.value {
                None => log.push(format!("@{} needs a value (`dd-mm-yyyy_hh:mm`)", mt.name)),
                Some(v) => match date_string_to_timestamp(v) {
                    Ok(time) => query_data.updated_after = Some(time),
                    Err(e) => log.push(format!("@{}: {e}", mt.name)),
                },
            },
            "updated_before" => match &mt.value {
                None => log.push(format!("@{} needs a value (`dd-mm-yyyy_hh:mm`)", mt.name)),
                Some(v) => match date_string_to_timestamp(v) {
                    Ok(time) => query_data.updated_before = Some(time),
                    Err(e) => log.push(format!("@{}: {e}", mt.name)),
                },
            },
            "is_set" => {
                query_data.is_set = true;
            }
            "is_file" => {
                query_data.is_file = true;
            }
            "untagged" => {
                query_data.untagged = true;
            }
            "include_set_files" => {
                query_data.include_set_files = true;
            }
            _ => log.push(format!("Unknown meta tag {}", mt.name)),
        }
    }
    if log.len() > 0 {
        return Err(log);
    }

    Ok(query_data)
}

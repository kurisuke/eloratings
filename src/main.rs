mod constants;
mod db;
mod error;
mod model;

use std::collections::HashMap;

use chrono::NaiveDate;
use futures::future::try_join_all;

use constants::*;
use db::Db;
use error::AppError;
use model::{MatchHistory, Team};

fn is_team_id(s: &str) -> bool {
    s.len() == 2 && s.chars().all(char::is_uppercase)
}

fn format_page_name(s: &str) -> String {
    s.replace(" ", "_")
        .replace(&['à', 'á', 'â', 'ã', 'ä', 'å'][..], "a")
        .replace("ç", "c")
        .replace(&['è', 'é', 'ê', 'ë'][..], "e")
        .replace(&['ì', 'í', 'î', 'ï'][..], "i")
        .replace(&['ò', 'ó', 'ô', 'õ', 'ö'][..], "o")
        .replace(&['ù', 'ú', 'û', 'ü'][..], "u")
        .replace("ñ", "n")
}

async fn get_teams() -> Result<HashMap<String, Team>, Box<dyn std::error::Error>> {
    let mut teams = HashMap::new();

    let req_t = format!("{}{}", BASE_URI, ENDPOINT_TEAMS);
    let req_h = format!("{}{}", BASE_URI, ENDPOINT_TEAMS_HIST);

    let resp_t = reqwest::get(&req_t);
    let resp_h = reqwest::get(&req_h);

    let text_t = resp_t.await?.text().await?;
    for line_t in text_t.lines() {
        let spl: Vec<_> = line_t.split('\t').collect();
        if spl.len() >= 2 && is_team_id(spl[0]) {
            teams.insert(
                spl[0].to_string(),
                Team {
                    team_id: spl[0].to_string(),
                    team_name: spl[1].to_string(),
                    successor_id: None,
                },
            );
        }
    }

    let text_h = resp_h.await?.text().await?;
    for line_h in text_h.lines() {
        let spl: Vec<_> = line_h.split('\t').collect();
        if spl.len() >= 2 && is_team_id(spl[0]) && is_team_id(spl[1]) {
            let team = teams.get_mut(spl[0]).unwrap();
            team.successor_id = Some(spl[1].to_string());
        }
    }

    Ok(teams)
}

fn get_hist_teams(id: &str, teams: &HashMap<String, Team>) -> Vec<String> {
    teams
        .values()
        .filter(|x| x.successor_id.as_ref().unwrap_or(&String::new()) == id || x.team_id == id)
        .map(|x| x.team_id.clone())
        .collect()
}

async fn get_match_history(
    team_id: &str,
    team_path: &str,
    hist_ids: &[String],
) -> Result<Vec<MatchHistory>, Box<dyn std::error::Error>> {
    let mut mh = vec![];
    let req_mh = format!("{}{}.tsv", BASE_URI, team_path);
    let text_mh = reqwest::get(&req_mh).await?.text().await?;

    for line_h in text_mh.lines() {
        let spl: Vec<_> = line_h.split('\t').collect();
        if spl.len() >= 12 {
            let year = spl[0].parse::<i32>()?;
            let month = spl[1].parse::<u32>()?;
            let day = spl[2].parse::<u32>()?;

            let month = month.max(1);
            let day = day.max(1);

            let date = NaiveDate::from_ymd(year, month, day);

            let team_home = spl[3];
            let team_away = spl[4];
            let goals_home = spl[5].parse::<u32>()?;
            let goals_away = spl[6].parse::<u32>()?;

            let tournament = spl[7];
            let location = if spl[8].is_empty() {
                None
            } else {
                Some(spl[8].to_string())
            };

            let elo = if hist_ids.iter().find(|x| **x == team_home).is_some() {
                spl[10].parse::<u32>()?
            } else if hist_ids.iter().find(|x| **x == team_away).is_some() {
                spl[11].parse::<u32>()?
            } else {
                return Err(Box::new(AppError::new(&format!(
                    "cannot identify home/away: team {}, home {}, away {}",
                    team_id, team_home, team_away
                ))));
            };
            mh.push(MatchHistory {
                team_id: team_id.to_string(),
                date,
                team_home: team_home.to_string(),
                team_away: team_away.to_string(),
                goals_home,
                goals_away,
                tournament: tournament.to_string(),
                location,
                elo,
            });
        } else {
            return Err(Box::new(AppError::new(&format!(
                "invalid match history line for team {}: {}",
                team_id, line_h
            ))));
        }
    }
    Ok(mh)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Db::init().await?;

    let teams = get_teams().await?;
    let futs_team: Vec<_> = teams
        .values()
        .map(|team| {
            let page_name = format_page_name(&team.team_name);
            assert!(page_name.chars().all(|c| c.is_ascii()));
            db.insert_team(&team)
        })
        .collect();
    try_join_all(futs_team).await?;

    for team in teams.values() {
        if team.successor_id.is_none() {
            print!("Downloading: {} ... ", team.team_name);
            let page_name = format_page_name(&team.team_name);
            let hist_teams = get_hist_teams(&team.team_id, &teams);
            let mhs = get_match_history(&team.team_id, &page_name, &hist_teams).await?;
            print!("Inserting {} matches ... ", mhs.len());
            let futs_mh: Vec<_> = mhs.iter().map(|mh| db.insert_match_history(mh)).collect();
            try_join_all(futs_mh).await?;
            println!("done");
        }
    }

    Ok(())
}

use chrono::NaiveDate;

#[derive(Debug)]
pub struct Team {
    pub team_id: String,
    pub team_name: String,
    pub successor_id: Option<String>,
}

#[derive(Debug)]
pub struct MatchHistory {
    pub team_id: String,
    pub date: NaiveDate,
    pub team_home: String,
    pub team_away: String,
    pub goals_home: u32,
    pub goals_away: u32,
    pub tournament: String,
    pub location: Option<String>,
    pub elo: u32,
}

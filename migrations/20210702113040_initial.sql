-- Initial setup

CREATE TABLE teams (
       team_id TEXT PRIMARY KEY,
       team_name TEXT NOT NULL,
       successor_id TEXT
);

CREATE TABLE match_history (
       team_id TEXT NOT NULL,
       date TEXT NOT NULL,
       team_home TEXT NOT NULL,
       team_away TEXT NOT NULL,
       goals_home INTEGER NOT NULL,
       goals_away INTEGER NOT NULL,
       tournament TEXT NOT NULL,
       location TEXT,
       elo INTEGER NOT NULL,
       PRIMARY KEY (team_id, date, team_home, team_away)
);

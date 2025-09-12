use temporal_rs::partial::PartialDate;

pub type WorkId = u64;

pub type Language = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum CreatorRole {
    Composer,
    Lyricist,
    Arranger,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreatorCredit {
    pub name: String,
    pub roles: Vec<CreatorRole>,
    pub artist_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkKey(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct Work {
    pub id: WorkId,
    pub title: String,
    pub aliases: Vec<String>,
    pub credits: Vec<CreatorCredit>,
    pub iswc: Option<String>,
    pub mbid: Option<String>,
    pub language: Vec<Language>,
    pub created: Option<PartialDate>,
    pub candidate_key: WorkKey,
}

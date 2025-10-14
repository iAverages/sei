use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum Animes {
    Table,
    Id,
    EnglishTitle,
    RomajiTitle,
    Status,
    Picture,
    CreatedAt,
    UpdatedAt,
    Season,
    SeasonYear,
}

#[derive(Iden)]
pub enum AnimeRelations {
    Table,
    AnimeId,
    RelationId,
    Relation,
}

#[derive(Iden)]
pub enum AnimeUsers {
    Table,
    UserId,
    AnimeId,
    Status,
    WatchPriority,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Sessions {
    Table,
    Id,
    UserId,
    ExpiresAt,
    CreatedAt,
}

pub enum Status {
    Table,
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
    OnHold,
}

pub enum AiringStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
}

pub enum Relation {
    Adaptation,
    Prequel,
    Sequel,
    Parent,
    SideStory,
    Character,
    Summary,
    Alternative,
    SpinOff,
    Other,
    Source,
    Compilation,
    Contains,
}

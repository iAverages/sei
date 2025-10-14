use sea_query::Iden;

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Name,
    MalId,
    MalAccessToken,
    MalRefreshToken,
    Picture,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    ListLastUpdate,
}

pub struct DBUser {
    pub id: String,
}

use std::fmt::{Display, Formatter};

use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct GqlQuery {
    pub query: String,
    pub variables: Value,
}

impl Display for GqlQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query: {}\nvariables: {}", self.query, self.variables)
    }
}
impl GqlQuery {
    pub fn generate_gql_query(ids: &[i32]) -> GqlQuery {
        if ids.len() > MAX_ANILIST_PER_QUERY {
            tracing::warn!("too many ids for query: {}", ids.len());
        }

        let mut query = "query media(".to_owned();
        let mut variables = json!({});

        for (index, id) in ids.iter().enumerate() {
            if index >= MAX_ANILIST_PER_QUERY {
                break;
            }
            let anime_index = index + 1;
            query.push_str(&format!("$anime{}: Int,", anime_index));
            let variable_name = "anime".to_owned() + &anime_index.to_string();
            variables[variable_name] = json!(id);
        }

        query.push_str(") {");

        let media_selection = String::from(ANILIST_MEDIA_SELECTION);
        for i in 1..ids.len() + 1 {
            let media_selection = media_selection.replace("{}", &i.to_string());
            query.push_str(&media_selection);
        }

        query.push('}');

        GqlQuery { query, variables }
    }
}

/// max number of anime data we can fetch per request
/// the limit here is based on the query, if the query
/// changes, this number will change. anilists max query
/// complexity is 500.
pub const MAX_ANILIST_PER_QUERY: usize = 35;
const ANILIST_MEDIA_SELECTION: &str = r#"
anime{}: Media(idMal: $anime{}, type: ANIME) {
    status
    idMal
    title {
      romaji
    }
    season
    seasonYear
    coverImage {
      large
    }
    relations {
      edges {
        relationType(version: 2)
        node {
          idMal
        }
      }
    }
  }
"#;

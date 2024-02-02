macro_rules! json_response {
    ($status:expr , $json:tt) => {
        ($status, Json(json!($json))).into_response()
    };
}

pub(crate) use json_response;

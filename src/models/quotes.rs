use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KanyeRestResponse {
    pub quote: String,
}

#[derive(Debug, Deserialize)]
pub struct TronaldDumpReponse {
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ChuckNorrisIoResponse {
    pub value: String,
}

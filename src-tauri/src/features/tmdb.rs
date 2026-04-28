use reqwest;
use serde::{Deserialize, Serialize};

const TMDB_API_KEY: &str = "4736e6f3761df4fb134856cc298f2d5e";
const BASE_URL: &str = "https://api.themoviedb.org/3";

#[tauri::command]
pub async fn get_discover_movie(params: String)->Result<Value, String> {
    let url = format!(
        "{}/{}?api_key={}&language=tr-TR{}",
        BASE_URL, endpoint, TMDB_API_KEY, params
    );

    let response = reqwest::get(url)
        .await()
        .map_err(|e| e.to_string())?;

    let data: Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(data);
}

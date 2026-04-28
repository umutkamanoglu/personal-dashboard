use reqwest;
use serde_json::Value; // Value için bu şart

const TMDB_API_KEY: &str = "4736e6f3761df4fb134856cc298f2d5e";
const BASE_URL: &str = "https://api.themoviedb.org/3";

#[tauri::command]
pub async fn get_discover_movie() -> Result<Value, String> {
    let url = format!(
        "{}/discover/movie?api_key={}&language=tr-TR",
        BASE_URL, TMDB_API_KEY
    );

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;

    let data: Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
pub async fn get_discover_series() -> Result<Value, String> {
    let url = format!(
        "{}/discover/tv?api_key={}&language=tr-TR",
        BASE_URL, TMDB_API_KEY
    );

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;

    let data: Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

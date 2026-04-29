use reqwest;
use serde_json::Value;

const TMDB_API_KEY: &str = "4736e6f3761df4fb134856cc298f2d5e";
const BASE_URL: &str = "https://api.themoviedb.org/3";

#[tauri::command]
pub async fn get_genres(media_type: String) -> Result<Value, String> {
    let url = format!(
        "{}/genre/{}/list?api_key={}&language=tr-TR",
        BASE_URL, media_type, TMDB_API_KEY
    );
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    Ok(response.json().await.map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn get_discover_movie(page: i32, genre_id: Option<i32>) -> Result<Value, String> {
    let genre_param = genre_id.map(|id| format!("&with_genres={}", id)).unwrap_or_default();
    let url = format!(
        "{}/discover/movie?api_key={}&language=tr-TR&page={}{}",
        BASE_URL, TMDB_API_KEY, page, genre_param
    );
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let mut data: Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(results) = data.get_mut("results").and_then(|v| v.as_array_mut()) {
        for item in results { item["media_type"] = serde_json::json!("movie"); }
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_discover_series(page: i32, genre_id: Option<i32>) -> Result<Value, String> {
    let genre_param = genre_id.map(|id| format!("&with_genres={}", id)).unwrap_or_default();
    let url = format!(
        "{}/discover/tv?api_key={}&language=tr-TR&page={}{}",
        BASE_URL, TMDB_API_KEY, page, genre_param
    );
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let mut data: Value = response.json().await.map_err(|e| e.to_string())?;

    if let Some(results) = data.get_mut("results").and_then(|v| v.as_array_mut()) {
        for item in results { item["media_type"] = serde_json::json!("tv"); }
    }
    Ok(data)
}

#[tauri::command]
pub async fn get_item_details(id: i32, media_type: String) -> Result<Value, String> {
    let url = format!(
        "{}/{}/{}?api_key={}&language=tr-TR&append_to_response=videos,credits,external_ids",
        BASE_URL, media_type, id, TMDB_API_KEY
    );
    let res = reqwest::get(url).await.map_err(|e| e.to_string())?;
    Ok(res.json().await.map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn search_all(query: String) -> Result<Value, String> {
    let url = format!(
        "{}/search/multi?api_key={}&language=tr-TR&query={}&include_adult=false",
        BASE_URL, TMDB_API_KEY, query
    );

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let mut data: Value = response.json().await.map_err(|e| e.to_string())?;

    // Sadece film ve dizi sonuçlarını tut, kişileri (oyuncuları) listeden çıkar
    if let Some(results) = data.get_mut("results").and_then(|v| v.as_array_mut()) {
        results.retain(|item| {
            item["media_type"] == "movie" || item["media_type"] == "tv"
        });
    }

    Ok(data)
}
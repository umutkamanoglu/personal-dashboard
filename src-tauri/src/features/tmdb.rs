use reqwest;
use serde_json::Value;

const TMDB_API_KEY: &str = "4736e6f3761df4fb134856cc298f2d5e";
const BASE_URL: &str = "https://api.themoviedb.org/3";

#[tauri::command]
pub async fn get_discover_movie() -> Result<Value, String> {
    let url = format!(
        "{}/discover/movie?api_key={}&language=tr-TR",
        BASE_URL, TMDB_API_KEY
    );
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let mut data: Value = response.json().await.map_err(|e| e.to_string())?;

    // JSON içindeki her bir film sonucuna "media_type: movie" ekliyoruz
    if let Some(results) = data.get_mut("results").and_then(|v| v.as_array_mut()) {
        for item in results {
            item["media_type"] = serde_json::json!("movie");
        }
    }

    Ok(data)
}

#[tauri::command]
pub async fn get_discover_series() -> Result<Value, String> {
    let url = format!(
        "{}/discover/tv?api_key={}&language=tr-TR",
        BASE_URL, TMDB_API_KEY
    );
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let mut data: Value = response.json().await.map_err(|e| e.to_string())?;

    // JSON içindeki her bir dizi sonucuna "media_type: tv" ekliyoruz
    if let Some(results) = data.get_mut("results").and_then(|v| v.as_array_mut()) {
        for item in results {
            item["media_type"] = serde_json::json!("tv");
        }
    }

    Ok(data)
}

#[tauri::command]
pub async fn get_item_details(id: i32, media_type: String) -> Result<Value, String> {
    // Buradaki 'append_to_response' oyuncuları (credits) ve fragmanları (videos) çekmemizi sağlar
    let url = format!(
        "{}/{}/{}?api_key={}&language=tr-TR&append_to_response=videos,credits,external_ids",
        BASE_URL, media_type, id, TMDB_API_KEY
    );

    let res = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
pub async fn search_all(query: String) -> Result<Value, String> {
    let url = format!(
        "{}/search/multi?api_key={}&language=tr-TR&query={}&include_adult=false",
        BASE_URL, TMDB_API_KEY, query
    );

    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let data: Value = response.json().await.map_err(|e| e.to_string())?;

    // Search sonuçlarında zaten media_type (movie/tv) gelir,
    // ama gelmeme ihtimaline karşı veya person (oyuncu) sonuçlarını ayıklamak için
    // küçük bir filtreleme yapabilirsin.
    Ok(data)
}

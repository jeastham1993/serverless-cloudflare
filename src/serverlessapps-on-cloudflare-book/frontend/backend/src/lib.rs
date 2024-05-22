use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use tracing_subscriber::{filter, FmtSubscriber, Layer};
use tracing_subscriber::fmt::format;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use worker::*;
use worker::kv::KvStore;

#[derive(Deserialize)]
struct QueryParameters {
    latitude: f32,
    longitude: f32,
}

impl QueryParameters{
    fn get_cache_key(&self) -> String {
        format!("weather:{}:{}", &self.latitude, &self.longitude)
    }
}

#[derive(Serialize)]
struct WeatherResult {
    forecast: Option<Forecast>,
    message: String
}

#[derive(Serialize, Deserialize)]
struct CurrentWeather {
    time: String,
    temperature: f32
}

#[derive(Serialize, Deserialize)]
struct Forecast {
    latitude: f32,
    longitude: f32,
    current_weather: CurrentWeather
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let stdout_log = tracing_subscriber::fmt::layer()
        .without_time()
        .pretty();

    let init_res = tracing_subscriber::registry()
        .with(
            stdout_log
                // Add an `INFO` filter to the stdout logging layer
                .with_filter(filter::LevelFilter::INFO)
        )
        .try_init();

    info!("Checking cache");

    let cached_todos = env.kv("COUNTRY_CACHE").unwrap();

    let extract_latitude: Result<QueryParameters> = req.query();

    let parameters = match extract_latitude {
        Ok(extracted) => extracted,
        Err(_) => return Ok(Response::from_json(&WeatherResult {
            forecast: None,
            message: "Please provide the 'latitude' and 'longitude' as query parameters.".to_string(),
        })
            .unwrap()
            .with_status(400))
    };

    let region_cache = cached_todos.get(&parameters.get_cache_key()).json().await;

    let res = match region_cache {
        Ok(cached_value) => {
            match cached_value {
                Some(region_data) => {
                    info!("Cache hit");
                    region_data
                }
                _ => retrieve_and_cache(&cached_todos, &parameters).await
            }
        },
        Err(_) => retrieve_and_cache(&cached_todos, &parameters).await
    };

    Ok(Response::from_json(&res).unwrap())
}

async fn retrieve_and_cache(cache: &KvStore, parameters: &QueryParameters) -> Forecast {
    info!("Querying API");

    let response = reqwest::get(format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true", parameters.latitude, parameters.longitude)).await.unwrap();

    let body= response.text().await.unwrap();

    let parsed_body: Forecast = serde_json::from_str(&body).unwrap();

    let result = serde_json::to_string(&body).unwrap();

    info!("Caching");

    let put_res = cache.put(&parameters.get_cache_key(), result.clone());

    parsed_body
}
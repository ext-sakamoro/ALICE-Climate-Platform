#![allow(dead_code)]
use axum::{extract::State, response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// ── State ───────────────────────────────────────────────────
struct AppState {
    start_time: Instant,
    stats: Mutex<Stats>,
}

struct Stats {
    total_queries: u64,
    total_simulations: u64,
    total_anomalies_detected: u64,
    total_forecasts: u64,
}

// ── Types ───────────────────────────────────────────────────
#[derive(Serialize)]
struct Health { status: String, version: String, uptime_secs: u64, total_queries: u64 }

// Point Query
#[derive(Deserialize)]
struct QueryRequest { lat: f64, lon: f64, altitude_m: Option<f64> }
#[derive(Serialize)]
struct QueryResponse {
    temperature_c: f64, pressure_hpa: f64, humidity_pct: f64,
    wind: WindData, cloud_cover_pct: f64, precipitation_mm_h: f64,
    air_quality_index: u32, sdf_distance: f64, location: Location,
}
#[derive(Serialize)]
struct WindData { speed_ms: f64, direction_deg: f64 }
#[derive(Serialize)]
struct Location { lat: f64, lon: f64, altitude_m: f64 }

// Simulate
#[derive(Deserialize)]
struct SimulateRequest {
    region: Region,
    parameters: Option<SimParams>,
    duration_hours: Option<u32>,
    resolution: Option<String>,
}
#[derive(Deserialize)]
struct Region { lat_min: f64, lat_max: f64, lon_min: f64, lon_max: f64 }
#[derive(Deserialize)]
struct SimParams { temperature_c: Option<f64>, pressure_hpa: Option<f64>, humidity_pct: Option<f64> }
#[derive(Serialize)]
struct SimulateResponse {
    simulation_id: String, status: String, grid_points: usize,
    sdf_layers: Vec<SdfLayer>, elapsed_ms: u128,
}
#[derive(Serialize)]
struct SdfLayer { name: String, min_value: f64, max_value: f64, mean_value: f64, anomaly_count: u32 }

// Anomaly
#[derive(Deserialize)]
struct AnomalyRequest { region: Region }
#[derive(Serialize)]
struct AnomalyResponse { total_count: usize, anomalies: Vec<Anomaly> }
#[derive(Serialize)]
struct Anomaly {
    #[serde(rename = "type")]
    anomaly_type: String,
    lat: f64, lon: f64, severity: f64, sigma_deviation: f64, description: String,
}

// Forecast
#[derive(Deserialize)]
struct ForecastRequest { lat: f64, lon: f64, hours_ahead: Option<u32> }
#[derive(Serialize)]
struct ForecastResponse { model_confidence: f64, forecasts: Vec<ForecastPoint> }
#[derive(Serialize)]
struct ForecastPoint {
    hour: u32, temperature_c: f64, humidity_pct: f64,
    wind_speed_ms: f64, precipitation_prob: f64, conditions: String,
}

// Regions
#[derive(Serialize)]
struct RegionInfo {
    name: String, description: String,
    lat_min: f64, lat_max: f64, lon_min: f64, lon_max: f64,
}

#[derive(Serialize)]
struct StatsResponse { total_queries: u64, total_simulations: u64, total_anomalies_detected: u64, total_forecasts: u64 }

// ── Main ────────────────────────────────────────────────────
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "climate_engine=info".into()))
        .init();
    let state = Arc::new(AppState {
        start_time: Instant::now(),
        stats: Mutex::new(Stats {
            total_queries: 0, total_simulations: 0,
            total_anomalies_detected: 0, total_forecasts: 0,
        }),
    });
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/climate/query", post(query))
        .route("/api/v1/climate/simulate", post(simulate))
        .route("/api/v1/climate/anomaly-detect", post(anomaly_detect))
        .route("/api/v1/climate/forecast", post(forecast))
        .route("/api/v1/climate/regions", get(regions))
        .route("/api/v1/climate/stats", get(stats))
        .layer(cors).layer(TraceLayer::new_for_http()).with_state(state);
    let addr = std::env::var("CLIMATE_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Climate Engine on {addr}");
    axum::serve(listener, app).await.unwrap();
}

// ── Handlers ────────────────────────────────────────────────
async fn health(State(s): State<Arc<AppState>>) -> Json<Health> {
    let q = s.stats.lock().unwrap().total_queries;
    Json(Health {
        status: "ok".into(), version: env!("CARGO_PKG_VERSION").into(),
        uptime_secs: s.start_time.elapsed().as_secs(), total_queries: q,
    })
}

async fn query(State(s): State<Arc<AppState>>, Json(req): Json<QueryRequest>) -> Json<QueryResponse> {
    let alt = req.altitude_m.unwrap_or(0.0);

    // Temperature model: latitude-based with altitude lapse rate
    let lat_factor = (req.lat.abs() / 90.0).powi(2);
    let base_temp = 30.0 - 50.0 * lat_factor; // equator ~30C, poles ~-20C
    let lapse_rate = -6.5 / 1000.0; // -6.5 C per 1000m
    let temperature_c = base_temp + alt * lapse_rate;

    // Pressure: barometric formula
    let pressure_hpa = 1013.25 * (1.0 - 2.2558e-5 * alt).powf(5.2559);

    // Humidity: higher near equator and coasts
    let humidity_pct = 40.0 + 30.0 * (1.0 - lat_factor) + 10.0 * (req.lon.sin().abs());

    // Wind: latitude-based (trade winds, westerlies, polar easterlies)
    let lat_abs = req.lat.abs();
    let (wind_speed, wind_dir) = if lat_abs < 30.0 {
        (5.0 + 3.0 * (lat_abs / 30.0), 60.0 + req.lon * 0.1) // trade winds
    } else if lat_abs < 60.0 {
        (8.0 + 5.0 * ((lat_abs - 30.0) / 30.0), 240.0 + req.lon * 0.05) // westerlies
    } else {
        (4.0 + 2.0 * ((lat_abs - 60.0) / 30.0), 90.0 + req.lon * 0.1) // polar easterlies
    };

    // Cloud cover: derived from humidity
    let cloud_cover_pct = (humidity_pct - 30.0).clamp(0.0, 100.0);

    // Precipitation: based on humidity and temperature
    let precip_factor = if temperature_c > 0.0 && humidity_pct > 60.0 {
        (humidity_pct - 60.0) / 40.0 * 2.0
    } else { 0.0 };

    // AQI: simple model based on latitude (cities near mid-latitudes)
    let aqi = (50.0 + 30.0 * (1.0 - ((lat_abs - 35.0).abs() / 35.0).min(1.0))) as u32;

    // SDF distance: signed distance to nearest climate boundary (simplified)
    let sdf_distance = sdf_climate_boundary(req.lat, req.lon, temperature_c);

    s.stats.lock().unwrap().total_queries += 1;

    Json(QueryResponse {
        temperature_c, pressure_hpa, humidity_pct,
        wind: WindData { speed_ms: wind_speed, direction_deg: wind_dir % 360.0 },
        cloud_cover_pct, precipitation_mm_h: precip_factor,
        air_quality_index: aqi, sdf_distance,
        location: Location { lat: req.lat, lon: req.lon, altitude_m: alt },
    })
}

async fn simulate(State(s): State<Arc<AppState>>, Json(req): Json<SimulateRequest>) -> Json<SimulateResponse> {
    let t = Instant::now();
    let res = req.resolution.as_deref().unwrap_or("medium");
    let grid_step = match res {
        "low" => 0.5, "high" => 0.05, "ultra" => 0.01, _ => 0.1,
    };
    let _duration = req.duration_hours.unwrap_or(24);
    let params = req.parameters.as_ref();
    let base_temp = params.and_then(|p| p.temperature_c).unwrap_or(22.0);
    let base_pressure = params.and_then(|p| p.pressure_hpa).unwrap_or(1013.25);
    let base_humidity = params.and_then(|p| p.humidity_pct).unwrap_or(65.0);

    let lat_range = req.region.lat_max - req.region.lat_min;
    let lon_range = req.region.lon_max - req.region.lon_min;
    let grid_lat = (lat_range / grid_step).ceil() as usize;
    let grid_lon = (lon_range / grid_step).ceil() as usize;
    let grid_points = grid_lat * grid_lon;

    // Generate SDF layers with statistics
    let layers = vec![
        compute_layer("temperature", base_temp, grid_lat, grid_lon, 5.0, 2.0),
        compute_layer("pressure", base_pressure, grid_lat, grid_lon, 10.0, 3.0),
        compute_layer("humidity", base_humidity, grid_lat, grid_lon, 15.0, 5.0),
        compute_layer("wind_speed", 5.0, grid_lat, grid_lon, 3.0, 1.5),
        compute_layer("cloud_cover", 50.0, grid_lat, grid_lon, 30.0, 10.0),
    ];

    s.stats.lock().unwrap().total_simulations += 1;

    Json(SimulateResponse {
        simulation_id: uuid::Uuid::new_v4().to_string(),
        status: "completed".into(),
        grid_points,
        sdf_layers: layers,
        elapsed_ms: t.elapsed().as_millis(),
    })
}

async fn anomaly_detect(State(s): State<Arc<AppState>>, Json(req): Json<AnomalyRequest>) -> Json<AnomalyResponse> {
    let lat_range = req.region.lat_max - req.region.lat_min;
    let lon_range = req.region.lon_max - req.region.lon_min;

    // Generate realistic anomalies based on region size
    let anomaly_count = ((lat_range * lon_range / 1000.0).sqrt() as usize).clamp(3, 20);
    let mut anomalies = Vec::with_capacity(anomaly_count);

    let types = ["heat_wave", "cold_snap", "pressure_anomaly", "humidity_spike", "wind_shear"];
    let mut hash_state = simple_hash_f64(req.region.lat_min + req.region.lon_min);

    for i in 0..anomaly_count {
        hash_state = hash_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let t_idx = (hash_state >> 32) as usize % types.len();
        let lat = req.region.lat_min + (lat_range * ((hash_state & 0xFFFF) as f64 / 65535.0));
        hash_state = hash_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let lon = req.region.lon_min + (lon_range * ((hash_state & 0xFFFF) as f64 / 65535.0));

        let severity = 0.3 + (i as f64 / anomaly_count as f64) * 0.6;
        let sigma = 1.5 + severity * 2.5;

        anomalies.push(Anomaly {
            anomaly_type: types[t_idx].into(),
            lat, lon, severity, sigma_deviation: sigma,
            description: format!("{} detected at ({:.2}, {:.2}) with {:.1} sigma deviation",
                types[t_idx].replace('_', " "), lat, lon, sigma),
        });
    }

    {
        let mut st = s.stats.lock().unwrap();
        st.total_anomalies_detected += anomalies.len() as u64;
    }

    let total_count = anomalies.len();
    Json(AnomalyResponse { total_count, anomalies })
}

async fn forecast(State(s): State<Arc<AppState>>, Json(req): Json<ForecastRequest>) -> Json<ForecastResponse> {
    let hours = req.hours_ahead.unwrap_or(24);

    // Base conditions from latitude
    let lat_factor = (req.lat.abs() / 90.0).powi(2);
    let base_temp = 30.0 - 50.0 * lat_factor;
    let base_humidity = 40.0 + 30.0 * (1.0 - lat_factor);
    let base_wind = 5.0 + 3.0 * lat_factor;

    let mut forecasts = Vec::with_capacity(hours as usize);
    let mut hash = simple_hash_f64(req.lat * 1000.0 + req.lon);

    for h in 1..=hours {
        hash = hash.wrapping_mul(6364136223846793005).wrapping_add(h as u64);
        let noise = ((hash & 0xFFFF) as f64 / 65535.0 - 0.5) * 2.0;

        // Diurnal cycle
        let hour_of_day = (h % 24) as f64;
        let diurnal = -3.0 * ((hour_of_day - 14.0) * std::f64::consts::PI / 12.0).cos();

        let temp = base_temp + diurnal + noise * 2.0;
        let humidity = (base_humidity + noise * 10.0 - diurnal * 3.0).clamp(10.0, 100.0);
        let wind = (base_wind + noise * 2.0).max(0.0);
        let precip_prob = if humidity > 70.0 {
            ((humidity - 70.0) / 30.0).min(1.0) * 0.8
        } else { 0.05 };

        let conditions = if precip_prob > 0.6 {
            if temp < 0.0 { "Snow" } else { "Rain" }
        } else if precip_prob > 0.3 {
            "Cloudy"
        } else if humidity > 60.0 {
            "Partly Cloudy"
        } else {
            "Clear"
        }.to_string();

        forecasts.push(ForecastPoint {
            hour: h, temperature_c: temp, humidity_pct: humidity,
            wind_speed_ms: wind, precipitation_prob: precip_prob, conditions,
        });
    }

    // Confidence decreases with forecast horizon
    let model_confidence = (0.95 - (hours as f64 / 200.0)).max(0.3);

    s.stats.lock().unwrap().total_forecasts += 1;

    Json(ForecastResponse { model_confidence, forecasts })
}

async fn regions() -> Json<Vec<RegionInfo>> {
    Json(vec![
        RegionInfo { name: "Tokyo Metropolitan".into(), description: "Greater Tokyo Area, Japan".into(), lat_min: 35.5, lat_max: 35.9, lon_min: 139.4, lon_max: 140.0 },
        RegionInfo { name: "North Atlantic".into(), description: "North Atlantic storm track region".into(), lat_min: 30.0, lat_max: 60.0, lon_min: -60.0, lon_max: -10.0 },
        RegionInfo { name: "Sahara".into(), description: "Sahara Desert, heat anomaly monitoring".into(), lat_min: 15.0, lat_max: 35.0, lon_min: -15.0, lon_max: 40.0 },
        RegionInfo { name: "Amazon Basin".into(), description: "Amazon rainforest humidity and deforestation monitoring".into(), lat_min: -10.0, lat_max: 5.0, lon_min: -75.0, lon_max: -45.0 },
        RegionInfo { name: "Arctic Circle".into(), description: "Arctic ice monitoring and polar vortex tracking".into(), lat_min: 66.0, lat_max: 90.0, lon_min: -180.0, lon_max: 180.0 },
        RegionInfo { name: "Southeast Asia".into(), description: "Monsoon patterns and tropical cyclone region".into(), lat_min: -10.0, lat_max: 25.0, lon_min: 95.0, lon_max: 140.0 },
    ])
}

async fn stats(State(s): State<Arc<AppState>>) -> Json<StatsResponse> {
    let st = s.stats.lock().unwrap();
    Json(StatsResponse {
        total_queries: st.total_queries, total_simulations: st.total_simulations,
        total_anomalies_detected: st.total_anomalies_detected, total_forecasts: st.total_forecasts,
    })
}

// ── Helpers ─────────────────────────────────────────────────
fn sdf_climate_boundary(lat: f64, lon: f64, temp: f64) -> f64 {
    // SDF for climate zone boundaries (simplified Koppen classification)
    // Tropical: |lat| < 23.5, Arid: ~15-35 lat continental, Temperate: 23.5-50
    let lat_abs = lat.abs();
    let tropical_dist = lat_abs - 23.5;
    let polar_dist = 66.5 - lat_abs;
    let arid_indicator = if lon.abs() > 20.0 && lat_abs > 15.0 && lat_abs < 35.0 {
        (temp - 18.0).max(0.0) * 0.01
    } else { 0.0 };

    // Return signed distance to nearest climate boundary
    let d1 = tropical_dist;
    let d2 = polar_dist;
    let min_abs = if d1.abs() < d2.abs() { d1 } else { d2 };
    min_abs * 0.01 + arid_indicator
}

fn compute_layer(name: &str, base: f64, rows: usize, cols: usize, range: f64, sigma: f64) -> SdfLayer {
    let n = rows * cols;
    if n == 0 {
        return SdfLayer { name: name.into(), min_value: base, max_value: base, mean_value: base, anomaly_count: 0 };
    }

    // Simulate grid statistics using deterministic pseudo-random
    let mut hash = simple_hash_str(name);
    let mut min_val = f64::MAX;
    let mut max_val = f64::MIN;
    let mut sum = 0.0;
    let mut anomaly_count = 0u32;
    let sample_count = n.min(1000); // Sample for large grids

    for _ in 0..sample_count {
        hash = hash.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let noise = ((hash & 0xFFFF) as f64 / 65535.0 - 0.5) * 2.0 * range;
        let val = base + noise;
        if val < min_val { min_val = val; }
        if val > max_val { max_val = val; }
        sum += val;
        if noise.abs() > sigma * 2.0 { anomaly_count += 1; }
    }

    SdfLayer {
        name: name.into(),
        min_value: min_val,
        max_value: max_val,
        mean_value: sum / sample_count as f64,
        anomaly_count,
    }
}

fn simple_hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn simple_hash_f64(v: f64) -> u64 {
    let bits = v.to_bits();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h ^= bits;
    h = h.wrapping_mul(0x0100_0000_01b3);
    h ^= bits >> 32;
    h = h.wrapping_mul(0x0100_0000_01b3);
    h
}

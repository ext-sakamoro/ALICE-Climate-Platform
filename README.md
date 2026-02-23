# ALICE Climate Platform

SDF-based climate simulation engine — point queries, atmospheric modeling, anomaly detection, and forecasting via REST API.

**License: AGPL-3.0**

---

## Architecture

```
                    ┌─────────────────┐
                    │   Browser / UI  │
                    │  Next.js :3000  │
                    └────────┬────────┘
                             │ HTTP
                    ┌────────▼────────┐
                    │   API Gateway   │
                    │     :8080       │
                    └────────┬────────┘
                             │ HTTP
                    ┌────────▼────────┐
                    │  Climate Engine │
                    │  Rust/Axum      │
                    │    :8081        │
                    └─────────────────┘
```

| Service | Port | Description |
|---------|------|-------------|
| Frontend | 3000 | Next.js dashboard |
| API Gateway | 8080 | Reverse proxy / auth |
| Climate Engine | 8081 | Rust/Axum core engine |

---

## API Endpoints

### POST /api/v1/climate/query

Query climate data at a geographic point.

**Request:**
```json
{
  "lat": 35.6762,
  "lon": 139.6503,
  "altitude_m": 40
}
```

**Response:**
```json
{
  "temperature_c": 18.3,
  "pressure_hpa": 1012.8,
  "humidity_pct": 62.5,
  "wind": { "speed_ms": 6.2, "direction_deg": 245.3 },
  "cloud_cover_pct": 32.5,
  "precipitation_mm_h": 0.0,
  "air_quality_index": 55,
  "sdf_distance": 0.0042,
  "location": { "lat": 35.6762, "lon": 139.6503, "altitude_m": 40 }
}
```

Temperature model uses latitude-based gradient with altitude lapse rate (-6.5 C/km).

---

### POST /api/v1/climate/simulate

Run a climate simulation over a geographic region.

**Request:**
```json
{
  "region": { "lat_min": 35.5, "lat_max": 35.8, "lon_min": 139.5, "lon_max": 139.9 },
  "parameters": { "temperature_c": 22.0, "pressure_hpa": 1013.25, "humidity_pct": 65.0 },
  "duration_hours": 24,
  "resolution": "medium"
}
```

**Response:**
```json
{
  "simulation_id": "...",
  "status": "completed",
  "grid_points": 1200,
  "sdf_layers": [
    { "name": "temperature", "min_value": 17.2, "max_value": 26.8, "mean_value": 22.0, "anomaly_count": 3 },
    { "name": "pressure", "min_value": 1003.2, "max_value": 1023.3, "mean_value": 1013.2, "anomaly_count": 0 }
  ],
  "elapsed_ms": 45
}
```

Resolution options: `low` (0.5 deg) | `medium` (0.1 deg) | `high` (0.05 deg) | `ultra` (0.01 deg)

---

### POST /api/v1/climate/anomaly-detect

Detect climate anomalies within a region.

**Request:**
```json
{
  "region": { "lat_min": -90, "lat_max": 90, "lon_min": -180, "lon_max": 180 }
}
```

**Response:**
```json
{
  "total_count": 8,
  "anomalies": [
    {
      "type": "heat_wave",
      "lat": 42.15,
      "lon": -87.32,
      "severity": 0.82,
      "sigma_deviation": 3.55,
      "description": "heat wave detected at (42.15, -87.32) with 3.6 sigma deviation"
    }
  ]
}
```

Anomaly types: `heat_wave` | `cold_snap` | `pressure_anomaly` | `humidity_spike` | `wind_shear`

---

### POST /api/v1/climate/forecast

Generate a multi-hour forecast for a location.

**Request:**
```json
{
  "lat": 35.6762,
  "lon": 139.6503,
  "hours_ahead": 24
}
```

**Response:**
```json
{
  "model_confidence": 0.83,
  "forecasts": [
    {
      "hour": 1,
      "temperature_c": 19.2,
      "humidity_pct": 58.0,
      "wind_speed_ms": 5.3,
      "precipitation_prob": 0.12,
      "conditions": "Partly Cloudy"
    }
  ]
}
```

Conditions: `Clear` | `Partly Cloudy` | `Cloudy` | `Rain` | `Snow`

---

### GET /api/v1/climate/regions

List predefined climate monitoring regions.

| Region | Description |
|--------|-------------|
| Tokyo Metropolitan | Greater Tokyo Area |
| North Atlantic | Storm track region |
| Sahara | Heat anomaly monitoring |
| Amazon Basin | Humidity and deforestation |
| Arctic Circle | Ice and polar vortex |
| Southeast Asia | Monsoon and tropical cyclone |

---

### GET /api/v1/climate/stats

Server-wide statistics.

---

### GET /health

Health check endpoint.

---

## Quick Start

### Climate Engine (Rust)

```bash
cd services/core-engine
cargo build --release
CLIMATE_ADDR=0.0.0.0:8081 ./target/release/climate-engine
```

### Frontend (Next.js)

```bash
cd frontend
npm install
npm run dev
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CLIMATE_ADDR` | `0.0.0.0:8081` | Engine bind address |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | API base URL for frontend |

---

## License

AGPL-3.0 — See [LICENSE](LICENSE) for details.

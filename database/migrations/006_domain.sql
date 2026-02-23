-- Climate Platform domain tables
create table if not exists public.climate_observations (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    station_id text not null,
    latitude double precision not null,
    longitude double precision not null,
    altitude_m double precision default 0.0,
    temperature_c double precision,
    humidity_pct double precision,
    pressure_hpa double precision,
    wind_speed_ms double precision,
    wind_direction_deg double precision,
    precipitation_mm double precision default 0.0,
    observed_at timestamptz not null,
    created_at timestamptz default now()
);
create table if not exists public.climate_simulations (
    id uuid primary key default gen_random_uuid(),
    user_id uuid references auth.users(id) on delete cascade,
    name text not null,
    model_type text not null default 'basic' check (model_type in ('basic', 'sdf_boundary', 'forecast', 'anomaly_detection')),
    region_bbox jsonb,
    time_range_start timestamptz,
    time_range_end timestamptz,
    resolution_km double precision default 10.0,
    status text default 'pending',
    results jsonb default '{}',
    created_at timestamptz default now(),
    completed_at timestamptz
);
create index idx_climate_obs_user on public.climate_observations(user_id);
create index idx_climate_obs_station on public.climate_observations(station_id, observed_at);
create index idx_climate_obs_location on public.climate_observations(latitude, longitude);
create index idx_climate_sims_user on public.climate_simulations(user_id);

'use client';
import { useState } from 'react';
import { useClimateStore } from '@/lib/hooks/use-store';

const API = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
type Tab = 'query' | 'simulate' | 'anomaly' | 'forecast' | 'regions';
const RESOLUTIONS = ['low', 'medium', 'high', 'ultra'];

export default function ConsolePage() {
  const [tab, setTab] = useState<Tab>('query');
  const { lat, setLat, lon, setLon, altitude, setAltitude, result, setResult, loading, setLoading } = useClimateStore();
  const [resolution, setResolution] = useState('medium');
  const [hours, setHours] = useState('24');

  const doFetch = async (path: string, body: unknown) => {
    setLoading(true); setResult(null);
    try {
      const r = await fetch(`${API}${path}`, { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-API-Key': 'demo' }, body: JSON.stringify(body) });
      setResult(await r.json());
    } catch (e) { setResult({ error: (e as Error).message }); } finally { setLoading(false); }
  };

  const doGet = async (path: string) => {
    setLoading(true); setResult(null);
    try {
      const r = await fetch(`${API}${path}`, { headers: { 'X-API-Key': 'demo' } });
      setResult(await r.json());
    } catch (e) { setResult({ error: (e as Error).message }); } finally { setLoading(false); }
  };

  const tabs: { key: Tab; label: string }[] = [
    { key: 'query', label: 'Point Query' },
    { key: 'simulate', label: 'Simulate' },
    { key: 'anomaly', label: 'Anomalies' },
    { key: 'forecast', label: 'Forecast' },
    { key: 'regions', label: 'Regions' },
  ];

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-2xl font-bold">Climate Console</h1>
      <div className="flex gap-1 border-b border-border overflow-x-auto">
        {tabs.map((t) => (
          <button key={t.key} onClick={() => { setTab(t.key); setResult(null); }}
            className={`px-4 py-2 text-sm font-medium border-b-2 whitespace-nowrap transition-colors ${tab === t.key ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'}`}>
            {t.label}
          </button>
        ))}
      </div>

      {tab === 'query' && (
        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-3">
            <Inp label="Latitude" value={lat} onChange={setLat} />
            <Inp label="Longitude" value={lon} onChange={setLon} />
            <Inp label="Altitude (m)" value={altitude} onChange={setAltitude} />
          </div>
          <button onClick={() => doFetch('/api/v1/climate/query', { lat: +lat, lon: +lon, altitude_m: +altitude })}
            disabled={loading} className="px-4 py-2 bg-sky-600 text-white rounded-md text-sm font-medium hover:bg-sky-700 disabled:opacity-50">
            {loading ? 'Querying...' : 'Query Climate'}
          </button>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <h3 className="text-sm font-semibold">Climate Data</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                <Stat label="Temperature" value={`${Number(result.temperature_c ?? 0).toFixed(1)} C`} accent />
                <Stat label="Pressure" value={`${Number(result.pressure_hpa ?? 0).toFixed(1)} hPa`} />
                <Stat label="Humidity" value={`${Number(result.humidity_pct ?? 0).toFixed(1)}%`} />
                {result.wind && <Stat label="Wind" value={`${Number((result.wind as Record<string, number>).speed_ms ?? 0).toFixed(1)} m/s @ ${Number((result.wind as Record<string, number>).direction_deg ?? 0).toFixed(0)} deg`} />}
                <Stat label="Cloud Cover" value={`${Number(result.cloud_cover_pct ?? 0).toFixed(0)}%`} />
                <Stat label="Precipitation" value={`${Number(result.precipitation_mm_h ?? 0).toFixed(2)} mm/h`} />
                <Stat label="AQI" value={String(result.air_quality_index ?? '-')} />
                <Stat label="SDF Distance" value={Number(result.sdf_distance ?? 0).toFixed(4)} />
              </div>
            </div>
          )}
        </div>
      )}

      {tab === 'simulate' && (
        <div className="space-y-4">
          <h3 className="text-sm font-semibold">Region (Tokyo Area)</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Inp label="Lat Min" value="35.5" onChange={() => {}} />
            <Inp label="Lat Max" value="35.8" onChange={() => {}} />
            <Inp label="Lon Min" value="139.5" onChange={() => {}} />
            <Inp label="Lon Max" value="139.9" onChange={() => {}} />
          </div>
          <div className="flex gap-2 items-center">
            <label className="text-xs font-medium text-muted-foreground">Resolution:</label>
            {RESOLUTIONS.map((r) => (
              <button key={r} onClick={() => setResolution(r)}
                className={`px-3 py-1.5 rounded-md text-xs font-medium ${resolution === r ? 'bg-sky-600 text-white' : 'bg-muted text-muted-foreground'}`}>{r}</button>
            ))}
          </div>
          <button onClick={() => doFetch('/api/v1/climate/simulate', {
            region: { lat_min: 35.5, lat_max: 35.8, lon_min: 139.5, lon_max: 139.9 },
            parameters: { temperature_c: 22.0, pressure_hpa: 1013.25, humidity_pct: 65.0 },
            duration_hours: 24, resolution,
          })} disabled={loading} className="px-4 py-2 bg-sky-600 text-white rounded-md text-sm font-medium hover:bg-sky-700 disabled:opacity-50">
            {loading ? 'Simulating...' : 'Run Simulation'}
          </button>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
                <Stat label="Grid Points" value={String(result.grid_points ?? '-')} accent />
                <Stat label="Status" value={String(result.status ?? '-')} />
                <Stat label="Time" value={`${result.elapsed_ms} ms`} />
              </div>
              {Array.isArray(result.sdf_layers) && (
                <div><h4 className="text-xs font-semibold text-muted-foreground mb-2">SDF Layers</h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {(result.sdf_layers as Array<Record<string, unknown>>).map((l) => (
                      <div key={String(l.name)} className="px-3 py-2 bg-muted rounded-md text-xs">
                        <span className="font-semibold text-sky-400">{String(l.name)}</span>
                        <span className="text-muted-foreground ml-2">min: {Number(l.min_value).toFixed(2)} | max: {Number(l.max_value).toFixed(2)} | mean: {Number(l.mean_value).toFixed(2)}</span>
                        {Number(l.anomaly_count) > 0 && <span className="ml-2 text-red-400">anomalies: {String(l.anomaly_count)}</span>}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {tab === 'anomaly' && (
        <div className="space-y-4">
          <button onClick={() => doFetch('/api/v1/climate/anomaly-detect', {
            region: { lat_min: -90, lat_max: 90, lon_min: -180, lon_max: 180 },
          })} disabled={loading} className="px-4 py-2 bg-red-600 text-white rounded-md text-sm font-medium hover:bg-red-700 disabled:opacity-50">
            {loading ? 'Detecting...' : 'Detect Global Anomalies'}
          </button>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              <Stat label="Total Anomalies" value={String(result.total_count ?? 0)} accent />
              {Array.isArray(result.anomalies) && (
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-muted-foreground border-b border-border">
                    <th className="py-1 pr-3">Type</th><th className="py-1 pr-3">Location</th><th className="py-1 pr-3">Severity</th><th className="py-1">Sigma</th>
                  </tr></thead>
                  <tbody>
                    {(result.anomalies as Array<Record<string, unknown>>).slice(0, 10).map((a, i) => (
                      <tr key={i} className="border-b border-border/50">
                        <td className="py-1 pr-3 text-red-400 font-medium">{String(a.type)}</td>
                        <td className="py-1 pr-3 text-xs font-mono">{Number(a.lat).toFixed(2)}, {Number(a.lon).toFixed(2)}</td>
                        <td className="py-1 pr-3">{Number(a.severity).toFixed(2)}</td>
                        <td className="py-1">{Number(a.sigma_deviation).toFixed(2)} sigma</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}
        </div>
      )}

      {tab === 'forecast' && (
        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-3">
            <Inp label="Latitude" value={lat} onChange={setLat} />
            <Inp label="Longitude" value={lon} onChange={setLon} />
            <Inp label="Hours Ahead" value={hours} onChange={setHours} />
          </div>
          <button onClick={() => doFetch('/api/v1/climate/forecast', { lat: +lat, lon: +lon, hours_ahead: +hours })}
            disabled={loading} className="px-4 py-2 bg-sky-600 text-white rounded-md text-sm font-medium hover:bg-sky-700 disabled:opacity-50">
            {loading ? 'Forecasting...' : 'Get Forecast'}
          </button>
          {result && !('error' in result) && (
            <div className="border border-border rounded-lg p-4 space-y-3">
              {result.model_confidence != null && <Stat label="Confidence" value={`${(Number(result.model_confidence) * 100).toFixed(0)}%`} accent />}
              {Array.isArray(result.forecasts) && (
                <table className="w-full text-sm">
                  <thead><tr className="text-left text-muted-foreground border-b border-border">
                    <th className="py-1 pr-3">Hour</th><th className="py-1 pr-3">Temp</th><th className="py-1 pr-3">Humidity</th><th className="py-1 pr-3">Wind</th><th className="py-1 pr-3">Precip</th><th className="py-1">Conditions</th>
                  </tr></thead>
                  <tbody>
                    {(result.forecasts as Array<Record<string, unknown>>).slice(0, 12).map((f, i) => (
                      <tr key={i} className="border-b border-border/50">
                        <td className="py-1 pr-3">+{String(f.hour)}h</td>
                        <td className="py-1 pr-3">{Number(f.temperature_c).toFixed(1)} C</td>
                        <td className="py-1 pr-3">{Number(f.humidity_pct).toFixed(0)}%</td>
                        <td className="py-1 pr-3">{Number(f.wind_speed_ms).toFixed(1)} m/s</td>
                        <td className="py-1 pr-3">{(Number(f.precipitation_prob) * 100).toFixed(0)}%</td>
                        <td className="py-1 text-sky-400">{String(f.conditions)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}
        </div>
      )}

      {tab === 'regions' && (
        <div className="space-y-4">
          <button onClick={() => doGet('/api/v1/climate/regions')} disabled={loading}
            className="px-4 py-2 bg-sky-600 text-white rounded-md text-sm font-medium hover:bg-sky-700 disabled:opacity-50">
            {loading ? 'Loading...' : 'Load Regions'}
          </button>
          {result && Array.isArray(result) && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {(result as Array<Record<string, unknown>>).map((r) => (
                <div key={String(r.name)} className="border border-border rounded-lg p-4">
                  <h3 className="font-semibold">{String(r.name)}</h3>
                  <p className="text-xs text-muted-foreground mt-1">{String(r.description)}</p>
                  <div className="text-xs font-mono text-muted-foreground mt-2">
                    Lat: {Number(r.lat_min).toFixed(0)} to {Number(r.lat_max).toFixed(0)} | Lon: {Number(r.lon_min).toFixed(0)} to {Number(r.lon_max).toFixed(0)}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {result && 'error' in result && <p className="text-sm text-red-500">{String(result.error)}</p>}
    </div>
  );
}

function Stat({ label, value, accent }: { label: string; value: string; accent?: boolean }) {
  return (
    <div className="px-3 py-2 bg-muted rounded-md">
      <div className="text-xs text-muted-foreground">{label}</div>
      <div className={`text-sm font-semibold ${accent ? 'text-sky-400' : ''}`}>{value}</div>
    </div>
  );
}

function Inp({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <div>
      <label className="text-xs font-medium text-muted-foreground block mb-1">{label}</label>
      <input value={value} onChange={(e) => onChange(e.target.value)} className="w-full px-3 py-2 border border-input rounded-md bg-background text-sm font-mono" />
    </div>
  );
}

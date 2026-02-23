const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...options?.headers },
  });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export const api = {
  health: () => request<{ status: string; version: string }>('/health'),
  simulate: (body: { region: { lat_min: number; lat_max: number; lon_min: number; lon_max: number }; parameters: { temperature_c: number; pressure_hpa: number; humidity_pct: number }; duration_hours: number; resolution?: string }) =>
    request('/api/v1/climate/simulate', { method: 'POST', body: JSON.stringify(body) }),
  query: (body: { lat: number; lon: number; altitude_m?: number }) =>
    request('/api/v1/climate/query', { method: 'POST', body: JSON.stringify(body) }),
  anomalyDetect: (body: { region: { lat_min: number; lat_max: number; lon_min: number; lon_max: number } }) =>
    request('/api/v1/climate/anomaly-detect', { method: 'POST', body: JSON.stringify(body) }),
  forecast: (body: { lat: number; lon: number; hours_ahead: number }) =>
    request('/api/v1/climate/forecast', { method: 'POST', body: JSON.stringify(body) }),
  regions: () => request('/api/v1/climate/regions'),
  stats: () => request('/api/v1/climate/stats'),
};

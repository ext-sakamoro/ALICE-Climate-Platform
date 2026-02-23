import { create } from 'zustand';

interface ClimateState {
  lat: string;
  lon: string;
  altitude: string;
  result: Record<string, unknown> | null;
  loading: boolean;
  setLat: (v: string) => void;
  setLon: (v: string) => void;
  setAltitude: (v: string) => void;
  setResult: (v: Record<string, unknown> | null) => void;
  setLoading: (v: boolean) => void;
  reset: () => void;
}

export const useClimateStore = create<ClimateState>((set) => ({
  lat: '35.6762',
  lon: '139.6503',
  altitude: '40',
  result: null,
  loading: false,
  setLat: (lat) => set({ lat }),
  setLon: (lon) => set({ lon }),
  setAltitude: (altitude) => set({ altitude }),
  setResult: (result) => set({ result }),
  setLoading: (loading) => set({ loading }),
  reset: () => set({ lat: '35.6762', lon: '139.6503', altitude: '40', result: null, loading: false }),
}));

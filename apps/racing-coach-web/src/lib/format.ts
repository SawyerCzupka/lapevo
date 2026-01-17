/**
 * Formatting utilities for displaying racing data
 */

/**
 * Format lap time in MM:SS.mmm format
 */
export function formatLapTime(seconds: number | null | undefined): string {
  if (seconds == null || seconds === 0) return '--:--.---';

  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;

  return `${minutes}:${secs.toFixed(3).padStart(6, '0')}`;
}

/**
 * Format speed from m/s to km/h or mph
 */
export function formatSpeed(metersPerSecond: number, unit: 'kmh' | 'mph' = 'kmh'): string {
  if (unit === 'mph') {
    const mph = metersPerSecond * 2.23694;
    return `${mph.toFixed(1)} mph`;
  }

  const kmh = metersPerSecond * 3.6;
  return `${kmh.toFixed(1)} km/h`;
}

/**
 * Format temperature
 */
export function formatTemperature(celsius: number, unit: 'C' | 'F' = 'C'): string {
  if (unit === 'F') {
    const fahrenheit = (celsius * 9) / 5 + 32;
    return `${fahrenheit.toFixed(1)}°F`;
  }

  return `${celsius.toFixed(1)}°C`;
}

/**
 * Format distance in meters to km or miles
 */
export function formatDistance(meters: number, unit: 'km' | 'mi' = 'km'): string {
  if (unit === 'mi') {
    const miles = meters / 1609.34;
    return `${miles.toFixed(2)} mi`;
  }

  const km = meters / 1000;
  return `${km.toFixed(2)} km`;
}

/**
 * Format G-force
 */
export function formatGForce(gForce: number): string {
  return `${gForce.toFixed(2)}G`;
}

/**
 * Format percentage
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  return `${(value * 100).toFixed(decimals)}%`;
}

/**
 * Format time delta (difference between two lap times)
 */
export function formatDelta(seconds: number): string {
  const sign = seconds >= 0 ? '+' : '';
  return `${sign}${seconds.toFixed(3)}s`;
}

/**
 * Get color for delta time (green = faster, red = slower)
 */
export function getDeltaColor(delta: number): string {
  if (delta < -0.01) return 'text-green-400'; // Faster (negative delta)
  if (delta > 0.01) return 'text-red-400'; // Slower (positive delta)
  return 'text-muted-foreground'; // Neutral
}

/**
 * Format date/time for display
 */
export function formatDateTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(d);
}

/**
 * Format relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date;
  const now = new Date();
  const diffInSeconds = Math.floor((now.getTime() - d.getTime()) / 1000);

  if (diffInSeconds < 60) return 'just now';
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} minutes ago`;
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} hours ago`;
  if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)} days ago`;

  return formatDateTime(d);
}

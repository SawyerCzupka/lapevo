/**
 * Centralized chart colors that match the theme.
 * These colors are designed to work with the dark theme and
 * align with the lapevo-marketing site's teal-based design system.
 */

export const chartColors = {
  /** Teal - speed, main data */
  primary: 'oklch(0.65 0.15 180)',
  /** Light teal - comparison data */
  secondary: 'oklch(0.75 0.13 180)',
  /** Green - throttle input */
  throttle: 'oklch(0.65 0.18 145)',
  /** Red/orange - brake input */
  brake: 'oklch(0.60 0.20 25)',
  /** Cyan - lateral G */
  lateralG: 'oklch(0.70 0.14 200)',
  /** Amber - longitudinal G */
  longitudinalG: 'oklch(0.65 0.15 60)',
};

export const plotLayout = {
  paper_bgcolor: 'transparent',
  plot_bgcolor: 'oklch(0.145 0 0 / 50%)',
  font: { color: 'oklch(0.985 0 0)', family: 'Inter, system-ui, sans-serif' },
  gridcolor: 'oklch(1 0 0 / 10%)',
  zerolinecolor: 'oklch(1 0 0 / 15%)',
};

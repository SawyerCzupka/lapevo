import { useMemo } from 'react';
import Plot from 'react-plotly.js';
import type { TelemetryFrame } from '@/lib/types';
import { chartColors, plotLayout } from '@/lib/chart-colors';

interface GForceChartProps {
  telemetry: TelemetryFrame[];
  height?: number;
}

export function GForceChart({ telemetry, height = 300 }: GForceChartProps) {
  const data = useMemo(() => {
    const distances = telemetry.map((f) => f.lap_distance);
    const lateralG = telemetry.map((f) => f.lateral_acceleration / 9.81); // Convert to G
    const longitudinalG = telemetry.map((f) => f.longitudinal_acceleration / 9.81);

    return [
      {
        x: distances,
        y: lateralG,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: 'Lateral G',
        line: {
          color: chartColors.lateralG,
          width: 2,
        },
        hovertemplate: '%{y:.2f}G<extra></extra>',
      },
      {
        x: distances,
        y: longitudinalG,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: 'Longitudinal G',
        line: {
          color: chartColors.longitudinalG,
          width: 2,
        },
        hovertemplate: '%{y:.2f}G<extra></extra>',
      },
    ];
  }, [telemetry]);

  const layout = useMemo(
    () => ({
      height,
      paper_bgcolor: plotLayout.paper_bgcolor,
      plot_bgcolor: plotLayout.plot_bgcolor,
      font: plotLayout.font,
      margin: { l: 60, r: 40, t: 40, b: 60 },
      xaxis: {
        title: { text: 'Distance (m)' },
        gridcolor: plotLayout.gridcolor,
        zerolinecolor: plotLayout.zerolinecolor,
      },
      yaxis: {
        title: { text: 'G-Force' },
        gridcolor: plotLayout.gridcolor,
        zerolinecolor: plotLayout.zerolinecolor,
      },
      hovermode: 'x unified' as const,
      legend: {
        x: 1,
        xanchor: 'right' as const,
        y: 1,
      },
    }),
    [height]
  );

  const config = useMemo(
    () => ({
      responsive: true,
      displayModeBar: true,
      displaylogo: false,
    }),
    []
  );

  return <Plot data={data as any} layout={layout as any} config={config} className="w-full" />;
}

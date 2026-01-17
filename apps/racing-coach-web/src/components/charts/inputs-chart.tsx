import { useMemo } from 'react';
import Plot from 'react-plotly.js';
import type { TelemetryFrame } from '@/lib/types';
import { chartColors, plotLayout } from '@/lib/chart-colors';

interface InputsChartProps {
  telemetry: TelemetryFrame[];
  height?: number;
}

export function InputsChart({ telemetry, height = 300 }: InputsChartProps) {
  const data = useMemo(() => {
    const distances = telemetry.map((f) => f.lap_distance);
    const throttle = telemetry.map((f) => f.throttle * 100);
    const brake = telemetry.map((f) => f.brake * 100);

    return [
      {
        x: distances,
        y: throttle,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: 'Throttle',
        line: {
          color: chartColors.throttle,
          width: 2,
        },
        hovertemplate: '%{y:.1f}%<extra></extra>',
      },
      {
        x: distances,
        y: brake,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: 'Brake',
        line: {
          color: chartColors.brake,
          width: 2,
        },
        hovertemplate: '%{y:.1f}%<extra></extra>',
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
        title: { text: 'Input (%)' },
        gridcolor: plotLayout.gridcolor,
        zerolinecolor: plotLayout.zerolinecolor,
        range: [0, 105],
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

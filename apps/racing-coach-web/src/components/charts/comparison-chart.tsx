import { useMemo } from 'react';
import Plot from 'react-plotly.js';
import type { TelemetryFrame } from '@/lib/types';
import { chartColors, plotLayout } from '@/lib/chart-colors';

interface ComparisonChartProps {
  lap1: TelemetryFrame[];
  lap2: TelemetryFrame[];
  lap1Label?: string;
  lap2Label?: string;
  metric: 'speed' | 'throttle' | 'brake';
  height?: number;
}

export function ComparisonChart({
  lap1,
  lap2,
  lap1Label = 'Lap 1',
  lap2Label = 'Lap 2',
  metric,
  height = 400,
}: ComparisonChartProps) {
  const data = useMemo(() => {
    const getValue = (frame: TelemetryFrame) => {
      switch (metric) {
        case 'speed':
          return frame.speed * 3.6; // m/s to km/h
        case 'throttle':
          return frame.throttle * 100;
        case 'brake':
          return frame.brake * 100;
      }
    };

    const lap1Distances = lap1.map((f) => f.lap_distance);
    const lap1Values = lap1.map(getValue);

    const lap2Distances = lap2.map((f) => f.lap_distance);
    const lap2Values = lap2.map(getValue);

    return [
      {
        x: lap1Distances,
        y: lap1Values,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: lap1Label,
        line: {
          color: chartColors.primary,
          width: 2,
        },
      },
      {
        x: lap2Distances,
        y: lap2Values,
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: lap2Label,
        line: {
          color: chartColors.secondary,
          width: 2,
          dash: 'dash',
        },
      },
    ];
  }, [lap1, lap2, lap1Label, lap2Label, metric]);

  const yAxisTitle = useMemo(() => {
    switch (metric) {
      case 'speed':
        return 'Speed (km/h)';
      case 'throttle':
      case 'brake':
        return `${metric.charAt(0).toUpperCase() + metric.slice(1)} (%)`;
    }
  }, [metric]);

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
        title: yAxisTitle,
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
    [height, yAxisTitle]
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

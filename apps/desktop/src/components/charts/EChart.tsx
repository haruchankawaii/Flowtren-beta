import {
    useEffect,
    useRef,
  } from "react";
  
  import * as echarts
    from "echarts";
  
  import type {
    EChartsOption,
  } from "echarts";
  
  type EChartProps = {
    option:
      EChartsOption;
  
    height?:
      number;
  };
  
  export function EChart(
    {
      option,
      height = 420,
    }: EChartProps,
  ) {
    const containerRef =
      useRef<HTMLDivElement | null>(
        null,
      );
  
    useEffect(
      () => {
        if (
          !containerRef.current
        ) {
          return;
        }
  
        const chart =
          echarts.init(
            containerRef.current,
          );
  
        chart.setOption(
          option,
        );
  
        const observer =
          new ResizeObserver(
            () => {
              chart.resize();
            },
          );
  
        observer.observe(
          containerRef.current,
        );
  
        return () => {
          observer.disconnect();
  
          chart.dispose();
        };
      },
      [
        option,
      ],
    );
  
    return (
      <div
        ref={
          containerRef
        }
        style={{
          width: "100%",
          height,
        }}
      />
    );
  }
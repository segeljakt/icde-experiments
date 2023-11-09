package org.example.metrics;

import org.apache.flink.api.common.accumulators.Histogram;
import org.apache.flink.api.common.functions.RichMapFunction;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.metrics.HistogramStatistics;
import org.apache.flink.metrics.Meter;
import org.apache.flink.metrics.MeterView;
import org.apache.flink.runtime.metrics.DescriptiveStatisticsHistogram;
import org.apache.flink.streaming.api.functions.ProcessFunction;
import org.apache.flink.util.Collector;

public class Throughput<T> extends RichMapFunction<T, T> {

    private Meter meter;
    private final String name;

    public Throughput(String name) {
        this.name = name;
    }

    @Override
    public void open(Configuration parameters) throws Exception {
        this.meter = getRuntimeContext()
                .getMetricGroup()
                .meter(name, new MeterView(1));
    }

    @Override
    public T map(T value) throws Exception {
        this.meter.markEvent(); // Marks an event for every record processed
        return value;
    }
}

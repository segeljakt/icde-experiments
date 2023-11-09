package org.example.metrics;

import org.apache.flink.metrics.Histogram;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.metrics.HistogramStatistics;
import org.apache.flink.runtime.metrics.DescriptiveStatisticsHistogram;
import org.apache.flink.streaming.api.functions.ProcessFunction;
import org.apache.flink.streaming.api.functions.sink.RichSinkFunction;
import org.apache.flink.util.Collector;

public class Latency<T> extends RichSinkFunction<T> {

    private Histogram latencyHistogram;
    private final String name;

    public Latency(String name) {
        this.name = name;
    }

    @Override
    public void open(Configuration parameters) {
        this.latencyHistogram = this.getRuntimeContext()
                .getMetricGroup()
                .histogram(name, new DescriptiveStatisticsHistogram(128));
    }

    @Override
    public void invoke(T event, Context context) {
        long currentTimestamp = System.currentTimeMillis();
        //long eventSourceTimestamp = event.getSourceTimestamp(); // Assuming the event carries its source timestamp
        //long endToEndLatency = currentTimestamp - eventSourceTimestamp;

        //latencyHistogram.add(endToEndLatency);
    }
}

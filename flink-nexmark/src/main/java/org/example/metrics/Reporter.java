package org.example.metrics;

import org.apache.flink.metrics.Metric;
import org.apache.flink.metrics.MetricConfig;
import org.apache.flink.metrics.MetricGroup;
import org.apache.flink.metrics.reporter.AbstractReporter;
import org.apache.flink.metrics.reporter.Scheduled;

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;

public class Reporter extends AbstractReporter implements Scheduled {

    private PrintWriter out;
    private final Map<String, Metric> metricsMap = new HashMap<>();

    @Override
    public void open(MetricConfig config) {
        String pathToFile = config.getString("path", "/path/to/your/metric/file.txt");
        try {
            out = new PrintWriter(new BufferedWriter(new FileWriter(pathToFile, true)));
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    @Override
    public void close() {
        out.close();
    }

    @Override
    public void notifyOfAddedMetric(Metric metric, String metricName, MetricGroup group) {
        String fullMetricName = group.toString() + "." + metricName;
        metricsMap.put(fullMetricName, metric);
    }

    @Override
    public void report() {
        for (Map.Entry<String, Metric> entry : metricsMap.entrySet()) {
            String metricName = entry.getKey();
            Metric metric = entry.getValue();
            if (metric instanceof org.apache.flink.metrics.Meter) {
                org.apache.flink.metrics.Meter meter = (org.apache.flink.metrics.Meter) metric;
                out.println(metricName + ": " + meter.getRate());
            }
            // Add other metric types if needed
        }
        out.flush();
    }

    @Override
    public String filterCharacters(String s) {
        return s;
    }
}


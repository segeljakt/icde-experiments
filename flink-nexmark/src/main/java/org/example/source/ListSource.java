package org.example.source;

import org.apache.flink.api.common.typeinfo.TypeHint;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;
import org.example.generator.NexmarkGenerator;

import java.io.Serializable;
import java.util.Iterator;
import java.util.List;

public class ListSource<T> extends RichSourceFunction<T> {

    /**
     * Flag to make the source cancelable.
     */
    private volatile boolean isRunning = true;
    private final List<T> list; // make the list transient
    private final TimestampExtractor<T> timestampExtractor;
    private final long frequency;
    private long watermark = 0;
    private long latestTimestamp = 0;
    private long i = 0;
    private final long slack;

    public ListSource(List<T> list, TimestampExtractor<T> timestampExtractor, long slack, long frequency) {
        this.list = list;
        this.timestampExtractor = timestampExtractor;
        this.frequency = frequency;
        this.slack = slack;
    }

    @Override
    public void run(SourceContext<T> ctx) throws Exception {
        Iterator<T> iter = list.iterator();
        while (isRunning && iter.hasNext()) {
            T event = iter.next();
            long timestamp = timestampExtractor.extractTimestamp(event);

            if (timestamp < watermark) {
                continue;
            }
            if (timestamp > latestTimestamp) {
                latestTimestamp = timestamp;
            }
            if (i % frequency == 0) {
                watermark = timestamp - slack;
                ctx.emitWatermark(new Watermark(watermark));
            }

            try {
                ctx.collectWithTimestamp(event, timestamp);
                i++;
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
    }

    @Override
    public void cancel() {
        isRunning = false;
    }

    @FunctionalInterface
    public interface TimestampExtractor<T> extends Serializable {
        long extractTimestamp(T data);
    }

}
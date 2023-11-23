package org.example.source;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.MappingIterator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvSchema;
import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;

import java.io.ByteArrayInputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.io.Serializable;

public class CsvByteArraySource<T> extends RichSourceFunction<T> {

    private volatile boolean isRunning = true;
    private final byte[] csvData;
    private final Class<T> targetType;
    private final TimestampExtractor<T> timestampExtractor;
    private final long frequency;
    private long watermark = 0;
    private long latestTimestamp = 0;
    private long i = 0;
    private final long slack;

    public CsvByteArraySource(byte[] csvData, Class<T> targetType, TimestampExtractor<T> timestampExtractor, long slack, long frequency) {
        this.csvData = csvData;
        this.targetType = targetType;
        this.timestampExtractor = timestampExtractor;
        this.frequency = frequency;
        this.slack = slack;
    }

    @Override
    public void run(SourceContext<T> ctx) throws Exception {
        CsvMapper mapper = new CsvMapper();
        CsvSchema schema = mapper.schemaFor(targetType).withHeader();
        Reader reader = new InputStreamReader(new ByteArrayInputStream(csvData));
        MappingIterator<T> iter = mapper.readerFor(targetType).with(schema).readValues(reader);

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
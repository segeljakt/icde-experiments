package org.example.source;

import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;

import java.io.RandomAccessFile;
import java.nio.MappedByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.MappingIterator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvSchema;

import java.io.Serializable;

public class CsvMmapSource<T> extends RichSourceFunction<T> {

    private volatile boolean isRunning = true;
    private final String filePath;
    private final Class<T> targetType;
    private final TimestampExtractor<T> timestampExtractor;
    private final long frequency;
    private final long slack;

    private final long numEvents;

    public CsvMmapSource(String filePath, Class<T> targetType, TimestampExtractor<T> timestampExtractor, long slack, long frequency, long numEvents) {
        this.filePath = filePath;
        this.targetType = targetType;
        this.timestampExtractor = timestampExtractor;
        this.frequency = frequency;
        this.slack = slack;
        this.numEvents = numEvents;
    }

    @Override
    public void run(SourceContext<T> ctx) throws Exception {
        try (RandomAccessFile randomAccessFile = new RandomAccessFile(filePath, "r");
             FileChannel fileChannel = randomAccessFile.getChannel()) {

            MappedByteBuffer buffer = fileChannel.map(FileChannel.MapMode.READ_ONLY, 0, fileChannel.size());
            byte[] byteArray = new byte[buffer.remaining()];
            buffer.get(byteArray);

            CsvMapper mapper = new CsvMapper();
            CsvSchema schema = mapper
                    .schemaFor(targetType)
                    .withHeader();

            MappingIterator<T> iter = mapper
                    .readerFor(targetType)
                    .with(schema)
                    .readValues(new String(byteArray, StandardCharsets.UTF_8));

            long watermark = 0;
            long latestTimestamp = 0;
            long i = 0;
            while (isRunning && iter.hasNext() && i < numEvents) {
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
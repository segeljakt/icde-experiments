package org.example.io;

import org.apache.flink.api.common.eventtime.SerializableTimestampAssigner;
import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.connector.file.src.FileSource;
import org.apache.flink.core.fs.Path;
import org.apache.flink.formats.csv.CsvReaderFormat;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.example.metrics.Throughput;

import java.time.Duration;

public class CsvFileSource {

    public static <T> DataStream<T> read(
            StreamExecutionEnvironment env,
            Class<T> c,
            String relativePath,
            SerializableTimestampAssigner<T> f) {
        return env
                .fromSource(
                        FileSource.forRecordStreamFormat(
                                CsvReaderFormat.forSchema(
                                        CsvMapper::new,
                                        csvMapper -> csvMapper
                                                .schemaFor(c)
                                                .withoutQuoteChar()
                                                .withColumnSeparator(','),
                                        TypeInformation.of(c)),
                                new Path(System.getProperty("user.dir") + "/" + relativePath)
                        ).build(),
                        WatermarkStrategy
                                .<T>forBoundedOutOfOrderness(Duration.ofSeconds(20))
                                .withTimestampAssigner(f),
                        relativePath
                );
    }
}

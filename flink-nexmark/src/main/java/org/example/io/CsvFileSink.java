package org.example.io;

import org.apache.flink.api.common.serialization.Encoder;
import org.apache.flink.connector.file.sink.FileSink;
import org.apache.flink.core.fs.Path;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.ObjectWriter;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.sink.filesystem.OutputFileConfig;
import org.example.metrics.Throughput;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

public class CsvFileSink {
    public static <T> void write(DataStream<T> stream, Class<T> c, String relativePath) {
        stream
                .sinkTo(FileSink
                        .forRowFormat(
                                new Path(System.getProperty("user.dir") + "/" + relativePath),
                                new CsvEncoder<T>(c)
                        )
                        .withOutputFileConfig(
                                OutputFileConfig
                                        .builder()
                                        .withPartSuffix(".csv")
                                        .build()
                        )
                        .build());
    }

    public static class CsvEncoder<T> implements Encoder<T> {
        private final ObjectWriter writer;

        public CsvEncoder(Class<T> typeClass) {
            CsvMapper csvMapper = new CsvMapper();
            this.writer = csvMapper.writer(
                    csvMapper
                            .schemaFor(typeClass)
                            .withoutHeader()
                            .withColumnSeparator(',')
            );
        }


        @Override
        public void encode(T element, OutputStream stream) throws IOException {
            stream.write(writer.writeValueAsString(element).getBytes(StandardCharsets.UTF_8));
        }
    }
}

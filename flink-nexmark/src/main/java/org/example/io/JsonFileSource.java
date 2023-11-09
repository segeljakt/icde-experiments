package org.example.io;

import org.apache.flink.api.common.serialization.DeserializationSchema;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.connector.file.src.FileSource;
import org.apache.flink.connector.file.src.reader.StreamFormat;
import org.apache.flink.core.fs.FSDataInputStream;
import org.apache.flink.core.fs.Path;
import org.apache.flink.formats.json.JsonDeserializationSchema;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.eventtime.SerializableTimestampAssigner;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.time.Duration;

public class JsonFileSource {


    public static <T> DataStream<T> read(
            StreamExecutionEnvironment env,
            Class<T> targetType,
            String relativePath,
            SerializableTimestampAssigner<T> timestampAssigner) {

        return env.fromSource(
                FileSource
                        .forRecordStreamFormat(
                                new JsonStreamFormat<T>(targetType),
                                new Path(System.getProperty("user.dir") + "/" + relativePath)
                        )
                        .build(),
                WatermarkStrategy
                        .<T>forBoundedOutOfOrderness(Duration.ofSeconds(20))
                        .withTimestampAssigner(timestampAssigner),
                relativePath
        );
    }

    public static class JsonStreamFormat<T> implements StreamFormat<T> {

        private final transient DeserializationSchema<T> deserializer;

        JsonStreamFormat(Class<T> targetType) {
            this.deserializer = new JsonDeserializationSchema<T>(targetType);
        }

        @Override
        public Reader<T> createReader(Configuration config, FSDataInputStream stream, long l, long l1) throws IOException {
            ObjectMapper mapper = new ObjectMapper();
            BufferedReader reader = new BufferedReader(new InputStreamReader(stream, StandardCharsets.UTF_8));
            return new Reader<T>() {
                @Override
                public T read() throws IOException {
                    String line = reader.readLine();
                    if (line != null) {
                        return mapper.readValue(line, getProducedType().getTypeClass());
                    } else {
                        return null;
                    }
                }

                @Override
                public void close() throws IOException {
                    reader.close();
                }
            };
        }

        @Override
        public Reader<T> restoreReader(Configuration configuration, FSDataInputStream fsDataInputStream, long l, long l1, long l2) throws IOException {
            return null;
        }

        @Override
        public boolean isSplittable() {
            return false;
        }

        @Override
        public TypeInformation<T> getProducedType() {
            return deserializer.getProducedType();
        }
    }
}

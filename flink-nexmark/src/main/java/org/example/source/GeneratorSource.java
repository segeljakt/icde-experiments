/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package org.example.source;

import org.apache.flink.configuration.Configuration;
import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;
import org.example.data.Event;
import org.example.generator.GeneratorConfig;
import org.example.generator.NexmarkGenerator;

import java.io.Serializable;
import java.util.Optional;

public class GeneratorSource<T> extends RichSourceFunction<T> {

    /**
     * Configuration for generator to use when reading synthetic events. May be split.
     */
    private final GeneratorConfig config;

    private final FilterMap<T> converter;
    private final TimestampExtractor<T> timestampExtractor;

    private long watermark = 0;
    private long latestTimestamp = 0;
    private long frequency = 0;
    private long i = 0;
    private final long slack;
    /**
     * Flag to make the source cancelable.
     */
    private volatile boolean isRunning = true;

    public GeneratorSource(GeneratorConfig config, FilterMap<T> converter, TimestampExtractor<T> timestampExtractor, long frequency, long slack) {
        this.config = config;
        this.converter = converter;
        this.timestampExtractor = timestampExtractor;
        this.frequency = frequency;
        this.slack = slack;
    }

    @Override
    public void run(SourceContext<T> ctx) throws Exception {
        NexmarkGenerator generator = new NexmarkGenerator(config);
        while (isRunning && generator.hasNext()) {
            long now = System.currentTimeMillis();
            NexmarkGenerator.NextEvent nextEvent = generator.nextEvent();
            Optional<T> optional = converter.filterMap(nextEvent.event);

            if (optional.isPresent()) {
                T event = optional.get();
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
    public interface FilterMap<T> extends Serializable {
        Optional<T> filterMap(Event data);
    }

    @FunctionalInterface
    public interface TimestampExtractor<T> extends Serializable {
        long extractTimestamp(T data);
    }
}
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
import org.apache.flink.streaming.api.functions.source.RichParallelSourceFunction;
import org.apache.flink.streaming.api.functions.source.RichSourceFunction;
import org.apache.flink.streaming.api.watermark.Watermark;
import org.example.data.Event;
import org.example.generator.GeneratorConfig;
import org.example.generator.NexmarkGenerator;

public class NexmarkSourceFunction extends RichSourceFunction<Event> {

    /**
     * Configuration for generator to use when reading synthetic events. May be split.
     */
    private final GeneratorConfig config;

    private transient NexmarkGenerator generator;

    /**
     * Flag to make the source cancelable.
     */
    private volatile boolean isRunning = true;

    public NexmarkSourceFunction(GeneratorConfig config) {
        this.config = config;
    }

    @Override
    public void open(Configuration parameters) throws Exception {
        super.open(parameters);
        this.generator = new NexmarkGenerator(getSubGeneratorConfig());
    }

    private GeneratorConfig getSubGeneratorConfig() {
        int parallelism = getRuntimeContext().getNumberOfParallelSubtasks();
        int taskId = getRuntimeContext().getIndexOfThisSubtask();
        return config.split(parallelism).get(taskId);
    }

    @Override
    public void run(SourceContext<Event> ctx) throws Exception {
        while (isRunning && generator.hasNext()) {
            long now = System.currentTimeMillis();
            NexmarkGenerator.NextEvent nextEvent = generator.nextEvent();

            // If the event happens in the future, wait for it
            //if (nextEvent.wallclockTimestamp > now) {
            //    Thread.sleep(nextEvent.wallclockTimestamp - now);
            //}

            try {
                ctx.collect(nextEvent.event);
            } catch (Exception e) {
                e.printStackTrace();
            }
        }
    }

    @Override
    public void cancel() {
        isRunning = false;
    }


    @Override
    public void close() throws Exception {
        super.close();
    }
}
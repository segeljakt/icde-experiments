package org.example.io;

import org.apache.flink.streaming.api.functions.sink.RichSinkFunction;

public class DataSink<T> extends RichSinkFunction<T> {

        /**
        * Flag to make the source cancelable.
        */
        private volatile boolean isRunning = true;

        private volatile long n = 0;

        public DataSink() {
        }

        @Override
        public void invoke(T value, Context context) throws Exception {
            n++;
        }

        @Override
        public void finish() throws Exception {
            System.out.println("n: " + n);
            super.finish();
        }

}

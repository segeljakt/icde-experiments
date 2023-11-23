package org.example.queries;

import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;

import java.util.HashSet;
import java.util.List;

public class UndefinedBehaviour {
    public static void run(StreamExecutionEnvironment env) {
        // Use only one task slot to force all operations to share the same slot
        env.setParallelism(2);

        // Source data
        DataStream<String> text = env.fromElements(
                "hello", "flink",
                "hello", "world",
                "hello", "world",
                "flink", "world"
        );

        // Shared HashSet across operators within the same task slot
        final HashSet<String> uniqueWords = new HashSet<>();

        // Define a simple FlatMap operation
        DataStream<String> words = text
                .filter(x -> {
                    if (uniqueWords.contains(x)) {
                        return false;
                    } else {
                        uniqueWords.add(x);
                        return true;
                    }
                });

        // Print the result
        try {
            List<String> w = words.executeAndCollect(10);
            System.out.println(w.toString());
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }
}

package org.example;

import org.apache.flink.api.common.JobExecutionResult;
import org.apache.flink.api.common.typeinfo.TypeHint;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.example.data.Auction;
import org.example.data.Person;
import org.example.queries.*;
import org.example.io.*;

public class Main {
    public static void main(String[] args) throws Exception {

        if (args.length == 0) {
            System.out.println("No query specified");
            return;
        }
        String dir = args[0];
        String query = args[1];

        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        env.setParallelism(1);
        env.setMaxParallelism(1);

        switch (query) {
            case "q3": {
                Query3.q3(persons(env, dir), auctions(env, dir)).addSink(new DataSink<>());
                break;
            }
            case "q3-opt": {
                Query3.q3Opt(persons(env, dir), auctions(env, dir)).addSink(new DataSink<>());
                break;
            }
            default:
                System.out.println("Unknown query: " + query);
        }

        JobExecutionResult result = env.execute();
        System.err.println(result.getNetRuntime());
    }

    public static <T> DataStream<T> read(StreamExecutionEnvironment env, String path, Class<T> c, DataSource.TimestampExtractor<T> t, TypeInformation<T> ti) {
        return env
                .addSource(new DataSource<T>(
                        () -> new CsvMmapIterator<T>(path, c),
                        t,
                        1000,
                        100
                ))
                .returns(ti);
    }

    public static DataStream<Auction> auctions(StreamExecutionEnvironment env, String dir) {
        String path = dir + "/auctions.csv";
        return read(env, path, Auction.class, x -> x.dateTime, TypeInformation.of(new TypeHint<Auction>() {
        }));
    }

    public static DataStream<Person> persons(StreamExecutionEnvironment env, String dir) {
        String path = dir + "/persons.csv";
        return read(env, path, Person.class, x -> x.dateTime, TypeInformation.of(new TypeHint<Person>() {
        }));
    }
}

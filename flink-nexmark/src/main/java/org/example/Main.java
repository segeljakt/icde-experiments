package org.example;

import org.apache.flink.configuration.ReadableConfig;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;
import org.example.io.CsvFileSink;
import org.example.io.CsvFileSource;
import org.example.queries.Query1;
import org.example.queries.Query2;
import org.example.queries.Query3;
import org.example.queries.Query4;

public class Main {
    public static void main(String[] args) throws Exception {
        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
        env.setParallelism(1);
        env.setMaxParallelism(1);
        //env.disableOperatorChaining();

        if (args.length == 0) {
            System.out.println("No query specified");
            return;
        }

        for (String arg : args) {
            switch (arg) {
                case "query1":
                    CsvFileSink.write(Query1.naive(bids(env)), Query1.Output.class, "data/query1.csv");
                    break;
                case "query2":
                    CsvFileSink.write(Query2.naive(bids(env)), Query2.Output.class, "data/query2.csv");
                    break;
                case "query3":
                    CsvFileSink.write(Query3.naive(persons(env), auctions(env)), Query3.Output.class, "data/query3.csv");
                    break;
                case "query3-opt":
                    CsvFileSink.write(Query3.optimised(persons(env), auctions(env)), Query3.Output.class, "data/query3-opt.csv");
                    break;
                case "query4":
                    CsvFileSink.write(Query4.naive(bids(env), auctions(env)), Query4.Output.class, "data/query4.csv");
                    break;
                default:
                    System.out.println("Unknown query: " + arg);
            }
        }

        env.execute("nexmark");
    }

    static DataStream<Bid> bids(StreamExecutionEnvironment env) {
        return CsvFileSource.read(env, Bid.class, "data/bids.csv", (bid, t) -> bid.dateTime);
    }

    static DataStream<Auction> auctions(StreamExecutionEnvironment env) {
        return CsvFileSource.read(env, Auction.class, "data/auctions.csv", (auction, t) -> auction.dateTime);
    }

    static DataStream<Person> persons(StreamExecutionEnvironment env) {
        return CsvFileSource.read(env, Person.class, "data/persons.csv", (person, t) -> person.dateTime);
    }
}

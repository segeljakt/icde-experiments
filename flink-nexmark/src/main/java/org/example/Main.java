package org.example;

import org.apache.flink.api.common.ExecutionConfig;
import org.apache.flink.api.common.JobExecutionResult;
import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.functions.FlatMapFunction;
import org.apache.flink.api.common.typeinfo.TypeHint;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.configuration.ReadableConfig;
import org.apache.flink.contrib.streaming.state.EmbeddedRocksDBStateBackend;
import org.apache.flink.runtime.state.hashmap.HashMapStateBackend;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.databind.MappingIterator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvMapper;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.dataformat.csv.CsvSchema;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.LocalStreamEnvironment;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.util.Collector;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Event;
import org.example.data.Person;
import org.example.generator.GeneratorConfig;
import org.example.generator.NexmarkGenerator;
import org.example.io.CsvFileSink;
import org.example.io.CsvFileSource;
import org.example.io.CsvReader;
import org.example.queries.*;
import org.example.source.*;

import javax.xml.crypto.KeySelector;
import java.io.File;
import java.io.IOException;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.time.Duration;
import java.util.*;

import static org.example.generator.GeneratorConfig.EventFilter.BID;
import static org.example.generator.GeneratorConfig.EventFilter.AUCTION;
import static org.example.generator.GeneratorConfig.EventFilter.PERSON;

public class Main {
    public static void main(String[] args) throws Exception {

        if (args.length == 0) {
            System.out.println("No query specified");
            return;
        }
        String query = args[0];
        long n = Long.parseLong(args[1]);
        String dir = args[2];

        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
        //LocalStreamEnvironment env = StreamExecutionEnvironment.createLocalEnvironment();

        env.setParallelism(1);
        env.setMaxParallelism(1);
        //env.setStateBackend(new EmbeddedRocksDBStateBackend());

        //List<Bid> bids = csvToList(Bid.class, (int) (numEvents * 0.92), dir + "/bids.csv");
        //List<Auction> auctions = csvToList(Auction.class, (int) (numEvents * 0.06), dir + "/auctions.csv");
        //List<Person> persons = csvToList(Person.class, (int) (numEvents * 0.02), dir + "/persons.csv");

        //System.out.println("Bids: " + bids.size());
        //System.out.println("Auctions: " + auctions.size());
        //System.out.println("Persons: " + persons.size());

        //byte[] bids = Files.readAllBytes(Paths.get(dir + "/bids.csv"));
        //byte[] auctions = Files.readAllBytes(Paths.get(dir + "/auctions.csv"));
        //byte[] persons = Files.readAllBytes(Paths.get(dir + "/persons.csv"));

        //System.out.println("Bids: " + bids.length);
        //System.out.println("Auctions: " + auctions.length);
        //System.out.println("Persons: " + persons.length);

        //NexmarkConfiguration nexmarkConfiguration = new NexmarkConfiguration();
        ////nexmarkConfiguration.firstEventRate = 1_000_000;
        ////nexmarkConfiguration.outOfOrderGroupSize = 1000;
        //GeneratorConfig config = new GeneratorConfig(nexmarkConfiguration, System.currentTimeMillis(), 1, numEvents, 1);

        //long bn = (long) (numEvents * 0.92);
        //long an = (long) (numEvents * 0.06);
        //long pn = (long) (numEvents * 0.02);

        //GeneratorConfig b = config.copy();
        //GeneratorConfig a = config.copy();
        //GeneratorConfig p = config.copy();

        //b.maxEvents = bn;
        //a.maxEvents = an;
        //p.maxEvents = pn;

        //b.eventFilter = BID;
        //a.eventFilter = AUCTION;
        //p.eventFilter = PERSON;

        //DataStream<Event> stream = env.addSource(new NexmarkSourceFunction(config));

        switch (query) {
            case "query1":
                Query1.naive(bidsMmap(env, dir, n)).addSink(new GeneratorSink<>());
                break;
            case "query2":
                Query2.naive(bidsMmap(env, dir, n)).addSink(new GeneratorSink<>());
                break;
            case "query3":
                Query3.naive(
                        personsMmap(env, dir, n),
                        auctionsMmap(env, dir, n)
                ).addSink(new GeneratorSink<>());
                break;
            case "query3-opt":
                Query3.optimised(
                        personsMmap(env, dir, n),
                        auctionsMmap(env, dir, n)
                ).addSink(new GeneratorSink<>());
                break;
            case "query3-opt2":
                Query3.optimised2(
                        personsMmap(env, dir, n),
                        auctionsMmap(env, dir, n)
                ).addSink(new GeneratorSink<>());
                ;
                break;
            case "query4":
                Query4.naive(
                        bidsMmap(env, dir, n),
                        auctionsMmap(env, dir, n)
                ).addSink(new GeneratorSink<>());
                break;
            case "bug":
                UndefinedBehaviour.run(env);
                break;
            default:
                System.out.println("Unknown query: " + query);
        }

        JobExecutionResult result = env.execute();
        System.err.println(result.getNetRuntime());
    }

    static DataStream<Bid> bidsGenerator(GeneratorConfig config, StreamExecutionEnvironment env) {
        return env.addSource(
                        new GeneratorSource<Bid>(config,
                                x -> x.bid != null ? Optional.of(x.bid) : Optional.empty(),
                                x -> x.dateTime,
                                1000,
                                100
                        )
                )
                .returns(TypeInformation.of(new TypeHint<Bid>() {
                }))
                ;
    }

    static DataStream<Auction> auctionsGenerator(GeneratorConfig config, StreamExecutionEnvironment env) {
        return env.addSource(
                        new GeneratorSource<Auction>(config,
                                x -> x.newAuction != null ? Optional.of(x.newAuction) : Optional.empty(),
                                x -> x.dateTime,
                                1000,
                                100
                        )
                )
                .returns(TypeInformation.of(new TypeHint<Auction>() {
                }));
    }

    static DataStream<Person> personsGenerator(GeneratorConfig config, StreamExecutionEnvironment env) {
        return env.addSource(
                        new GeneratorSource<Person>(config,
                                x -> x.newPerson != null ? Optional.of(x.newPerson) : Optional.empty(),
                                x -> x.dateTime,
                                1000,
                                100
                        )
                )
                .returns(TypeInformation.of(new TypeHint<Person>() {
                }));
    }

    static DataStream<Bid> bids(DataStream<Event> stream) {
        return stream
                .filter(event -> event.bid != null)
                .map(event -> event.bid)
                .assignTimestampsAndWatermarks(
                        WatermarkStrategy
                                .<Bid>forBoundedOutOfOrderness(Duration.ofMillis(100))
                                .withTimestampAssigner((bid, t) -> bid.dateTime));
    }

    static DataStream<Auction> auctions(DataStream<Event> stream) {
        return stream
                .filter(event -> event.newAuction != null)
                .map(event -> event.newAuction)
                .assignTimestampsAndWatermarks(
                        WatermarkStrategy
                                .<Auction>forBoundedOutOfOrderness(Duration.ofMillis(100))
                                .withTimestampAssigner((auction, t) -> auction.dateTime));
    }

    static DataStream<Person> persons(DataStream<Event> stream) {
        return stream
                .filter(event -> event.newPerson != null)
                .map(event -> event.newPerson)
                .assignTimestampsAndWatermarks(
                        WatermarkStrategy
                                .<Person>forBoundedOutOfOrderness(Duration.ofMillis(100))
                                .withTimestampAssigner((person, t) -> person.dateTime));
    }

    public static <T> List<T> csvToList(Class<T> c, int n, String filePath) {
        try {
            List<T> resultList = new ArrayList<>();
            CsvMapper mapper = new CsvMapper();
            CsvSchema.Builder schemaBuilder = CsvSchema.builder();
            for (Field field : c.getDeclaredFields()) {
                schemaBuilder.addColumn(field.getName());
            }
            CsvSchema schema = schemaBuilder.build().withoutHeader();

            File csvFile = new File(filePath);
            MappingIterator<T> iterator = mapper
                    .readerFor(c)
                    .with(schema)
                    .readValues(csvFile);

            int lineCount = 0;
            while (iterator.hasNext() && lineCount < n) {
                resultList.add(iterator.next());
                lineCount++;
            }

            return resultList;
        } catch (IOException e) {
            e.printStackTrace();
            return Collections.emptyList();
        }
    }

    public static DataStream<Bid> bidsMmap(StreamExecutionEnvironment env, String dir, long n) {
        long nb = (long) (n * 0.92);
        return env
                .addSource(new CsvMmapSource<Bid>(dir + "/bids.csv", Bid.class, x -> x.dateTime, 1000, 100, nb))
                .returns(TypeInformation.of(new TypeHint<Bid>() {
                }));
    }

    public static DataStream<Auction> auctionsMmap(StreamExecutionEnvironment env, String dir, long n) {
        long na = (long) (n * 0.06);
        return env
                .addSource(new CsvMmapSource<Auction>(dir + "/auctions.csv", Auction.class, x -> x.dateTime, 1000, 100, na))
                .returns(TypeInformation.of(new TypeHint<Auction>() {
                }));
    }

    public static DataStream<Person> personsMmap(StreamExecutionEnvironment env, String dir, long n) {
        long np = (long) (n * 0.02);
        return env
                .addSource(new CsvMmapSource<Person>(dir + "/persons.csv", Person.class, x -> x.dateTime, 1000, 100, np))
                .returns(TypeInformation.of(new TypeHint<Person>() {
                }));
    }


    public static DataStream<Bid> bidsStreamFromBytes(StreamExecutionEnvironment env, byte[] bytes) {
        return env
                .addSource(new CsvByteArraySource<Bid>(bytes, Bid.class, x -> x.dateTime, 1000, 100))
                .returns(TypeInformation.of(new TypeHint<Bid>() {
                }));
    }

    public static DataStream<Auction> auctionsStreamFromBytes(StreamExecutionEnvironment env, byte[] bytes) {
        return env
                .addSource(new CsvByteArraySource<Auction>(bytes, Auction.class, x -> x.dateTime, 1000, 100))
                .returns(TypeInformation.of(new TypeHint<Auction>() {
                }));
    }

    public static DataStream<Person> personsStreamFromBytes(StreamExecutionEnvironment env, byte[] bytes) {
        return env
                .addSource(new CsvByteArraySource<Person>(bytes, Person.class, x -> x.dateTime, 1000, 100))
                .returns(TypeInformation.of(new TypeHint<Person>() {
                }));
    }

    public static DataStream<Bid> bidsStream(StreamExecutionEnvironment env, List<Bid> list) {
        return env.fromCollection(list);
        //return env
        //        .addSource(new ListSource<Bid>(list, x -> x.dateTime, 1000, 100))
        //        .returns(TypeInformation.of(new TypeHint<Bid>() {
        //        }));
    }

    public static DataStream<Auction> auctionsStream(StreamExecutionEnvironment env, List<Auction> list) {
        return env.fromCollection(list);
        //return env
        //        .addSource(new ListSource<Auction>(list, x -> x.dateTime, 1000, 100))
        //        .returns(TypeInformation.of(new TypeHint<Auction>() {
        //        }));
    }

    public static DataStream<Person> personsStream(StreamExecutionEnvironment env, List<Person> list) {
        return env.fromCollection(list);
        //return env
        //        .addSource(new ListSource<Person>(list, x -> x.dateTime, 1000, 100))
        //        .returns(TypeInformation.of(new TypeHint<Person>() {
        //        }));
    }

}

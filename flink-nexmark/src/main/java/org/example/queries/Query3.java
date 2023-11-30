package org.example.queries;

import org.apache.flink.api.common.functions.JoinFunction;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.example.data.Auction;
import org.example.data.Person;
import org.example.operators.FilterMap;

import java.util.Optional;

// Who is selling in OR, ID or CA in category 10, and for what auction ids?
public class Query3 {
    public static DataStream<Output> q3(DataStream<Person> persons, DataStream<Auction> auctions) {
        return auctions.join(persons)
                .where(auction -> auction.seller)
                .equalTo(person -> person.id)
                .window(TumblingEventTimeWindows.of(Time.seconds(10)))
                .apply(new JoinFunction<Auction, Person, Tuple2<Auction, Person>>() {
                    @Override
                    public Tuple2<Auction, Person> join(Auction auction, Person person) {
                        return new Tuple2<Auction, Person>(auction, person);
                    }
                })
                .filter(tuple -> (tuple.f1.state.equals("OR") || tuple.f1.state.equals("ID") || tuple.f1.state.equals("CA")) && tuple.f0.category == 10)
                .map(tuple -> new Output(tuple.f1.name, tuple.f1.city, tuple.f1.state, tuple.f0.id))
                .returns(Output.class);
    }

    // Optimisations:
    // - Data pruning
    // - Predicate pushdown
    // - Fuse filter and map
    public static DataStream<Output> q3Opt(DataStream<Person> persons, DataStream<Auction> auctions) {
        DataStream<PrunedPerson> persons2 = persons
                .process(new FilterMap<Person, PrunedPerson>(p -> {
                    if (p.state.equals("OR") || p.state.equals("ID") || p.state.equals("CA")) {
                        return Optional.of(new PrunedPerson(p.id, p.name, p.city, p.state));
                    } else {
                        return Optional.empty();
                    }
                }))
                .returns(PrunedPerson.class);
        DataStream<PrunedAuction> auctions2 = auctions
                .process(new FilterMap<Auction, PrunedAuction>(a -> {
                    if (a.category == 10) {
                        return Optional.of(new PrunedAuction(a.seller, a.id));
                    } else {
                        return Optional.empty();
                    }
                }))
                .returns(PrunedAuction.class);
        return auctions2
                .map(a -> new PrunedAuction(a.seller, a.id))
                .join(persons2)
                .where(a -> a.seller)
                .equalTo(p -> p.id)
                .window(TumblingEventTimeWindows.of(Time.seconds(10)))
                .allowedLateness(Time.milliseconds(0))
                .apply((a, p) -> new Output(p.name, p.city, p.state, a.id));
    }

    public static class PrunedPerson {
        public long id;
        public String name;
        public String city;
        public String state;

        public PrunedPerson(long id, String name, String city, String state) {
            this.id = id;
            this.name = name;
            this.city = city;
            this.state = state;
        }

        public PrunedPerson() {
        }
    }

    public static class PrunedAuction {
        public long seller;
        public long id;

        public PrunedAuction(long seller, long id) {
            this.seller = seller;
            this.id = id;
        }

        public PrunedAuction() {
        }
    }

    @JsonPropertyOrder({"name", "city", "state", "id"})
    public static class Output {
        public String name;
        public String city;
        public String state;
        public long id;

        public Output(String name, String city, String state, long id) {
            this.name = name;
            this.city = city;
            this.state = state;
            this.id = id;
        }

        public Output() {
        }
    }
}

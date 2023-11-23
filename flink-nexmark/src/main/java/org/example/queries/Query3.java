package org.example.queries;

import org.apache.flink.api.common.functions.JoinFunction;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.api.java.tuple.Tuple4;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;

// Who is selling in OR, ID or CA in category 10, and for what auction ids?
public class Query3 {
    public static DataStream<Output> naive(DataStream<Person> persons, DataStream<Auction> auctions) {
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
    // - Filter auctions by category before joining
    // - Filter people by state before joining
    // - Fuse map with apply
    public static DataStream<Output> optimised(DataStream<Person> persons, DataStream<Auction> auctions) {
        DataStream<Person> filtered = persons
                .filter(auction -> auction.state.equals("OR") || auction.state.equals("ID") || auction.state.equals("CA"));
        return auctions
                .filter(a -> a.category == 10)
                .join(filtered)
                .where(auction -> auction.seller)
                .equalTo(person -> person.id)
                .window(TumblingEventTimeWindows.of(Time.seconds(10)))
                .allowedLateness(Time.milliseconds(0))
                .apply((auction, person) -> new Output(person.name, person.city, person.state, auction.id));
    }

    public static DataStream<Output> optimised2(DataStream<Person> persons, DataStream<Auction> auctions) {
        DataStream<PartialPerson> filtered = persons
                .filter(auction -> auction.state.equals("OR") || auction.state.equals("ID") || auction.state.equals("CA"))
                .map(person -> new PartialPerson(person.id, person.name, person.city, person.state));
        return auctions
                .filter(a -> a.category == 10)
                .map(auction -> new PartialAuction(auction.seller, auction.id))
                .join(filtered)
                .where(auction -> auction.seller)
                .equalTo(person -> person.id)
                .window(TumblingEventTimeWindows.of(Time.seconds(10)))
                .allowedLateness(Time.milliseconds(0))
                .apply((auction, person) -> new Output(person.name, person.city, person.state, auction.id));
    }

    public static class PartialPerson {
        public long id;
        public String name;
        public String city;
        public String state;

        public PartialPerson(long id, String name, String city, String state) {
            this.id = id;
            this.name = name;
            this.city = city;
            this.state = state;
        }

        public PartialPerson() {}
    }

    public static class PartialAuction {
        public long seller;
        public long id;

        public PartialAuction(long seller, long id) {
            this.seller = seller;
            this.id = id;
        }

        public PartialAuction() {}
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

        public Output() {}
    }
}

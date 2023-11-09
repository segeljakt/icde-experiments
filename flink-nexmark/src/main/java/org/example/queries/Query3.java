package org.example.queries;

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
                .window(TumblingEventTimeWindows.of(Time.seconds(60)))
                .apply(Tuple2::new)
                .filter(tuple -> (tuple.f1.state.equals("OR") || tuple.f1.state.equals("ID") || tuple.f1.state.equals("CA")) && tuple.f0.category == 10)
                .map(tuple -> new Output(tuple.f1.name, tuple.f1.city, tuple.f1.state, tuple.f0.id))
                .returns(Output.class);
    }

    // Optimisations:
    // - Filter auctions by category before joining
    // - Filter people by state before joining
    // - Fuse map with apply
    public static DataStream<Output> optimised(DataStream<Person> persons, DataStream<Auction> auctions) {
        DataStream<Person> filtered = persons.filter(auction -> auction.state.equals("OR") || auction.state.equals("ID") || auction.state.equals("CA"));
        return auctions
                .filter(a -> a.category == 10)
                .join(filtered)
                .where(auction -> auction.seller)
                .equalTo(person -> person.id)
                .window(TumblingEventTimeWindows.of(Time.seconds(60)))
                .apply((a, b) -> new Output(b.name, b.city, b.state, a.id));
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
    }
}

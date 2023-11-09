package org.example.queries;

import org.apache.flink.api.common.accumulators.AverageAccumulator;
import org.apache.flink.api.common.functions.AggregateFunction;
import org.apache.flink.api.java.tuple.Tuple1;
import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.api.java.tuple.Tuple3;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.windowing.assigners.TumblingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;

// Select the average of the winning bid prices for all auctions in each category.

public class Query4 {
    public static DataStream<Output>
    naive(DataStream<Bid> bids, DataStream<Auction> auctions) {
        return auctions.join(bids).where(auction -> auction.id)
                .equalTo(bid -> bid.auction)
                .window(TumblingEventTimeWindows.of(Time.minutes(1)))
                .apply(Tuple2::new)
                .filter(tuple -> tuple.f1.dateTime < tuple.f0.dateTime)
                .map(tuple -> new Tuple3<>(tuple.f0.id, tuple.f0.category, tuple.f1.price))
                // keyby auction id
                .keyBy(tuple -> tuple.f0)
                .window(TumblingEventTimeWindows.of(Time.minutes(1)))
                // Max bid price per auction (winning bid price)
                .maxBy(2)
                .map(tuple -> new Tuple2<>(tuple.f1, tuple.f2))
                // keyby category id
                .keyBy(tuple -> tuple.f0)
                .window(TumblingEventTimeWindows.of(Time.minutes(1)))
                // Average price of winning bid per category
                .aggregate(new AverageAggregator())
                .returns(Output.class);
    }

    // Helper function to calculate average
    public static class AverageAggregator implements AggregateFunction<Tuple2<Long, Long>, Tuple2<Long, Long>, Output> {
        @Override
        public Tuple2<Long, Long> createAccumulator() {
            return new Tuple2<>(0L, 0L);
        }

        @Override
        public Tuple2<Long, Long> add(Tuple2<Long, Long> value, Tuple2<Long, Long> accumulator) {
            return new Tuple2<>(accumulator.f0 + value.f1, accumulator.f1 + 1);
        }

        @Override
        public Output getResult(Tuple2<Long, Long> accumulator) {
            return new Output(accumulator.f0 / accumulator.f1);
        }

        @Override
        public Tuple2<Long, Long> merge(Tuple2<Long, Long> a, Tuple2<Long, Long> b) {
            return new Tuple2<>(a.f0 + b.f0, a.f1 + b.f1);
        }
    }

    @JsonPropertyOrder({"price"})
    public static class Output {
        public long price;

        public Output(long price) {
            this.price = price;
        }
    }
}

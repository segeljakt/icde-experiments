package org.example.queries;

import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.functions.windowing.WindowFunction;
import org.apache.flink.streaming.api.windowing.assigners.SlidingEventTimeWindows;
import org.apache.flink.streaming.api.windowing.time.Time;
import org.apache.flink.streaming.api.windowing.windows.TimeWindow;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;

// Which auctions have seen the most bids in the last period?
public class Query5 {
    public static DataStream<Long>
    naive(DataStream<Bid> bids, DataStream<Person> people, DataStream<Auction> auctions) {
        DataStream<Tuple2<Long, Long>> bidsPerAuction = bids
                .keyBy(bid -> bid.auction)
                .window(SlidingEventTimeWindows.of(Time.minutes(60), Time.minutes(1)))
                .apply((key, window, input, out) -> {
                    long count = 0;
                    for (Bid b : input) {
                        count++;
                    }
                    out.collect(new Tuple2<>(key, count));
                });

        DataStream<Long> maxBidCount = bidsPerAuction
                .windowAll(SlidingEventTimeWindows.of(Time.minutes(60), Time.minutes(1)))
                .maxBy(1)
                .map(tuple -> tuple.f1);

        // TODO
        //  BroadcastStream<Long> a = maxBidCount.broadcast();
        //  BroadcastConnectedStream<Tuple2<Long,Long>, Long> x = bidsPerAuction
        //          .connect();
        //          .process(new BroadcastProcessFunction<Tuple2<Long, Long>, Long, Long>());
        return null;
    }

}

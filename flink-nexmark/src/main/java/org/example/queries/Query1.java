package org.example.queries;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.example.data.Bid;
import org.example.source.GeneratorSink;

// Convert each bid value from dollars to euros
public class Query1 {
    public static DataStream<Output> naive(DataStream<Bid> bids) {
        return bids
                .map(b -> new Output(b.auction, (long) Math.floor(b.price * 0.85), b.bidder, b.dateTime))
                .returns(Output.class);
    }

    @JsonPropertyOrder({"auction", "price", "bidder", "dateTime"})
    public static class Output {
        public long auction;
        public long price;
        public long bidder;
        public long dateTime;

        public Output(long auction, long price, long bidder, long dateTime) {
            this.auction = auction;
            this.price = price;
            this.bidder = bidder;
            this.dateTime = dateTime;
        }

        public Output() {}
    }
}

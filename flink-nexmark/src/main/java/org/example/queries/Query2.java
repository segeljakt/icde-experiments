package org.example.queries;

import org.apache.flink.api.java.tuple.Tuple2;
import org.apache.flink.api.java.tuple.Tuple4;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.example.data.Auction;
import org.example.data.Bid;
import org.example.data.Person;


// This query selects real-time bids for specific auctions. The auctions are filtered by their IDs (1007, 1020, 2001, 2019, 2087).
public class Query2 {
    public static DataStream<Output> naive(DataStream<Bid> bids) {
        return bids
                .filter(bid -> bid.auction == 1007 || bid.auction == 1020 || bid.auction == 2001 || bid.auction == 2019 || bid.auction == 2087)
                .map(b -> new Output(b.auction, b.bidder))
                .returns(Output.class);
    }

    @JsonPropertyOrder({"auction", "bidder"})
    public static class Output {
        public long auction;
        public long bidder;

        public Output(long auction, long bidder) {
            this.auction = auction;
            this.bidder = bidder;
        }

        public Output() {}
    }
}

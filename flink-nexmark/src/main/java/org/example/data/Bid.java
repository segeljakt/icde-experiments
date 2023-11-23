package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonCreator;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonProperty;
import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;

import java.io.Serializable;
import java.time.Instant;


@JsonPropertyOrder({"auction", "bidder", "price", "channel", "url", "dateTime", "extra"})
public class Bid implements Serializable {
    public long auction;
    public long bidder;
    public long price;
    public String channel;
    public String url;
    public long dateTime;
    public String extra;

    @JsonCreator
    public Bid(@JsonProperty("auction") long auction,
               @JsonProperty("bidder") long bidder,
               @JsonProperty("price") long price,
               @JsonProperty("channel") String channel,
               @JsonProperty("url") String url,
               @JsonProperty("dateTime") long dateTime,
               @JsonProperty("extra") String extra) {
        this.auction = auction;
        this.bidder = bidder;
        this.price = price;
        this.channel = channel;
        this.url = url;
        this.dateTime = dateTime;
        this.extra = extra;
    }
    //public Bid(long auction, long bidder, long price, String channel, String url, long dateTime, String extra) {
    //    this.auction = auction;
    //    this.bidder = bidder;
    //    this.price = price;
    //    this.channel = channel;
    //    this.url = url;
    //    this.dateTime = dateTime;
    //    this.extra = extra;
    //}

    public Bid() {}
}

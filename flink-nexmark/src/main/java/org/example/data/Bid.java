package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;


@JsonPropertyOrder({"auction", "bidder", "price", "channel", "url", "dateTime", "extra"})
public class Bid {
    public long auction;
    public long bidder;
    public long price;
    public String channel;
    public String url;
    public long dateTime;
    public String extra;
}

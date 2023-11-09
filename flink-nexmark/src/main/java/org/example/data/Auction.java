package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;

@JsonPropertyOrder({"id", "itemName", "description", "initialBid", "reserve", "dateTime", "expires", "seller", "category", "extra"})
public class Auction {
    public long id;
    public String itemName;
    public String description;
    public long initialBid;
    public long reserve;
    public long dateTime;
    public long expires;
    public long seller;
    public long category;
    public String extra;
}

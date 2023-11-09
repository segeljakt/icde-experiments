package org.example.data;

import org.apache.flink.shaded.jackson2.com.fasterxml.jackson.annotation.JsonPropertyOrder;


@JsonPropertyOrder({"id", "name", "emailAddress", "creditCard", "city", "state", "dateTime", "extra"})
public class Person {
    public long id;
    public String name;
    public String emailAddress;
    public String creditCard;
    public String city;
    public String state;
    public long dateTime;
    public String extra;
}

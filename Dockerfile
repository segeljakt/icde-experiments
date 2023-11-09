FROM ubuntu:20.04

# Set the noninteractive timezone (prevents configuration prompts)
ARG DEBIAN_FRONTEND=noninteractive
ENV TZ=Europe/London

# Set a working directory
WORKDIR /experiment

# Install apt packages
RUN apt-get update && apt-get install -y \
  curl wget software-properties-common default-jdk python3 python3-pip

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Apache Flink
RUN wget https://downloads.apache.org/flink/flink-1.18.0/flink-1.18.0-bin-scala_2.12.tgz \
    && tar xzf flink-1.18.0-bin-scala_2.12.tgz \
    && mv flink-1.18.0 /opt/flink
ENV PATH="/opt/flink/bin:${PATH}"

# Set the environment variable for JAVA_HOME
ENV JAVA_HOME /usr/lib/jvm/java-11-openjdk-amd64

# Install Python dependencies
RUN pip install matplotlib numpy

# Cleanup
RUN apt-get clean

# Copy the Rust and Java source code into the image
COPY . .

# The default command to run when starting the container could be your experiments
CMD ["python3", "run.py"]

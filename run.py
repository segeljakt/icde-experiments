import subprocess
import numpy as np
import os
import matplotlib.pyplot as plt

flink = ["flink", "run", "flink-nexmark/target/flink-nexmark-1.0-SNAPSHOT.jar"]
rust = ["./rust-nexmark/target/release/rust-nexmark"]

num_iterations = 10
num_warmups = 5
num_events = 1_000_000
bid_proportion = 0
auction_proportion = 75
person_proportion = 25
data = f"data-{num_events}/"

# Start Flink cluster if not running
p = subprocess.run(["jps"], capture_output=True)
if "TaskManager" not in p.stdout.decode('utf-8'):
    subprocess.run(["start-cluster.sh", "-c", "flink-conf.yaml"], check=True)

# Build Rust project
subprocess.run(["cargo", "build", "--release",
               "--manifest-path=rust-nexmark/Cargo.toml"])

# Generate data
if not os.path.exists(data):
    # Generate input files
    subprocess.run([
        "cargo", "run",
        "--release",
        "--manifest-path=data-generator/Cargo.toml",
        "--",
        "--bid-proportion=" + str(bid_proportion),
        "--auction-proportion=" + str(auction_proportion),
        "--person-proportion=" + str(person_proportion),
        "--num-events=" + str(num_events),
        "--dir=" + data
    ], check=True)

# Build Java project
subprocess.run(["mvn", "clean", "package", "-f", "flink-nexmark/pom.xml"])

# Run experiments
std_devs = []
means = []
labels = []


def measure(name, cmd, query):
    execution_times = []
    for iteration in range(num_iterations + num_warmups):
        if iteration == num_warmups:
            print("Warmup done, starting measurements")
        print(
            f'{cmd}, '
            f'Iteration {iteration+1}/{num_iterations + num_warmups}')
        output = subprocess.run(
            cmd + [data, query],
            capture_output=True,
            text=True, check=True).stderr
        seconds = int(output) / 1000
        print("Execution time:", seconds, "sec")
        if iteration >= num_warmups:
            execution_times.append(seconds)
    std_devs.append(np.std(execution_times))
    means.append(np.mean(execution_times))
    labels.append(f"{name} {query}")


measure("flink", flink, "q3")
measure("rust", rust, "q3")
measure("flink", flink, "q3-opt")
measure("rust", rust, "q3-opt")

x = np.arange(len(labels))
width = 0.35

fig, ax = plt.subplots()
rects1 = ax.bar(x - width/2, means, width,
                label='Execution Time', yerr=std_devs)

upper_limit = max(means) + max(std_devs)
ax.set_ylim(0, upper_limit * 1.1)

# Add some text for labels, title and custom x-axis tick labels, etc.
ax.set_ylabel('Execution Time (seconds)')
ax.set_title(f'Execution Time by System and Query '
             f'({num_events} events, {num_iterations} iterations)')
ax.set_xticks(x)
ax.set_xticklabels(labels)
ax.legend()

ax.bar_label(rects1, padding=3)

plt.savefig(f'results-{num_events}-{num_iterations}.pdf', bbox_inches='tight')
plt.show()

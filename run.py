import subprocess
import numpy as np
import os
import matplotlib.pyplot as plt

flink = ["flink", "run", "flink-nexmark/target/flink-nexmark-1.0-SNAPSHOT.jar"]
rust = ["./rust-nexmark/target/release/rust-nexmark"]

num_iterations = 10
num_warmups = 5
num_events = 1_000_000
q1_data = f"q1-data-{num_events}/"
q3_data = f"q3-data-{num_events}/"

# Start Flink cluster if not running
p = subprocess.run(["jps"], capture_output=True)
if "TaskManager" not in p.stdout.decode('utf-8'):
    subprocess.run(["start-cluster.sh", "-c", "flink-conf.yaml"], check=True)

# Build Rust project
subprocess.run(["cargo", "build", "--release",
               "--manifest-path=rust-nexmark/Cargo.toml"])

# Generate data for query1
if not os.path.exists(q1_data):
    # Generate input files
    subprocess.run([
        "cargo", "run",
        "--release",
        "--manifest-path=data-generator/Cargo.toml",
        "--",
        "--bid-proportion=100",
        "--auction-proportion=0",
        "--person-proportion=0",
        "--num-events=" + str(num_events),
        "--dir=" + q1_data
    ], check=True)

# Generate data for query3
if not os.path.exists(q3_data):
    # Generate input files
    subprocess.run([
        "cargo", "run",
        "--release",
        "--manifest-path=data-generator/Cargo.toml",
        "--",
        "--bid-proportion=0",
        "--auction-proportion=75",
        "--person-proportion=25",
        "--num-events=" + str(num_events),
        "--dir=" + q3_data
    ], check=True)

# Build Java project
subprocess.run(["mvn", "clean", "package", "-f", "flink-nexmark/pom.xml"])


# Run experiments
def measure(name, program, query, data):
    execution_times = []
    cmd = program + [data, query]
    for iteration in range(num_iterations + num_warmups):
        if iteration == num_warmups:
            print("Warmup done, starting measurements")
        print(
            f'{cmd}, '
            f'Iteration {iteration+1}/{num_iterations + num_warmups}')
        output = subprocess.run(
            cmd,
            capture_output=True,
            text=True, check=True).stderr
        seconds = int(output) / 1000
        print("Execution time:", seconds, "sec")
        if iteration >= num_warmups:
            execution_times.append(seconds)
    std_devs.append(np.std(execution_times))
    means.append(np.mean(execution_times))
    labels.append(f"{name} {query}")


def plot(query, means, std_devs, labels):
    x = np.arange(len(labels))
    width = 0.35

    fig, ax = plt.subplots()
    rects1 = ax.bar(x - width/2, means, width,
                    label='Execution Time', yerr=std_devs)

    upper_limit = max(means) + max(std_devs)
    ax.set_ylim(0, upper_limit * 1.1)

    ax.set_ylabel('Execution Time (seconds)')
    ax.set_title(f'Execution Time by System and Query ({num_events} events)')

    ax.set_xticks(x - width/2)
    ax.set_xticklabels(labels, ha='center')

    ax.tick_params(axis='x', length=0)

    ax.legend()

    ax.bar_label(rects1, padding=3)

    plt.tight_layout()
    plt.savefig(f'{query}-{num_events}-{num_iterations}.pdf',
                bbox_inches='tight')


labels = []
means = []
std_devs = []

measure("flink", flink, "q1-io", q1_data)
measure("rust", rust, "q1-io", q1_data)
measure("flink", flink, "q1", q1_data)
measure("rust", rust, "q1", q1_data)

plot("q1", means, std_devs, labels)

labels = []
means = []
std_devs = []

measure("flink", flink, "q3-io", q3_data)
measure("rust", rust, "q3-io", q3_data)
measure("flink", flink, "q3", q3_data)
measure("rust", rust, "q3", q3_data)
measure("flink", flink, "q3-opt", q3_data)
measure("rust", rust, "q3-opt", q3_data)

plot("q3", means, std_devs, labels)

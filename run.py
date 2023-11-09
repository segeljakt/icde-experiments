import subprocess
import numpy as np
import os
import matplotlib.pyplot as plt
import re

setup = {
    "programs": [
        {
            "name": "flink",
            "cmd": ["flink", "run",
                    "flink-nexmark/target/flink-nexmark-1.0-SNAPSHOT.jar"],

        },
        {
            "name": "rust",
            "cmd": ["./rust-nexmark/target/release/rust-nexmark"],
        },
    ],
    "queries": ["query1", "query2"],  # , "query3", "query3-opt"],
    "num_events": 10_000_000,
    "person-proportion": 1,
    "auction-proportion": 3,
    "bid-proportion": 46,
    "num_iterations": 5,
}

if not os.path.exists('data/'):
    # Generate input files
    subprocess.run([
        "cargo", "run",
        "--release",
        "--manifest-path=data-generator/Cargo.toml",
        "--",
        "--num-events=" + str(setup['num_events']),
        "--person-proportion=" + str(setup['person-proportion']),
        "--auction-proportion=" + str(setup['auction-proportion']),
        "--bid-proportion=" + str(setup['bid-proportion']),
        "--event-type=bid",
        "--dir=data/"
    ], check=True)

# Start Flink cluster
if "TaskManager" not in \
        subprocess.run(["jps"], capture_output=True).stdout.decode('utf-8'):
    subprocess.run(["start-cluster.sh"], check=True)

# Build Java project
subprocess.run(["mvn", "clean", "package",
                "-f", "flink-nexmark/pom.xml"])

# Build Rust project
subprocess.run(["cargo", "build",
                "--release",
                "--manifest-path=rust-nexmark/Cargo.toml",
                ], check=True)

# Run experiments
results = []

for program in setup['programs']:
    for query in setup["queries"]:
        execution_times = []
        for i in range(setup['num_iterations']):
            print("Running", program['name'], query,
                  i + 1, "/", setup['num_iterations'])
            cmd = program['cmd'] + [query]
            output = subprocess.run(cmd,
                                    capture_output=True,
                                    text=True,
                                    check=True).stdout
            millis = int(re.search(r"Job Runtime: (\d+) ms", output).group(1))
            seconds = millis / 1000
            print("Execution time:", seconds, "sec")
            execution_times.append(seconds)
        results.append((program['name'], query, execution_times))

means = [np.mean(times) for _, _, times in results]
stds = [np.std(times) for _, _, times in results]

# Number of groups (queries) and width of each bar
n_groups = len(setup['queries'])
bar_width = 0.35

# Setting up the x positions for the bars
index = np.arange(n_groups) * len(setup['programs']) * bar_width
fig, ax = plt.subplots(figsize=(15, 6))

for i, (program, _, _) in enumerate(results):
    ax.bar(index[i % n_groups] + (i // n_groups) * bar_width,
           means[i], bar_width,
           yerr=stds[i],
           label=program,
           capsize=5, alpha=0.7)

# Labeling and aesthetics
ax.set_xlabel('Query')
ax.set_ylabel('Average Execution Time (sec)')
ax.set_xticks(index + bar_width * (len(setup['programs']) / 2 - 0.5))
ax.set_xticklabels(setup['queries'])
ax.legend()
ax.grid(axis='y')

total_proportion = setup['person-proportion'] \
    + setup['auction-proportion'] \
    + setup['bid-proportion']

ax.set_title('Average Execution Time of Nexmark Queries for Flink and Rust for'
             '{0}M events ({1}% Person, {2}% Auction, {3}% Bid)'.format(
                 setup['num_events'] // 1_000_000,
                 setup['person-proportion'] / total_proportion * 100,
                 setup['auction-proportion'] / total_proportion * 100,
                 setup['bid-proportion'] / total_proportion * 100))

plt.tight_layout()
plt.savefig('nexmark-results.pdf')

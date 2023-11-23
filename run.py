import subprocess
import numpy as np
# import matplotlib.pyplot as plt
import os

setup = {
    "programs": [
        {
            "name": "rust",
            "cmd": ["./rust-nexmark/target/release/rust-nexmark"],
        },
        {
            "name": "flink",
            "cmd": ["flink", "run",
                    "flink-nexmark/target/flink-nexmark-1.0-SNAPSHOT.jar"],
        },
    ],
    "query_groups": [("query3", "query3-opt")],
    "num_events": [1_000_000],
    "num_iterations": 1,
    "dir": "data/"
}

# Start Flink cluster if not running
p = subprocess.run(["jps"], capture_output=True)
if "TaskManager" not in p.stdout.decode('utf-8'):
    subprocess.run(["start-cluster.sh", "-c", "flink-conf.yaml"], check=True)

# Build Rust project
subprocess.run(["cargo", "build", "--release",
               "--manifest-path=rust-nexmark/Cargo.toml"])

# Generate data
if not os.path.exists('data/'):
    # Generate input files
    subprocess.run([
        "cargo", "run",
        "--release",
        "--manifest-path=data-generator/Cargo.toml",
        "--",
        "--num-events=" + str(max(setup['num_events'])),
        "--dir=" + setup['dir']
    ], check=True)

# Build Java project
subprocess.run(["mvn", "clean", "package", "-f", "flink-nexmark/pom.xml"])

# Run experiments
results = {}
plot_data = {}

for query_group in setup["query_groups"]:
    for query in query_group:
        for (i, program) in enumerate(setup['programs']):
            execution_times = []
            for num_events in setup['num_events']:
                for i in range(setup['num_iterations']):
                    print(f'{program["name"]}, {query}, {num_events}: '
                          f'{i+1}/{setup["num_iterations"]}')
                    output = subprocess.run(
                        program['cmd'] +
                        [query, str(num_events), setup['dir']],
                        capture_output=True,
                        text=True,
                        check=True).stderr
                    seconds = int(output) / 1000
                    print("Execution time:", seconds, "sec")
                    execution_times.append(seconds)
            mean = np.mean(execution_times)
            std_dev = np.std(execution_times)

print(results)

# # Plotting
# for query_group, group_results in results.items():
#     fig, ax = plt.subplots(figsize=(15, 6))
#     bar_width = 0.35 / len(setup['programs'])
#     num_programs = len(setup['programs'])
#     index = np.arange(len(setup['num_events']))
#
#     for query in query_group:
#         for program, i, num_events, mean_time, std_dev in group_results[query]:
#             bar_pos = index + (bar_width * i) if num_programs > 1 else index
#             ax.bar(bar_pos,
#                    mean_time,
#                    bar_width,
#                    yerr=std_dev,
#                    label=f'{program} ({query}, {num_events} events)',
#                    capsize=5,
#                    alpha=0.7)
#
#     # Labeling and aesthetics
#     ax.set_title('Job Execution Time of Nexmark '
#                  f'{query_group[0]} vs {query_group[1]} '
#                  f'(2% Person, 6% Auction, 92% Bid)'.format())
#
#     ax.set_xlabel('Number of Events')
#     ax.set_ylabel('Average Execution Time (sec)')
#     ax.set_xticks(index + bar_width / 2)
#     ax.set_xticklabels([f"{n//1_000_000}M" for n in setup['num_events']])
#     ax.legend()
#     ax.grid(axis='y')
#
#     plt.tight_layout()
#     plt.savefig(f'nexmark-results-{query_group[0]}.pdf')

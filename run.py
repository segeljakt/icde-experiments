import subprocess
import numpy as np
import os
import matplotlib.pyplot as plt

flink = ["flink", "run", "flink-nexmark/target/flink-nexmark-1.0-SNAPSHOT.jar"]
rust = ["./rust-nexmark/target/release/rust-nexmark"]

num_iterations = 3
num_warmups = 2
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


# Experiment 1 ----------------------------------------------------------------

labels = []
means = []
std_devs = []

measure("flink", flink, "q1-io", q1_data)
measure("rust", rust, "q1-io", q1_data)
measure("flink", flink, "q1", q1_data)
measure("rust", rust, "q1", q1_data)

x = np.arange(2)
width = 0.35  # Width of the bars

fig, ax = plt.subplots()

# Plotting the first pair (flink q1-io and flink q1)
ax.bar(x[0], means[0], width, yerr=std_devs[0], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[0], means[2], width, yerr=std_devs[2], label=labels[2], alpha=0.7)
ax.text(x[0], means[0]/2, "IO", ha='center', va='center', fontsize=11)

# Plotting the second pair (rust q1-io and rust q1)
ax.bar(x[1], means[1], width, yerr=std_devs[1], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[1], means[3], width, yerr=std_devs[3], label=labels[3], alpha=0.7)
ax.text(x[1], means[1]/2, "IO", ha='center', va='center', fontsize=11)

ax.set_ylabel('Execution Time (seconds)')
ax.set_title(f'Execution Time by System and Query ({num_events} events)')
ax.set_xticks(x)
ax.set_xticklabels([labels[2], labels[3]])
ax.legend()

plt.tight_layout()
plt.savefig(f'q1-{num_events}-{num_iterations}.pdf', bbox_inches='tight')

# Experiment 2 ----------------------------------------------------------------

labels = []
means = []
std_devs = []

measure("flink", flink, "q3-io", q3_data)
measure("rust", rust, "q3-io", q3_data)
measure("flink", flink, "q3", q3_data)
measure("rust", rust, "q3", q3_data)
measure("flink", flink, "q3-opt", q3_data)
measure("rust", rust, "q3-opt", q3_data)

x = np.arange(4)
width = 0.35  # Width of the bars

fig, ax = plt.subplots()

# Plotting the first pair (flink q1)
ax.bar(x[0], means[0], width, yerr=std_devs[0], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[0], means[2], width, yerr=std_devs[2], label=labels[2], alpha=0.7)
ax.text(x[0], means[0]/2, "IO", ha='center', va='center', fontsize=11)

# Plotting the second pair (rust q1)
ax.bar(x[1], means[1], width, yerr=std_devs[1], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[1], means[3], width, yerr=std_devs[3], label=labels[3], alpha=0.7)
ax.text(x[1], means[1]/2, "IO", ha='center', va='center', fontsize=11)

# Plotting the second pair (flink q1-opt)
ax.bar(x[2], means[0], width, yerr=std_devs[0], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[2], means[4], width, yerr=std_devs[4], label=labels[4], alpha=0.7)
ax.text(x[2], means[0]/2, "IO", ha='center', va='center', fontsize=11)

# Plotting the second pair (rust q1-opt)
ax.bar(x[3], means[1], width, yerr=std_devs[1], hatch='/',
       color='none', edgecolor='black', linewidth=0.5)
ax.bar(x[3], means[5], width, yerr=std_devs[5], label=labels[5], alpha=0.7)
ax.text(x[3], means[1]/2, "IO", ha='center', va='center', fontsize=11)

ax.set_ylabel('Execution Time (seconds)')
ax.set_title(f'Execution Time by System and Query ({num_events} events)')
ax.set_xticks(x)
ax.set_xticklabels([labels[2], labels[3], labels[4], labels[5]])
ax.legend()

plt.tight_layout()
plt.savefig(f'q3-{num_events}-{num_iterations}.pdf', bbox_inches='tight')

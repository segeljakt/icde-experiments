Below are instructions for setting up a custom runner, running in a Docker container, for GitHub actions.

```bash
# Setup docker

docker pull ubuntu:latest
docker run -i -t ubuntu:latest /bin/bash

# Setup user

passwd # change root password
adduser arc-runner sudo
su -l arc-runner

# Install apt dependencies

sudo apt install software-properties-common
sudo add-apt-repository ppa:git-core/ppa -y
sudo apt update && apt upgrade -y
sudo apt install -y git vim curl z3 libz3-dev curl libssl-dev gcc pkg-config make ninja-build zip openjdk-8-jdk software-properties-common texlive-xetex latexmk gettext ccache cmake

sudo ln -s /usr/bin/clang-16 /usr/local/bin/clang
sudo ln -s /usr/bin/clang++-16 /usr/local/bin/clang

# Install Rust

curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
echo 'source $HOME/.cargo/env' >> ~/.bashrc
cargo install mdbook

# Install GitHub Actions Runner

# Follow this tutorial (Which generates a unique token):
#     https://github.com/cda-group/arc/settings/actions/runners/new?arch=x64&os=linux

# Setup Runner

./run.sh &
disown <PID>

# To exit the container: <C-p><C-q>

# [OPTIONAL] Check that everything builds

cd ~
git clone https://github.com/cda-group/arc
cd ~/arc
git checkout mlir
git submodule update --init --recursive

# Check if arc-lang builds

cd ~/arc/arc-lang
cargo check --all-features --tests --bins --examples --benches

# Check if arc-mlir builds

cd ~/arc/arc-mlir/
./arc-mlir-build
```

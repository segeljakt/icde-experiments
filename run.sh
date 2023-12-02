docker build --tag experiment .
docker run --name experiment experiment
docker cp experiment:/output output

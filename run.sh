docker build --tag experiment .
docker run --name experiment experiment
docker cp experiment:/experiment/output output

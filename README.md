# ICDE Experiments

To run the experiments, install and startup [docker](https://docs.docker.com/) and then run:

```
docker build --tag experiment .
docker run --tag experiment --name experiment
docker cp experiment:/output output
```

The output of the experiment can then be found in the generated `output/` folder.

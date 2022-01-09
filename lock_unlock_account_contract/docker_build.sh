#!/bin/sh
docker build -t contract-builder --build-arg USER_ID=$(id -u) --build-arg GROUP_ID=$(id -g) .

#!/bin/sh

HOST_DIR="${HOST_DIR:-$(pwd)}"

docker run \
     --mount type=bind,source=$HOST_DIR,target=/host -w /host \
     --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
     -i -t nearnames-marketplace \
     /bin/bash

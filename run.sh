#!/bin/sh
#Problemas con wayland
set -x
set -e
WINIT_UNIX_BACKEND=x11 cargo run

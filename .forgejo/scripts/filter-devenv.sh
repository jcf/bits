#!/usr/bin/env bash

# Filter devenv spinner noise from output, preserving error lines
grep -v '^[⠋⠙⠹⠸⠼⠴⠦⠧⠇]' || true

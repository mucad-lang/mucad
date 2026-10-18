#!/bin/sh

watch -n 0.1 'ps aux | grep -v grep | grep -q "mucad-lsp -l " && echo "mucad-lsp running" || echo "mucad-lsp NOT running"'

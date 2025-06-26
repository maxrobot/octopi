# Octopi

## Overview

This is _"a simple toy payments engine that reads a series of transactions
from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV"_.

## Information

## General Plan

1. Implement the account system and transaction engine

- Store this in memory start with f64 but consider moving to the rust_decimal package which might be slower but more performant
- Make this its own package and get a load of tests done
- Consumer with backpressure handling

2. As streaming for the CSV reader

- Producer model

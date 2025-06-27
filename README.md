# Octopi

## Overview

This is _"a simple toy payments engine that reads a series of transactions
from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the
state of clients accounts as a CSV"_.

## Assumptions

1. A withdrawal cannot be disputed

It doesn't make sense that a withdrawal could be dispute as the mechanism, i.e. increase held decrease available, doesn't even hold in that scenario. Further as per the documentation this should work similar to an ATM which cannot really dispute withdrawals.

2. Disputes with insufficient funds are handled partially

If we make a deposit and then withdraw but subsequently perform a dispute then we may handle it partially. This could cover the case where a bad actor makes a deposit and then manages to withdraw some of the funds, we are then able to nonetheless dispute the deposit and cover a portion of the losses from what is available.

This does open up the issue of multiple disputes which could be the case if a malicious actor hacked many accounts depositing into the engine and then at a later date withdrew some funds, then disputes would be resolved on a first-come first-served basis, which is probably not ideal but we will ignore this edge case in this toy example.

## General Plan

1. Implement the account system and transaction engine

- Store this in memory start with f64 but consider moving to the rust_decimal package which might be slower but more performant
- Make this its own package and get a load of tests done
- Consumer with backpressure handling

2. As streaming for the CSV reader

- Producer model

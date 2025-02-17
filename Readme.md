# Reproduction Example for non-deterministic FIFO SNS/SQS topics

The setup is the following:

* 1 SNS topic, FIFO=true
* 3 SQS queues, FIFO=true, all subscribed to above topic

Rust client binary that does the following:

* Submit 100 messages to SNS topic
* Receive 100 messages from all 3 queues

Due to the FIFO setting, one would expect that the 3 100-message vectors collected from each queue are identical.
However, this is not the case.

## Reproduce

```bash
./setup_localstack.sh
(source .env && cargo run)
```

Observe panic due to message vectors not being equal.

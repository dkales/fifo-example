use std::time::Duration;

use aws_config::BehaviorVersion;
use clap::Parser;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Parser)]
struct Args {
    #[clap(long, env = "SNS_TOPIC_ARN")]
    sns_topic_arn: String,

    #[clap(long, env = "SQS_QUEUE0_URL")]
    sqs_queue0_url: String,
    #[clap(long, env = "SQS_QUEUE1_URL")]
    sqs_queue1_url: String,
    #[clap(long, env = "SQS_QUEUE2_URL")]
    sqs_queue2_url: String,

    #[clap(long, env = "INTERVAL_MS", default_value = "10")]
    interval_ms: u64,

    #[clap(long, env = "LIMIT", default_value = "100")]
    limit: usize,
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    println!("sns_topic_arn: {}", args.sns_topic_arn);
    println!("sqs_queue0_url: {}", args.sqs_queue0_url);
    println!("sqs_queue1_url: {}", args.sqs_queue1_url);
    println!("sqs_queue2_url: {}", args.sqs_queue2_url);

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    let sns_client = aws_sdk_sns::Client::new(&config);
    let sqs_client = aws_sdk_sqs::Client::new(&config);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(args.interval_ms));
        let mut count = 0;
        loop {
            if count >= args.limit {
                println!("sent {} messages", count);
                break;
            }
            interval.tick().await;
            count += 1;
            let message = format!("{};{}", count, Uuid::new_v4());
            sns_client
                .publish()
                .message(message)
                .topic_arn(args.sns_topic_arn.clone())
                .message_group_id("TEST")
                .send()
                .await
                .unwrap();
        }
    });

    let (res0, res1, res2) = tokio::join!(
        recv_message(&sqs_client, &args.sqs_queue0_url, args.limit),
        recv_message(&sqs_client, &args.sqs_queue1_url, args.limit),
        recv_message(&sqs_client, &args.sqs_queue2_url, args.limit),
    );

    assert_eq!(res0, res1);
    assert_eq!(res0, res2);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnsMessage {
    #[serde(rename = "Type")]
    typ: String,
    #[serde(rename = "MessageId")]
    message_id: String,
    #[serde(rename = "TopicArn")]
    topic_arn: String,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Timestamp")]
    timestamp: Option<String>,
    #[serde(rename = "UnsubscribeUrl")]
    unsubscribe_url: Option<String>,
    #[serde(rename = "SequenceNumber")]
    sequence_number: Option<String>,
}

async fn recv_message(sqs_client: &aws_sdk_sqs::Client, queue_url: &str, limit: usize) -> Vec<u64> {
    let mut res = Vec::with_capacity(limit);
    for _ in 0..limit {
        loop {
            let resp = sqs_client
                .receive_message()
                .queue_url(queue_url)
                .max_number_of_messages(1)
                .send()
                .await
                .unwrap();
            if let Some(messages) = resp.messages {
                assert!(messages.len() == 1);
                for message in messages {
                    let body = message.body.unwrap();
                    let parsed: SnsMessage = serde_json::from_str(&body).unwrap();
                    let split = parsed.message.split(';').next().unwrap();
                    let num = split.parse::<u64>().unwrap();
                    res.push(num);
                    sqs_client
                        .delete_message()
                        .queue_url(queue_url)
                        .receipt_handle(message.receipt_handle.unwrap())
                        .send()
                        .await
                        .unwrap();
                }
                break;
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    res
}

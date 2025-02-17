#!/bin/bash
docker run -d -p 4566:4566 -p 4571:4571 --name localstack -e SERVICES=sns,sqs --rm localstack/localstack:4.1.1
sleep 2

export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_REGION=us-east-1
export AWS_ENDPOINT_URL=http://localhost:4566

create_fifo_queue() {
  local QUEUE_NAME_TO_CREATE=$1
  awslocal sqs create-queue --queue-name "${QUEUE_NAME_TO_CREATE}" --region $AWS_REGION --attributes "{\"FifoQueue\":\"true\", \"ContentBasedDeduplication\":\"true\", \"VisibilityTimeout\":\"30\"}"
}

create_sns(){
  local TOPIC_NAME_TO_CREATE=$1
  awslocal sns create-topic --name "${TOPIC_NAME_TO_CREATE}" --region $AWS_REGION --attributes "{\"FifoTopic\":\"true\", \"ContentBasedDeduplication\":\"true\"}"
}

# Create the topic
SNS_TOPIC_NAME=input.fifo
create_sns $SNS_TOPIC_NAME
SNS_TOPIC_ARN=arn:aws:sns:us-east-1:000000000000:$SNS_TOPIC_NAME

# Create the queues
SQS_QUEUE0_NAME=queue0.fifo
SQS_QUEUE1_NAME=queue1.fifo
SQS_QUEUE2_NAME=queue2.fifo
create_fifo_queue $SQS_QUEUE0_NAME
create_fifo_queue $SQS_QUEUE1_NAME
create_fifo_queue $SQS_QUEUE2_NAME
SQS_QUEUE0_ARN=arn:aws:sqs:us-east-1:000000000000:$SQS_QUEUE0_NAME
SQS_QUEUE1_ARN=arn:aws:sqs:us-east-1:000000000000:$SQS_QUEUE1_NAME
SQS_QUEUE2_ARN=arn:aws:sqs:us-east-1:000000000000:$SQS_QUEUE2_NAME

# Subscribe the queues to the topic
awslocal sns subscribe --topic-arn "$SNS_TOPIC_ARN" --protocol sqs --notification-endpoint $SQS_QUEUE0_ARN --region $AWS_REGION
awslocal sns subscribe --topic-arn "$SNS_TOPIC_ARN" --protocol sqs --notification-endpoint $SQS_QUEUE1_ARN --region $AWS_REGION
awslocal sns subscribe --topic-arn "$SNS_TOPIC_ARN" --protocol sqs --notification-endpoint $SQS_QUEUE2_ARN --region $AWS_REGION

echo "export AWS_ACCESS_KEY_ID=test" > .env
echo "export AWS_SECRET_ACCESS_KEY=test" >> .env
echo "export AWS_REGION=us-east-1" >> .env
echo "export AWS_ENDPOINT_URL=http://localhost:4566" >> .env
echo "export SNS_TOPIC_ARN=$SNS_TOPIC_ARN" >> .env
echo "export SQS_QUEUE0_URL=http://sqs.us-east-1.localhost.localstack.cloud:4566/000000000000/$SQS_QUEUE0_NAME" >> .env
echo "export SQS_QUEUE1_URL=http://sqs.us-east-1.localhost.localstack.cloud:4566/000000000000/$SQS_QUEUE1_NAME" >> .env
echo "export SQS_QUEUE2_URL=http://sqs.us-east-1.localhost.localstack.cloud:4566/000000000000/$SQS_QUEUE2_NAME" >> .env

cat .env

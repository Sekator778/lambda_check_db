# lambda_check_db

## Overview

This project contains a Rust-based AWS Lambda function that checks the connection to a PostgreSQL database hosted on an EC2 instance. The function attempts to connect to the database and verifies its readiness by running a simple SQL query.

## Project Structure

- `Cargo.toml`: Contains the metadata and dependencies for the Rust project.
- `Cargo.lock`: Locks the dependencies to specific versions.
- `src/main.rs`: The main Rust source code file containing the Lambda function logic.
- `.gitignore`: Specifies which files and directories Git should ignore.

## Prerequisites

- Rust programming language installed.
- Docker installed (for building the Lambda function for AWS).
- AWS CLI configured with the necessary permissions.
- Environment variables set for the database connection parameters.

## Environment Variables

- `DB_HOST`: The hostname of the PostgreSQL database.
- `DB_USER`: The username for the PostgreSQL database.
- `DB_PASSWORD`: The password for the PostgreSQL database.
- `DB_NAME`: The name of the PostgreSQL database.

## Building and Deploying the Lambda Function

### Step 1: Build the Lambda Function

Use Docker to build the Lambda function for the `x86_64-unknown-linux-musl` target:

```sh
docker build -t lambda_check_db .
container_id=$(docker create lambda_check_db)
docker cp ${container_id}:/app/bootstrap ./bootstrap
docker rm ${container_id}
zip lambda.zip bootstrap
```

### Step 2: Deploy the Lambda Function

#### First Deployment

Create the Lambda function using the AWS CLI:

```sh
aws lambda create-function --function-name checkDbConnection \
  --handler bootstrap \
  --runtime provided.al2 \
  --role arn:aws:iam::xxxx:role/service-role/xxx-xxxx-role-xxx \
  --zip-file fileb://lambda.zip \
  --region eu-central-1 
```

#### Redeploying

If the Lambda function already exists, you can update it:

```sh
aws lambda update-function-configuration --function-name checkDbConnection --timeout 30 --region eu-central-1
```

## Security Group Setup

### Create a New Security Group

```sh
aws ec2 create-security-group \
  --group-name lambdacheckdb-sg-avbo \
  --description "Security group for Lambda function" \
  --vpc-id vpc-06e64af531b400f35 \
  --region eu-central-1
```

### Get the Security Group ID

```sh
SG_ID=$(aws ec2 describe-security-groups \
  --filters Name=group-name,Values=lambdacheckdb-sg-avbo \
  --query "SecurityGroups[0].GroupId" \
  --output text \
  --region eu-central-1)
```

### Add Inbound Rule

Allow TCP traffic on port 5432 from the CIDR block 10.0.0.0/8:

```sh
aws ec2 authorize-security-group-ingress \
  --group-id "sg-xx" \
  --protocol tcp \
  --port 23-10052 \
  --cidr 10.0.0.0/8 \
  --region eu-central-1
```

### Add Outbound Rule

Allow TCP traffic on port 5432 to the CIDR block 10.0.0.0/8:

```sh
aws ec2 authorize-security-group-egress \
  --group-id "sg-xxx" \
  --protocol tcp \
  --port 5432 \
  --cidr 10.0.0.0/8 \
  --region eu-central-1
```

## Lambda Function Configuration

### Update Lambda Function Configuration

```sh
# Set Variables
vpc_id="vpc-xxx"
subnet_ids="subnet-xxx,subnet-xxx,subnet-xxx"
region="eu-central-1"
function_name="checkDbConnection"
sg_id="sg-xxxx"  # The actual security group ID for your Lambda

# Update Lambda Function Configuration
aws lambda update-function-configuration --function-name $function_name \
  --vpc-config SubnetIds=$subnet_ids,SecurityGroupIds=$sg_id \
  --region $region \
  --timeout 300
```

## Example Data for the Lambda Function

```json
{
  "db_host": "xxxxa",
  "db_port": "5432",
  "db_user": "xxx",
  "db_password": "P@xxxxx",
  "db_name": "xxxx"
}
```

## Testing the Lambda Function

### Using Curl

```sh
curl -v https://gsm2cya2zhkx435sipznidwt4a0nhgcx.lambda-url.eu-central-1.on.aws/ \
     -H "Content-Type: application/json" \
     -d '{"db_host":"xxx","db_port":"5432","db_user":"xx","db_password":"xxx","db_name":"xx"}'
```

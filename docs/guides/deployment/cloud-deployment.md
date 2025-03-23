---
title: Navius Cloud Deployment Guide
description: Guide for deploying Navius applications to major cloud providers
category: guides
tags:
  - deployment
  - cloud
  - aws
  - azure
  - gcp
related:
  - production-deployment.md
  - ../../reference/configuration/environment-variables.md
  - ../development/project-navigation.md
last_updated: March 23, 2025
version: 1.0
---

# Navius Cloud Deployment Guide

## Overview
This guide provides specific instructions for deploying Navius applications to major cloud providers, including AWS, Azure, and Google Cloud Platform. It covers cloud-specific services, architectures, and optimization strategies to make the most of each platform.

## Prerequisites
Before deploying to a cloud provider, ensure you have:

- A built and tested Navius application
- Cloud provider account with appropriate permissions
- Basic familiarity with the target cloud platform
- Cloud provider CLI tools installed locally

## AWS Deployment

### ECS/Fargate Deployment

AWS Elastic Container Service (ECS) with Fargate is a serverless container platform ideal for Navius applications:

1. **Create ECR Repository**:

```bash
aws ecr create-repository --repository-name navius-api
```

2. **Build and Push Docker Image**:

```bash
# Build image
docker build -t navius-api .

# Tag image
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin <your-account-id>.dkr.ecr.us-east-1.amazonaws.com
docker tag navius-api:latest <your-account-id>.dkr.ecr.us-east-1.amazonaws.com/navius-api:latest

# Push image
docker push <your-account-id>.dkr.ecr.us-east-1.amazonaws.com/navius-api:latest
```

3. **Create ECS Task Definition**:

```json
{
  "family": "navius-api",
  "networkMode": "awsvpc",
  "executionRoleArn": "arn:aws:iam::<your-account-id>:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "navius-api",
      "image": "<your-account-id>.dkr.ecr.us-east-1.amazonaws.com/navius-api:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "hostPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        { "name": "RUST_LOG", "value": "info" },
        { "name": "RUN_ENV", "value": "production" }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/navius-api",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ],
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512"
}
```

4. **Create ECS Service**:

```bash
aws ecs create-service \
  --cluster navius-cluster \
  --service-name navius-api \
  --task-definition navius-api:1 \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-12345678,subnet-87654321],securityGroups=[sg-12345678],assignPublicIp=ENABLED}" \
  --load-balancers "targetGroupArn=arn:aws:elasticloadbalancing:us-east-1:<your-account-id>:targetgroup/navius-tg/1234567890123456,containerName=navius-api,containerPort=8080"
```

### Lambda Deployment (Serverless)

Navius can be deployed as a serverless application using AWS Lambda:

1. **Configure using Serverless Framework**:

```yaml
# serverless.yml
service: navius-api

provider:
  name: aws
  runtime: provided.al2
  region: us-east-1
  memorySize: 1024
  timeout: 29

functions:
  api:
    handler: bootstrap
    events:
      - http:
          path: /{proxy+}
          method: any
    environment:
      RUST_LOG: info
      DATABASE_URL: ${env:DATABASE_URL}

package:
  individually: true
  patterns:
    - '!**/*'
    - bootstrap
```

2. **Build for Lambda**:

```bash
# Build with Lambda support
cargo lambda build --release

# Deploy using Serverless Framework
serverless deploy
```

### Complete AWS Architecture

For a production-grade setup, consider this architecture:

```
                          ┌─────────────────┐
                          │ Route 53 (DNS)  │
                          └────────┬────────┘
                                  │
                          ┌────────▼────────┐
                          │  CloudFront     │
                          │      CDN        │
                          └────────┬────────┘
                                  │
                          ┌────────▼────────┐
                          │ Application Load│
                          │    Balancer     │
                          └────────┬────────┘
                                  │
                   ┌──────────────┴──────────────┐
                   │                             │
           ┌───────▼──────┐             ┌────────▼─────┐
           │  ECS Fargate  │            │  ECS Fargate  │
           │  Container 1  │            │  Container 2  │
           └───────┬──────┘             └────────┬─────┘
                   │                             │
                   └──────────────┬──────────────┘
                                 │
                   ┌─────────────▼─────────────┐
                   │      Amazon Aurora         │
                   │      PostgreSQL            │
                   └─────────────┬─────────────┘
                                 │
                   ┌─────────────▼─────────────┐
                   │      ElastiCache           │
                   │        Redis               │
                   └───────────────────────────┘
```

## Azure Deployment

### AKS (Azure Kubernetes Service)

Deploy Navius to Azure Kubernetes Service:

1. **Create AKS Cluster**:

```bash
az aks create \
  --resource-group navius-rg \
  --name navius-cluster \
  --node-count 2 \
  --enable-addons monitoring \
  --generate-ssh-keys
```

2. **Deploy Navius**:

```bash
# Get AKS credentials
az aks get-credentials --resource-group navius-rg --name navius-cluster

# Apply Kubernetes manifests
kubectl apply -f kubernetes/deployment.yaml
kubectl apply -f kubernetes/service.yaml
```

3. **Sample Kubernetes Manifests**:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: navius-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: navius-api
  template:
    metadata:
      labels:
        app: navius-api
    spec:
      containers:
      - name: navius-api
        image: your-registry.azurecr.io/navius-api:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: navius-secrets
              key: database-url
```

### Azure App Service

For simpler deployments, use Azure App Service:

1. **Create App Service Plan**:

```bash
az appservice plan create \
  --name navius-plan \
  --resource-group navius-rg \
  --sku P1V2 \
  --is-linux
```

2. **Create Web App**:

```bash
az webapp create \
  --resource-group navius-rg \
  --plan navius-plan \
  --name navius-api \
  --deployment-container-image-name your-registry.azurecr.io/navius-api:latest
```

3. **Configure Environment Variables**:

```bash
az webapp config appsettings set \
  --resource-group navius-rg \
  --name navius-api \
  --settings RUST_LOG=info DATABASE_URL="your-connection-string"
```

## Google Cloud Platform Deployment

### GKE (Google Kubernetes Engine)

Deploy to Google Kubernetes Engine:

1. **Create GKE Cluster**:

```bash
gcloud container clusters create navius-cluster \
  --zone us-central1-a \
  --num-nodes 2 \
  --machine-type e2-standard-2
```

2. **Deploy to GKE**:

```bash
# Configure kubectl
gcloud container clusters get-credentials navius-cluster --zone us-central1-a

# Deploy
kubectl apply -f kubernetes/deployment.yaml
kubectl apply -f kubernetes/service.yaml
```

### Cloud Run (Serverless)

For serverless deployments, use Cloud Run:

1. **Build and Push Docker Image**:

```bash
# Build image
docker build -t gcr.io/your-project/navius:latest .

# Push to Google Container Registry
docker push gcr.io/your-project/navius:latest
```

2. **Deploy to Cloud Run**:

```bash
gcloud run deploy navius-api \
  --image gcr.io/your-project/navius:latest \
  --platform managed \
  --region us-central1 \
  --memory 512Mi \
  --set-env-vars="RUST_LOG=info,DATABASE_URL=postgres://user:pass@host/db"
```

## Multi-Cloud Strategy

For critical applications, consider a multi-cloud strategy:

1. **Deployment Abstractions**:

```rust
// In your configuration
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
}

pub fn get_cache_client(provider: CloudProvider) -> CacheClient {
    match provider {
        CloudProvider::AWS => ElastiCacheClient::new(),
        CloudProvider::Azure => AzureRedisClient::new(),
        CloudProvider::GCP => MemorystoreClient::new(),
    }
}
```

2. **Infrastructure as Code**:

Use Terraform for multi-cloud deployments:

```hcl
# main.tf
provider "aws" {
  region = "us-east-1"
}

provider "azurerm" {
  features {}
}

module "aws_deployment" {
  source = "./modules/aws"
  # AWS-specific vars
}

module "azure_deployment" {
  source = "./modules/azure"
  # Azure-specific vars
}
```

## Cloud Cost Optimization

### Resource Sizing

Navius applications are efficient and require less resources than most frameworks:

| Cloud Provider | Service | Recommended Size | Estimated Monthly Cost |
|----------------|---------|------------------|------------------------|
| AWS | ECS Fargate | 0.25 vCPU, 512MB | ~$15-20 |
| AWS | Lambda | 512MB | ~$5-10 (depends on invocations) |
| Azure | App Service | P1V2 | ~$70 |
| Azure | AKS | Standard_B2s | ~$50 per node |
| GCP | Cloud Run | 1 vCPU, 512MB | ~$15-20 |
| GCP | GKE | e2-small | ~$20 per node |

### Autoscaling Configuration

Configure autoscaling for optimal cost:

```yaml
# AWS ECS Autoscaling
aws application-autoscaling register-scalable-target \
  --service-namespace ecs \
  --resource-id service/navius-cluster/navius-api \
  --scalable-dimension ecs:service:DesiredCount \
  --min-capacity 1 \
  --max-capacity 10

# Azure AKS Autoscaling
kubectl autoscale deployment navius-api --min=1 --max=10 --cpu-percent=70

# GCP Cloud Run Autoscaling
gcloud run services update navius-api \
  --min-instances 1 \
  --max-instances 10
```

## Cloud-specific Monitoring

### AWS CloudWatch

```bash
# Create CloudWatch Dashboard
aws cloudwatch put-dashboard \
  --dashboard-name NaviusMonitoring \
  --dashboard-body file://dashboard.json
```

### Azure Monitor

```bash
# Create custom Azure Monitor metric
az monitor metrics alert create \
  --name navius-high-cpu \
  --resource-group navius-rg \
  --scopes /subscriptions/{subscription-id}/resourceGroups/navius-rg/providers/Microsoft.Web/sites/navius-api \
  --condition "avg Percentage CPU > 75" \
  --window-size 5m \
  --evaluation-frequency 1m \
  --action /subscriptions/{subscription-id}/resourceGroups/navius-rg/providers/Microsoft.Insights/actionGroups/navius-alerts
```

### GCP Cloud Monitoring

```bash
# Create GCP alert policy
gcloud alpha monitoring policies create \
  --display-name="Navius High CPU" \
  --conditions="condition-display-name='High CPU Usage' target-type=service filter='metric.type=\"run.googleapis.com/container/cpu/utilization\" resource.type=\"cloud_run_revision\" resource.label.service_name=\"navius-api\"' aggregations.alignment-period=300s aggregations.per-series-aligner=ALIGN_MEAN aggregations.cross-series-reducer=REDUCE_MEAN comparison-type=COMPARISON_GT comparison-threshold=0.75"
```

## Related Documents

- [Production Deployment Guide](production-deployment.md) - General production deployment guidelines
- [Environment Variables Reference](../../reference/configuration/environment-variables.md) - Configuration options
- [Project Navigation](../development/project-navigation.md) - Understanding the project structure 
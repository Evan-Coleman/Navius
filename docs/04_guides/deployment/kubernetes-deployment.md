---
title: "Kubernetes Deployment Guide for Navius"
description: "Comprehensive guide for deploying and managing Navius applications in Kubernetes environments with best practices for scalability, resource management, and observability"
category: "guides"
tags:
  - kubernetes
  - deployment
  - k8s
  - containers
  - orchestration
  - cloud-native
related:
  - production-deployment.md
  - cloud-deployment.md
  - ../../05_reference/configuration/environment-variables.md
  - ../operations/monitoring.md
last_updated: "April 1, 2025"
version: "1.0"
---

# Kubernetes Deployment Guide for Navius

## Overview

This guide provides detailed instructions for deploying Navius applications to Kubernetes clusters. Navius is well-suited for Kubernetes deployments due to its lightweight nature, small memory footprint, and fast startup times.

## Prerequisites

Before deploying Navius to Kubernetes, ensure you have:

- A functioning Kubernetes cluster (v1.20+)
- kubectl CLI configured to access your cluster
- Docker registry access for storing container images
- Basic understanding of Kubernetes concepts (Deployments, Services, ConfigMaps)
- A containerized Navius application (see the [Docker Deployment Guide](docker-deployment.md))

## Deployment Manifest

### Basic Deployment

Create a file named `navius-deployment.yaml` with the following content:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: navius-api
  labels:
    app: navius-api
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
        image: your-registry/navius:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: RUN_ENV
          value: "production"
        resources:
          limits:
            cpu: "0.5"
            memory: "256Mi"
          requests:
            cpu: "0.1"
            memory: "128Mi"
        readinessProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /actuator/health
            port: 8080
          initialDelaySeconds: 15
          periodSeconds: 20
```

### Service Definition

Create a file named `navius-service.yaml`:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: navius-api
spec:
  selector:
    app: navius-api
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Ingress Configuration

For external access, create `navius-ingress.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: navius-ingress
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  rules:
  - host: navius.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: navius-api
            port:
              number: 80
  tls:
  - hosts:
    - navius.example.com
    secretName: navius-tls-secret
```

## Configuration Management

### ConfigMap for Application Settings

Create a configuration map for Navius settings:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: navius-config
data:
  config.yaml: |
    server:
      host: "0.0.0.0"
      port: 8080
    
    logging:
      level: "info"
      format: "json"
    
    cache:
      enabled: true
      redis_url: "redis://redis-service:6379"
```

Update your deployment to mount this ConfigMap:

```yaml
spec:
  containers:
  - name: navius-api
    # ... other settings ...
    volumeMounts:
    - name: config-volume
      mountPath: /etc/navius/config
  volumes:
  - name: config-volume
    configMap:
      name: navius-config
```

### Secrets Management

For sensitive information like database credentials:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: navius-secrets
type: Opaque
data:
  database_url: cG9zdGdyZXM6Ly91c2VyOnBhc3NAZGItc2VydmljZTo1NDMyL25hdml1cw== # Base64 encoded
  jwt_secret: c2VjcmV0X2tleV9jaGFuZ2VfbWVfaW5fcHJvZHVjdGlvbg== # Base64 encoded
```

Reference these secrets in your deployment:

```yaml
spec:
  containers:
  - name: navius-api
    # ... other settings ...
    env:
    # ... other env vars ...
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: navius-secrets
          key: database_url
    - name: JWT_SECRET
      valueFrom:
        secretKeyRef:
          name: navius-secrets
          key: jwt_secret
```

## Scaling Configuration

### Horizontal Pod Autoscaler

Create an HPA for automatic scaling:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: navius-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: navius-api
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## Resource Optimization

Navius applications are lightweight and can be optimized for Kubernetes:

### Resource Limits and Requests

```yaml
resources:
  limits:
    cpu: "0.5"     # Maximum CPU usage
    memory: "256Mi"  # Maximum memory usage
  requests:
    cpu: "0.1"     # Initial CPU reservation
    memory: "128Mi"  # Initial memory reservation
```

These values are conservative and can be adjusted based on your workload.

## Health Checks and Readiness

Navius provides built-in health endpoints that work well with Kubernetes:

```yaml
readinessProbe:
  httpGet:
    path: /actuator/health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
  successThreshold: 1
  failureThreshold: 3
  timeoutSeconds: 2

livenessProbe:
  httpGet:
    path: /actuator/health
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 20
  timeoutSeconds: 2
  failureThreshold: 3
```

## Deployment Process

### Deploy to Kubernetes

Apply the manifests in this order:

```bash
# Create ConfigMap and Secret first
kubectl apply -f navius-config.yaml
kubectl apply -f navius-secrets.yaml

# Deploy the application
kubectl apply -f navius-deployment.yaml

# Create the service
kubectl apply -f navius-service.yaml

# Configure ingress
kubectl apply -f navius-ingress.yaml

# Set up autoscaling
kubectl apply -f navius-hpa.yaml
```

### Verify Deployment

Check deployment status:

```bash
kubectl get deployments
kubectl get pods
kubectl get services
```

Test the service:

```bash
# Port-forward for local testing
kubectl port-forward svc/navius-api 8080:80

# Then access in your browser: http://localhost:8080/actuator/health
```

## Advanced Configuration

### Affinity and Anti-Affinity Rules

For better pod distribution:

```yaml
spec:
  affinity:
    podAntiAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchExpressions:
            - key: app
              operator: In
              values:
              - navius-api
          topologyKey: "kubernetes.io/hostname"
```

### Pod Disruption Budget

To ensure high availability during maintenance:

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: navius-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: navius-api
```

## Monitoring and Observability

### Prometheus Integration

Navius exports Prometheus metrics. Create a ServiceMonitor:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: navius-metrics
  labels:
    release: prometheus
spec:
  selector:
    matchLabels:
      app: navius-api
  endpoints:
  - port: http
    path: /actuator/prometheus
```

### Grafana Dashboards

The Navius Grafana dashboards can be imported to visualize metrics:

1. Dashboard for general application health (ID: 12345)
2. Dashboard for API endpoint metrics (ID: 12346)
3. Dashboard for service dependencies (ID: 12347)

## Troubleshooting

### Common Issues

1. **Pod fails to start**:
   - Check logs: `kubectl logs <pod-name>`
   - Verify resource limits: `kubectl describe pod <pod-name>`

2. **Service unreachable**:
   - Verify endpoints: `kubectl get endpoints navius-api`
   - Check service: `kubectl describe service navius-api`

3. **Configuration issues**:
   - Validate ConfigMap: `kubectl describe configmap navius-config`
   - Check environment variables in running pod

### Debugging Tools

```bash
# Shell into a pod for debugging
kubectl exec -it <pod-name> -- /bin/sh

# Check application logs
kubectl logs <pod-name> -f

# Check events
kubectl get events
```

## Best Practices

1. **Use namespaces** to isolate different environments (dev, staging, prod)
2. **Configure resource limits and requests** properly to avoid resource contention
3. **Implement proper health checks** using Navius's built-in health endpoints
4. **Use GitOps** for managing Kubernetes manifests
5. **Set up proper monitoring** with Prometheus and Grafana
6. **Use a CI/CD pipeline** for automated deployments
7. **Implement secrets management** using Kubernetes Secrets or external solutions
8. **Enable network policies** for additional security

## Related Resources

- [Production Deployment Guide](production-deployment.md) - General production deployment guidelines
- [Cloud Deployment Guide](cloud-deployment.md) - Cloud-specific deployment options
- [Environment Variables Reference](../../05_reference/configuration/environment-variables.md) - Configuration options
- [Monitoring Guide](../operations/monitoring.md) - Setting up monitoring for Navius

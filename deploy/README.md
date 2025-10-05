# Deployment Guide

This directory contains deployment configurations and scripts for the MD2DOCX Converter in various environments.

## Quick Start

### Local Development with Docker Compose

```bash
# Build and start services
./deploy.sh local --build

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Production Deployment

```bash
# Deploy to production Kubernetes cluster
./deploy.sh production v1.0.0 --build --health-check

# Rollback if needed
./deploy.sh production --rollback
```

## Deployment Options

### 1. Docker Compose (Recommended for Development)

**Files:**
- `docker-compose.yml` - Main compose configuration
- `nginx/nginx.conf` - Nginx reverse proxy configuration

**Usage:**
```bash
# Start all services
docker-compose up -d

# Start with monitoring stack
docker-compose --profile with-monitoring up -d

# Start with caching
docker-compose --profile with-cache up -d

# View logs
docker-compose logs -f md2docx-converter
```

**Services included:**
- `md2docx-converter` - Main application
- `nginx` - Reverse proxy (optional)
- `redis` - Caching (optional)
- `prometheus` - Metrics collection (optional)
- `grafana` - Monitoring dashboard (optional)

### 2. Kubernetes

**Files:**
- `kubernetes/deployment.yaml` - Complete Kubernetes manifests

**Features:**
- Horizontal Pod Autoscaler (HPA)
- Pod Disruption Budget (PDB)
- ConfigMap for configuration
- Secret for API keys
- Ingress for external access
- Health checks and probes

**Usage:**
```bash
# Deploy to Kubernetes
kubectl apply -f kubernetes/deployment.yaml -n md2docx-production

# Check status
kubectl get pods -n md2docx-production

# View logs
kubectl logs -f deployment/md2docx-converter -n md2docx-production

# Scale deployment
kubectl scale deployment/md2docx-converter --replicas=5 -n md2docx-production
```

### 3. Systemd Service

**Files:**
- `systemd/md2docx-converter.service` - Systemd service configuration

**Installation:**
```bash
# Create user and directories
sudo useradd -r -s /bin/false -m -d /opt/md2docx-converter md2docx
sudo mkdir -p /opt/md2docx-converter/{uploads,output}
sudo mkdir -p /var/log/md2docx

# Copy binary and configuration
sudo cp target/release/md2docx-server /opt/md2docx-converter/
sudo cp examples/config.yaml /opt/md2docx-converter/
sudo chown -R md2docx:md2docx /opt/md2docx-converter
sudo chown -R md2docx:md2docx /var/log/md2docx

# Install and start service
sudo cp deploy/systemd/md2docx-converter.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable md2docx-converter
sudo systemctl start md2docx-converter

# Check status
sudo systemctl status md2docx-converter
```

### 4. Manual Deployment

**Direct binary execution:**
```bash
# Build release binary
cargo build --release

# Set environment variables
export RUST_LOG=info
export PORT=3000
export OPENAI_API_KEY=your_api_key

# Run server
./target/release/md2docx-server
```

## Configuration

### Environment Variables

Create a `.env` file or set environment variables:

```bash
# Application
RUST_LOG=info
PORT=3000
HOST=0.0.0.0

# OpenAI API (optional)
OPENAI_API_KEY=your_api_key_here

# File limits
MAX_FILE_SIZE=104857600  # 100MB
MAX_BATCH_SIZE=50

# Rate limiting
RATE_LIMIT_PER_MINUTE=100
CONVERSION_RATE_LIMIT=10

# Directories
UPLOAD_DIR=/app/uploads
OUTPUT_DIR=/app/output
```

### Configuration File

Use a YAML configuration file for default formatting:

```yaml
# config.yaml
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: "Times New Roman"
    size: 12.0
```

## Security Considerations

### Production Security Checklist

- [ ] **API Keys**: Store OpenAI API key securely (Kubernetes secrets, environment variables)
- [ ] **HTTPS**: Enable SSL/TLS termination (nginx, ingress controller, or load balancer)
- [ ] **Authentication**: Implement API key authentication for production use
- [ ] **Rate Limiting**: Configure appropriate rate limits
- [ ] **CORS**: Set specific allowed origins instead of `*`
- [ ] **File Validation**: Ensure file size and type limits are enforced
- [ ] **Network Security**: Use firewalls and network policies
- [ ] **Container Security**: Run containers as non-root user
- [ ] **Secrets Management**: Use proper secrets management (Vault, AWS Secrets Manager, etc.)
- [ ] **Monitoring**: Set up logging and monitoring
- [ ] **Backup**: Implement backup strategy for persistent data

### Nginx Security Headers

The included nginx configuration adds security headers:
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Strict-Transport-Security` (HSTS)
- `Referrer-Policy: strict-origin-when-cross-origin`

## Monitoring and Logging

### Prometheus Metrics

The application exposes metrics at `/metrics`:
- Request count and duration
- Active connections
- Memory usage
- Conversion success/failure rates

### Grafana Dashboard

Import the provided dashboard configuration:
1. Access Grafana at `http://localhost:3001` (admin/admin)
2. Add Prometheus datasource: `http://prometheus:9090`
3. Import dashboard from `monitoring/grafana/dashboards/`

### Log Aggregation

For production, consider log aggregation:
- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Fluentd**: Log collection and forwarding
- **Cloud Solutions**: AWS CloudWatch, GCP Cloud Logging, Azure Monitor

## Scaling and Performance

### Horizontal Scaling

The application is stateless and can be scaled horizontally:

```bash
# Docker Compose
docker-compose up --scale md2docx-converter=3

# Kubernetes
kubectl scale deployment/md2docx-converter --replicas=5
```

### Performance Tuning

**Environment Variables:**
```bash
WORKER_THREADS=4           # Number of worker threads
MAX_CONNECTIONS=1000       # Maximum concurrent connections
REQUEST_TIMEOUT=300        # Request timeout in seconds
MAX_MEMORY_USAGE=1073741824  # Memory limit in bytes
```

**Resource Limits (Kubernetes):**
```yaml
resources:
  requests:
    memory: "256Mi"
    cpu: "250m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

### Load Balancing

Use a load balancer for high availability:
- **Nginx**: Reverse proxy with upstream servers
- **HAProxy**: High-performance load balancer
- **Cloud Load Balancers**: AWS ALB, GCP Load Balancer, Azure Load Balancer
- **Kubernetes Ingress**: Built-in load balancing

## Backup and Disaster Recovery

### Data to Backup

- Configuration files
- Custom templates (if any)
- Logs (for audit purposes)
- Metrics data (optional)

### Backup Strategy

```bash
# Configuration backup
tar -czf backup-$(date +%Y%m%d).tar.gz \
  config.yaml \
  docker-compose.yml \
  nginx/ \
  monitoring/

# Automated backup script
#!/bin/bash
BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
tar -czf "$BACKUP_DIR/md2docx-config-$DATE.tar.gz" /opt/md2docx-converter/
find "$BACKUP_DIR" -name "md2docx-config-*.tar.gz" -mtime +30 -delete
```

## Troubleshooting

### Common Issues

**Container won't start:**
```bash
# Check logs
docker-compose logs md2docx-converter

# Check resource usage
docker stats

# Verify configuration
docker-compose config
```

**High memory usage:**
```bash
# Monitor memory
docker stats --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}"

# Adjust memory limits
# Edit docker-compose.yml or Kubernetes resources
```

**Connection issues:**
```bash
# Test connectivity
curl http://localhost:3000/api/health

# Check port binding
netstat -tulpn | grep :3000

# Verify firewall rules
sudo ufw status
```

### Health Checks

**Manual health check:**
```bash
curl -f http://localhost:3000/api/health
```

**Kubernetes health check:**
```bash
kubectl get pods -n md2docx-production
kubectl describe pod <pod-name> -n md2docx-production
```

### Log Analysis

**View application logs:**
```bash
# Docker Compose
docker-compose logs -f md2docx-converter

# Kubernetes
kubectl logs -f deployment/md2docx-converter -n md2docx-production

# Systemd
sudo journalctl -u md2docx-converter -f
```

**Common log patterns to watch:**
- `ERROR` - Application errors
- `WARN` - Warnings and potential issues
- `rate limit exceeded` - Rate limiting triggered
- `memory allocation` - Memory issues
- `connection refused` - Network connectivity issues

## CI/CD Integration

### GitHub Actions

The included workflow (`.github/workflows/ci.yml`) provides:
- Automated testing
- Security scanning
- Multi-platform builds
- Docker image building
- Deployment automation

### Deployment Automation

**Trigger deployment on release:**
```yaml
# In GitHub Actions workflow
- name: Deploy to production
  if: github.event_name == 'release'
  run: |
    ./deploy/deploy.sh production ${{ github.event.release.tag_name }} --health-check
```

**Manual deployment:**
```bash
# Deploy specific version
./deploy.sh production v1.2.3 --build --health-check

# Deploy latest
./deploy.sh production latest --health-check
```

## Support and Maintenance

### Regular Maintenance Tasks

1. **Update dependencies**: `cargo update`
2. **Security patches**: Monitor and apply security updates
3. **Log rotation**: Ensure logs don't fill disk space
4. **Backup verification**: Test backup and restore procedures
5. **Performance monitoring**: Review metrics and optimize as needed
6. **Certificate renewal**: Update SSL certificates before expiration

### Monitoring Checklist

- [ ] Application health checks passing
- [ ] Memory usage within limits
- [ ] CPU usage normal
- [ ] Disk space available
- [ ] Network connectivity working
- [ ] SSL certificates valid
- [ ] Backup jobs running successfully
- [ ] Log aggregation working
- [ ] Metrics collection active
- [ ] Alerts configured and working

For additional support, see the main [README.md](../README.md) and [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md).
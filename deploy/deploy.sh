#!/bin/bash

# Deployment script for MD2DOCX Converter
# Usage: ./deploy.sh [environment] [options]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENVIRONMENT="${1:-development}"
DOCKER_IMAGE="md2docx-converter"
DOCKER_TAG="${2:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
MD2DOCX Converter Deployment Script

Usage: $0 [environment] [tag] [options]

Environments:
  development    Deploy to development environment (default)
  staging        Deploy to staging environment
  production     Deploy to production environment
  local          Deploy locally with docker-compose

Options:
  --build        Build Docker image before deployment
  --no-cache     Build without Docker cache
  --pull         Pull latest base images
  --migrate      Run database migrations (if applicable)
  --rollback     Rollback to previous version
  --health-check Perform health check after deployment
  --help         Show this help message

Examples:
  $0 development latest --build
  $0 production v1.2.3 --health-check
  $0 local --build --no-cache
EOF
}

# Parse command line arguments
BUILD_IMAGE=false
NO_CACHE=false
PULL_IMAGES=false
RUN_MIGRATIONS=false
ROLLBACK=false
HEALTH_CHECK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build)
            BUILD_IMAGE=true
            shift
            ;;
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --pull)
            PULL_IMAGES=true
            shift
            ;;
        --migrate)
            RUN_MIGRATIONS=true
            shift
            ;;
        --rollback)
            ROLLBACK=true
            shift
            ;;
        --health-check)
            HEALTH_CHECK=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            if [[ $1 != development && $1 != staging && $1 != production && $1 != local ]]; then
                DOCKER_TAG="$1"
            fi
            shift
            ;;
    esac
done

log_info "Starting deployment to $ENVIRONMENT environment with tag $DOCKER_TAG"

# Validate environment
case $ENVIRONMENT in
    development|staging|production|local)
        ;;
    *)
        log_error "Invalid environment: $ENVIRONMENT"
        log_info "Valid environments: development, staging, production, local"
        exit 1
        ;;
esac

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    # Check Docker Compose for local deployment
    if [[ $ENVIRONMENT == "local" ]] && ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    # Check kubectl for Kubernetes deployments
    if [[ $ENVIRONMENT != "local" ]] && ! command -v kubectl &> /dev/null; then
        log_warning "kubectl is not installed. Kubernetes deployment may not work."
    fi
    
    log_success "Prerequisites check completed"
}

# Build Docker image
build_image() {
    if [[ $BUILD_IMAGE == true ]]; then
        log_info "Building Docker image..."
        
        cd "$PROJECT_ROOT"
        
        BUILD_ARGS=""
        if [[ $NO_CACHE == true ]]; then
            BUILD_ARGS="$BUILD_ARGS --no-cache"
        fi
        
        if [[ $PULL_IMAGES == true ]]; then
            BUILD_ARGS="$BUILD_ARGS --pull"
        fi
        
        docker build $BUILD_ARGS -t "$DOCKER_IMAGE:$DOCKER_TAG" .
        
        log_success "Docker image built successfully"
    fi
}

# Deploy to local environment
deploy_local() {
    log_info "Deploying to local environment..."
    
    cd "$PROJECT_ROOT"
    
    # Create .env file if it doesn't exist
    if [[ ! -f .env ]]; then
        log_info "Creating .env file from template..."
        cp deploy/production.env .env
        log_warning "Please update .env file with your configuration"
    fi
    
    # Start services
    docker-compose down --remove-orphans
    docker-compose up -d
    
    log_success "Local deployment completed"
    log_info "Application available at: http://localhost:3000"
}

# Deploy to Kubernetes
deploy_kubernetes() {
    log_info "Deploying to Kubernetes ($ENVIRONMENT)..."
    
    # Check if kubectl is configured
    if ! kubectl cluster-info &> /dev/null; then
        log_error "kubectl is not configured or cluster is not accessible"
        exit 1
    fi
    
    # Create namespace if it doesn't exist
    NAMESPACE="md2docx-$ENVIRONMENT"
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply Kubernetes manifests
    cd "$SCRIPT_DIR/kubernetes"
    
    # Update image tag in deployment
    sed -i.bak "s|image: md2docx-converter:.*|image: $DOCKER_IMAGE:$DOCKER_TAG|g" deployment.yaml
    
    # Apply manifests
    kubectl apply -f deployment.yaml -n "$NAMESPACE"
    
    # Wait for deployment to be ready
    kubectl rollout status deployment/md2docx-converter -n "$NAMESPACE" --timeout=300s
    
    # Restore original deployment file
    mv deployment.yaml.bak deployment.yaml
    
    log_success "Kubernetes deployment completed"
    
    # Get service information
    kubectl get services -n "$NAMESPACE"
}

# Rollback deployment
rollback_deployment() {
    if [[ $ROLLBACK == true ]]; then
        log_info "Rolling back deployment..."
        
        case $ENVIRONMENT in
            local)
                log_warning "Rollback not supported for local deployment"
                ;;
            *)
                NAMESPACE="md2docx-$ENVIRONMENT"
                kubectl rollout undo deployment/md2docx-converter -n "$NAMESPACE"
                kubectl rollout status deployment/md2docx-converter -n "$NAMESPACE"
                log_success "Rollback completed"
                ;;
        esac
    fi
}

# Health check
perform_health_check() {
    if [[ $HEALTH_CHECK == true ]]; then
        log_info "Performing health check..."
        
        case $ENVIRONMENT in
            local)
                HEALTH_URL="http://localhost:3000/api/health"
                ;;
            *)
                # Get the service URL from Kubernetes
                NAMESPACE="md2docx-$ENVIRONMENT"
                SERVICE_IP=$(kubectl get service md2docx-converter-service -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
                if [[ -z "$SERVICE_IP" ]]; then
                    SERVICE_IP=$(kubectl get service md2docx-converter-service -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}')
                fi
                HEALTH_URL="http://$SERVICE_IP/api/health"
                ;;
        esac
        
        # Wait for service to be ready
        for i in {1..30}; do
            if curl -f -s "$HEALTH_URL" > /dev/null 2>&1; then
                log_success "Health check passed"
                return 0
            fi
            log_info "Waiting for service to be ready... ($i/30)"
            sleep 10
        done
        
        log_error "Health check failed"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up temporary files..."
    # Add cleanup logic here if needed
}

# Trap cleanup on exit
trap cleanup EXIT

# Main deployment flow
main() {
    check_prerequisites
    
    if [[ $ROLLBACK == true ]]; then
        rollback_deployment
        return 0
    fi
    
    build_image
    
    case $ENVIRONMENT in
        local)
            deploy_local
            ;;
        development|staging|production)
            deploy_kubernetes
            ;;
    esac
    
    perform_health_check
    
    log_success "Deployment completed successfully!"
    
    # Show useful information
    case $ENVIRONMENT in
        local)
            echo
            log_info "Useful commands:"
            echo "  View logs: docker-compose logs -f"
            echo "  Stop services: docker-compose down"
            echo "  Restart: docker-compose restart"
            ;;
        *)
            NAMESPACE="md2docx-$ENVIRONMENT"
            echo
            log_info "Useful commands:"
            echo "  View logs: kubectl logs -f deployment/md2docx-converter -n $NAMESPACE"
            echo "  Scale: kubectl scale deployment/md2docx-converter --replicas=5 -n $NAMESPACE"
            echo "  Status: kubectl get pods -n $NAMESPACE"
            ;;
    esac
}

# Run main function
main "$@"